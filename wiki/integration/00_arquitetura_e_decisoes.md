# 00 — Arquitetura e decisões para quem integra

Este documento resume o **modelo mental** do `knudge-core` e as decisões que afetam a API pública
de um projeto que o embute. Para o detalhamento completo, veja
[`../specs/ARCHITECTURE.md`](../specs/ARCHITECTURE.md).

## Camadas

```
seu projeto Rust
   │  usa a fachada (adapters std) ou as portas (fakes/próprias)
   ▼
knudge-core
   ├── domínio puro   (schema, toon, store, retrieval, write, task, handoff, ...)
   ├── ports          (traits: Clock, Rng, Env, Fs, Git, HookRunner, Logger, Embedder)
   └── adapters       (StdFs, StdGit, StdEnv, SystemClock, HttpEmbedder, ...) — opcional
```

**Regra central (D65):** o domínio **nunca** toca terminal, `argv`, relógio/RNG global ou sistema
de arquivos direto — só por portas. Isso dá três propriedades úteis para integração:

1. **Testabilidade:** com fakes (`ports::fakes`) você reproduz byte a byte.
2. **Portabilidade:** troque `StdFs` por outra `Fs` (memória, remota, sandbox) sem tocar no
   domínio.
3. **Confinamento:** `unsafe` é proibido (`#![forbid(unsafe_code)]`) e a impureza fica em
   `adapters`.

## Dois níveis de API

| Nível | Quando usar | Entrada |
|---|---|---|
| **Fachada** | Quero a mesma montagem da CLI/MCP (adapters `std` + projeto + config) | `Knudge`/`KnudgeBuilder` |
| **Portas + domínio** | Quero injetar `Clock`/`Fs`/`Git`/`Embedder` próprios, embutir num runtime, testar | `Project`, `Store`, `Index`, `recall`, `write`, `ports::fakes` |

Ambos acessam **o mesmo** domínio. A fachada é conveniência; o domínio é a fundação.

## Invariantes que você pode assumir

- **Sem `unsafe`** (`#![forbid(unsafe_code)]`).
- **Sem panic** no caminho de produção: funções retornam `Result<T>` (`unwrap`/`expect`/`panic`
  são proibidos por lint).
- **Determinismo:** toda iteração sobre coleções tem ordem estável (`BTreeMap`/`IndexMap`, nunca
  `HashMap`); ids/hashes derivam de conteúdo.
- **Aritmética checada:** `overflow-checks` ligado em dev e release.
- **Contrato de bytes:** frontmatter TOON, ids, hashes e ordem de chaves são contrato
  (`wiki/specs/TOON.md`); mudanças exigem `schema_version`/rebuild.

## Layout de conhecimento configurável (D214)

`notas/`, `eventos/`, `config.toml` etc. vivem em `<raiz>/<layout>/`. O `layout` é um caminho
**relativo** (default `.knudge`), transportado pelo `Project`:

```rust
use knudge_core::git::Project;

let project = Project::at("/meu/projeto", ".a/b")?; // sem Git
let knowledge = project.knowledge_dir();             // /meu/projeto/.a/b
```

O layout propaga para os padrões de Git (`info/exclude`, `.gitattributes`, `sync`, `AGENTS.md`) e
para a varredura de âncoras (`Project::ignored_dirs`). **Não** existe chave de config para o
layout: ele é injetado (evita o ciclo config↔diretório).

## Erros são valores, não exceções

Toda API de domínio devolve `knudge_core::Result<T>` = `Result<T, Error>`. `ErrorKind` é o
**contrato de máquina**; `exit_code()` mapeia para processos. Nunca há `Box<dyn Error>` na API
pública. Detalhes em [12 — Erros e logs](12_erros_logs_e_determinismo.md).

## Leitura tolerante x escrita estrita

- **Leitura** tolera nota malformada: chave desconhecida → aviso; `type` desconhecido → nota
  **pulada** com orientação, sem derrubar o corpus (D16–D18).
- **Escrita** é estrita: chave desconhecida é rejeitada, opcionais vazios são **omitidos** (nunca
  `null`).
- **Degradação graciosa (R33):** canal/recurso opcional que falha devolve resultado **parcial** +
  `warnings[]`. Em `behavior.strict`, o aviso vira erro.

## Decisões que mais impactam integradores

| D | Efeito prático |
|---|---|
| D01 | `id` é endereçado por conteúdo (`type + U+001F + normalize(statement)`); o prefixo é histórico. |
| D05/D16/D17 | Escrita estrita na forma; opcionais omitidos; tipo desconhecido não derruba a leitura. |
| D20/D21 | Escrita atômica (tmp+rename) e **nota antes do evento**; crash deixa nota sem evento. |
| D23–D25 | Lock advisory por alvo; `LockGuard` é RAII. |
| D34 | `persist_in_project` decide o que vai ao Git. |
| D65 | Domínio puro + portas + adapters. |
| D79 | `Embedder` é plugável; troca de modelo invalida o índice. |
| D87/D189 | Confiança é **derivada** (Beta-Bernoulli), nunca armazenada. |
| D92 | Sem `unwrap`/`panic`; arquivos ≤ 300 linhas. |
| D95 | `normalize` = NFC + trim + colapso; `body_hash`/`id` derivam dele. |
| D214 | Núcleo publicável + layout configurável + fachada `Knudge`. |

## Checklist de decisão antes de codar

1. **Vou usar a fachada ou injetar portas?** (Se precisa de mock/relógio fixo, portas.)
2. **O layout é `.knudge`?** Se não, passe `knowledge_dir` e garanta que seu onboarding/sync use o
   mesmo.
3. **Preciso de determinismo total?** Use `FixedClock`/`SeqRng`/`MemFs`/`FakeGit`.
4. **Quem cuida do índice?** Reconstrua com `Corpus`/`load_fresh` e reaproveite (doc 13).
5. **O que faço com `warnings`?** Propague na sua API; em `strict`, deixe virar erro.
