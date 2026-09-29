# Integração do `knudge-core` em outro projeto Rust

Esta pasta documenta como usar o **`knudge-core`** como **biblioteca** dentro de outro projeto
Rust: dependência, montagem da sessão, cada subsistema (modelo, store, busca, escrita, tarefas,
grafo, saúde, ciclo de vida, embeddings, git/config) e como **manter o funcionamento otimizado**.

> **Escopo.** A CLI `kd` e o servidor `knudge-mcp` são **bordas** (parsing, saída, gatilhos MCP,
> instalação). O núcleo contém o **domínio**: tudo que é memória, busca, escrita, tarefas,
> handoff, saúde, ciclo de vida e embeddings. Este guia cobre o núcleo.

## Quando usar o core direto

| Você quer… | Use |
|---|---|
| Embutir a memória num serviço/agente Rust | **`knudge-core`** (esta wiki) |
| Usar o binário/CLI | `kd` (`wiki/usage/`) |
| Gatilhos proativos via MCP | `knudge-mcp` (`wiki/usage/17_mcp.md`) |
| Só buscar/gravar notas de um diretório próprio | `Store`/`Index`/`write` diretamente (docs 04–06) |

O que **não** está no core (por decisão): motor de hints do MCP, texto do `prime`, materialização
de `MAP.md`/hubs, instalação/`self`/scripts e o envelope de saída — são responsabilidade da borda.

## Pré-requisitos

- **Rust 1.97+** (edição 2024). MSRV é contrato.
- O crate expõe `Result<T> = Result<T, Error>`; trate `ErrorKind` (`wiki/specs/erros.md`).
- O domínio é **puro**: relógio/RNG/FS/ambiente/Git entram por **portas** (`ports`). A fachada
  `Knudge` monta as implementações `std` (`adapters`); quem quiser determinismo total injeta
  fakes.

## Adicionar a dependência

```toml
# Cargo.toml do seu projeto
[dependencies]
knudge-core = "0.5"           # quando publicado
# ou, durante o desenvolvimento:
# knudge-core = { path = "../knudge/core/crates/knudge-core" }
```

## Exemplo mínimo (fachada)

```rust
use knudge_core::{Knudge, Result};

fn main() -> Result<()> {
    // default: resolve o worktree principal e usa `.knudge/`
    let kd = Knudge::open()?;

    let corpus = kd.corpus()?; // notas + índice + grafo numa só passada
    println!("{} notas indexadas", corpus.notes.len());
    Ok(())
}
```

Diretório de conhecimento alternativo (aceita aninhado):

```rust
let kd = Knudge::builder().knowledge_dir(".a/b").open()?;
```

## Documentos

| # | Documento | Cobre |
|---|---|---|
| [00](00_arquitetura_e_decisoes.md) | Arquitetura e decisões | camadas, portas/adaptadores, determinismo, o que é contrato |
| [01](01_instalacao_e_fachada.md) | Instalação e fachada | `Knudge`/`KnudgeBuilder`, raiz, layout, ciclo de vida da sessão |
| [02](02_modelo_de_dados.md) | Modelo de dados | schema, enums, ids/hashes, TOON, `Note`/`Frontmatter`/`Value` |
| [03](03_store_e_persistencia.md) | Store e persistência | atomicidade, ordem de commit, lock, rebuild, purge, resíduos |
| [04](04_busca_e_recall.md) | Busca e recall | `Index`, `Graph`, `recall`, filtros, views, âncoras, `get`, `rank`/`tags`/`suggest` |
| [05](05_escrita_e_ciclo_de_vida.md) | Escrita | `WriteContext`, `Draft`, dedup, `update`/supersede, `forget`/`link`, lote |
| [06](06_tarefas_e_handoff.md) | Tarefas e handoff | `TaskSpec`/`submit`, fluxo, `rewind`, `context_id`, orçamento |
| [07](07_grafo_e_inferencia.md) | Grafo e inferência | arestas, integridade/ciclos, ontologia, TMS, PageRank/PPR, comunidades |
| [08](08_saude_e_manutencao.md) | Saúde e manutenção | `doctor`/`audit`/validators/gate, `diff`/`learn`/`compact` |
| [09](09_ciclo_de_vida.md) | Ciclo de vida | confiança Beta, shelf-life, decay, retenção, clusters, drift, demolição |
| [10](10_embeddings.md) | Embeddings | porta `Embedder`, cache, `drain`, `EmbeddingIndex`, ranking/sugestão |
| [11](11_git_e_config.md) | Git e configuração | `Project`, `onboard`, exclude/attributes, `sync`, config 2 níveis, segredos |
| [12](12_erros_logs_e_determinismo.md) | Erros, logs e determinismo | `Error`/exit codes, `warnings`/`strict`, `Logger`/redação, fakes |
| [13](13_performance_e_otimizacao.md) | Performance | playbook de otimização e checklist de produção |

## Convenções

- **Português** na prosa; identificadores de código em inglês.
- Cada documento cita os `Dxx` que o regem (em `plan/03_decisoes-fechadas.md`) e os módulos onde o
  comportamento vive.
- Fonte da verdade técnica: [`../specs/`](../specs/README.md). Este guia é **orientado à
  integração**, não substitui os specs.
