---
name: knudge
description: Use ao trabalhar com memória de projeto por LLM via knudge (`kd`) — buscar, gravar, planejar tarefas, retomar contexto e manter a base. Dispare em palavras-chave: knudge, kd, memória, nota, recall, ask, write, tarefa, task, épico, handoff, rewind, TOON, embedding, RRF, MCP.
---

# knudge — Active Usage Guide

**Papel:** dar ao agente **memória durável por projeto** — buscar antes de gravar, registrar
conhecimento, planejar/executar tarefas e retomar contexto — com um binário único (`kd`), sem
servidor e sem banco. A **nota Markdown é a verdade**; o índice (BM25, embeddings, grafo) é
**derivado** e reconstruível.

**Quando carregar:** sempre que a tarefa envolver lembrar/registrar decisões, fatos, erros,
riscos ou perguntas do projeto; planejar ou acompanhar trabalho; retomar contexto entre sessões;
ou buscar conhecimento já acumulado.

> **Detalhe vive nos docs.** Este guia é o índice orientado a decisão. Referência completa da
> CLI: [`plan/implementation/16_cli_surface.md`](plan/implementation/16_cli_surface.md). Matriz
> de aceite: [`17_matriz_aceitacao.md`](plan/implementation/17_matriz_aceitacao.md). Contrato de
> bytes: [`TOON.md`](TOON.md). Arquitetura: [`ARCHITECTURE.md`](ARCHITECTURE.md). Decisões:
> [`03_decisoes-fechadas.md`](plan/03_decisoes-fechadas.md). Embeddings:
> [`04_embeddings.md`](plan/04_embeddings.md). MCP:
> [`18_mcp_transporte.md`](plan/implementation/18_mcp_transporte.md).

---

## Ciclo central

```
kd ask → kd write → kd task → kd sync
(buscar)  (gravar)   (executar) (commit)
```

**Sempre busque antes de gravar** — o `write` faz dedup (0.75/0.92). O `prime` é o protocolo
estático (byte-idêntico): `kd` sem argumentos = `kd prime`.

### Capacidades-chave

- **Busca híbrida** — `kd ask` combina filtros determinísticos → BM25 → âncoras → RRF (e o canal
  vetorial quando há índice), tudo num só envelope.
- **Gravação idempotente** — create por conteúdo, merge em quase-duplicata, rejeição em
  duplicata exata; `--update` versiona; `--link` cria aresta.
- **Tarefas com hierarquia fechada** — `epic ⊃ { issue ⊃ task | task }` (épico é a raiz); rollup de
  progresso por épico; fechar exige evidência.
- **Handoff ponto-no-tempo** — `kd rewind` monta o contexto dentro de um orçamento de tokens,
  com `next:`/`fresh:` e `--resume` 1:1.
- **Manutenção read-only** — `doctor`, `learn`, `compact`, `prune` **só propõem**; nada muda sem
  aceite (D47).
- **MCP** — `knudge-mcp` serve 4 tools de gatilho; os hints são ponteiros.

### Quick Reference

| Ação | Comando |
|------|---------|
| Protocolo (o "help da IA") | `kd prime` / `kd` |
| Buscar conhecimento | `kd ask "<query>" --brief` |
| Corpo de ids | `kd ask --id <ID>...` |
| Expandir o grafo | `kd ask --around <ID> [--via ARESTA] [--depth N]` |
| Mais confiáveis (sem query) | `kd ask --rank` |
| Vocabulário de tags | `kd ask --tags` |
| Gravar fato/decisão/erro/risco/pergunta | `kd write --type <T> "<...>" [--tag T] [--anchor PATH]` |
| Versionar | `kd write --update <ID> "<...>"` |
| Aresta explícita | `kd write --link <FROM:ARESTA:TO>` |
| Evidência numa nota | `kd write --outcome <success\|partial\|failure\|abandoned> <ID> [--note TXT]` |
| Nova tarefa | `kd task new "<...>" --scope <plan\|epic\|issue\|task> [--parent ID]` |
| Listar prontas/bloqueadas | `kd task list --ready` / `--blocked [--explain]` |
| Contexto do item | `kd task show <ID>` |
| Fechar com evidência | `kd task close <ID> --outcome success --note "..."` |
| WBS | `kd task graph [--program plan/<slug>.md\|--root ID]` |
| Handoff | `kd rewind [--scope C] [--files PATH...] [--budget N]` |
| Mapa de conhecimento | `kd knowledge map [--axis A] [--semantic] [--members]` |
| Manutenção | `kd maintenance doctor [--audit]` |
| Esquecer (soft) | `kd forget <ID>` (`--restore`, `--purge`) |
| Commit | `kd sync` |

### Âncoras (`--anchor PATH`) — o que liga memória a código

Ancorar é amarrar a nota/tarefa a um arquivo ou glob. É o canal que faz o `ask` responder “o
que já sei sobre `src/gateway.rs`” mesmo sem query textual.

- **Use em toda nota/tarefa que fala de código:**
  `kd write --type decision "..." --anchor src/gateway.rs` e
  `kd task new "..." --scope task --anchor plan/016.md`.
- **Repetível e com vírgula:** `--anchor src/a.rs --anchor src/b.rs` ou `--anchor src/a.rs,src/b.rs`.
- **Glob casa subárvores:** `--anchor src/gateway/**`.
- **Busca por âncora (sem query):** `kd ask --anchor src/gateway.rs`.
- **NÃO ancore** nota de conceito global (sem arquivo) nem path que ainda não existe.
- **Manutenção:** `kd maintenance doctor --audit` lista âncoras quebradas (arquivo removido).
- **Alias:** `--anchors` (plural) continua aceito em `write`/`task new`.

---

## Orientação (leia primeiro)

O knudge é **medir/registrar, não adivinhar**. Três invariantes:

1. **Busque antes de gravar.** `kd ask "<rascunho>"`; `< 0.75` cria, `0.75–0.92` merge, `≥ 0.92`
   rejeita.
2. **stdout = dados; stderr = logs.** Para máquina, `--json` =
   `{success, command, data?, error?, warnings?}`.
3. **Nunca invente id.** O id deriva de `type + statement`; reclassificar não reescreve.

Regras de bolso:

- **Nota boa é curta e autocontida** — um fato por nota; ancore código com `--anchor PATH`.
- **Classifique certo** — `foundational` (dura), `tactical` (muda), `observational` (efêmera).
- **Confirmação é derivada** — tarefas com `--outcome` de sucesso que compartilham âncoras
  confirmam a nota (X1/D108); não escreva "confirmado" à mão.
- **Nada sem aceite** — `learn`/`compact`/`prune` propõem; você aplica via
  `write`/`write --link`/`forget`.
- **Orçamento** — `kd rewind --budget N` corta em `ceil(len/4)` tokens; use `--brief` no `ask`
  para gastar menos contexto.

---

## Interpretação (o que ler na saída)

| Sinal | Significado / ação |
|---|---|
| `why = lexical` | Casou por BM25 (termos). |
| `why = semantic` | Casou pelo vetor (paráfrase). |
| `why = anchor` / `file_match` | Casou pela âncora de arquivo. |
| `score < 0.75` no `write` | Cria nota nova. |
| `0.75–0.92` | Merge na existente (revise antes). |
| `≥ 0.92` | Rejeita (duplicata). |
| `warnings[]` | Degradação graciosa (ex.: embeddings fora do ar → BM25). Com `strict`, vira erro. |
| `stale`/`expiring`/`pending` no `rewind` | Nota desatualizada / perto de expirar / embedding na fila. |
| `epico: <id>|<título> (<done>/<total>)` | Progresso do épico (folhas de trabalho fechadas). |
| `next:` no `rewind` | Tarefas `ready` abertas por impacto. |

---

## Workflows

### Registrar conhecimento

```bash
kd ask "rate limit do gateway" --brief                     # 1. já existe?
kd write --type decision "Rate limit é 100 rps por chave" \
  --tag gateway --anchor src/gateway.rs                    # 2. grava
kd write --link "decision_01m81b6h:refines:fact_01abc123"  # 3. relaciona
```

### Planejar e executar tarefas

```bash
kd task new "Sync offline-first" --scope epic
kd task new "Resolver conflito de merge" --scope task --parent <epic>
kd task list --ready --sort impact
kd task claim <task> --by agente-a
kd task close <task> --outcome success --note "testes verdes"
```

### Retomar contexto (handoff)

```bash
kd rewind --budget 2000                 # manifest + next:/fresh:
kd rewind --files src/gateway.rs        # só o working set
kd rewind --resume <context_id>         # retoma 1:1
```

### Auditar / manter

```bash
kd maintenance doctor --audit           # integridade + arestas sugeridas
kd maintenance learn                    # o que deveria virar nota?
kd knowledge map --axis scope --semantic
kd maintenance prune                    # propõe forget por shelf-life
kd maintenance index --status           # fila de embeddings (pending) por projeto
kd maintenance watch-service --status   # saúde do worker de auto-drain (systemd/launchd)
```

O worker de auto-drain é gerenciado por `kd maintenance watch-service`: `--install` (pré-flight +
agendador `systemd --user`/`launchd` + servidor de embeddings persistente `knudge-embed` +
cadastra o projeto), `--subscribe`/`--unsubscribe` (multi-projeto; não desinstalam o sistema),
`--status` (default) e `--uninstall`. O script é **embutido** no binário (sem download) e o GGUF
mora ao lado do `config.toml` global. Com `embeddings.mode=lazy` (default) o próprio `kd` já
drena um lote ao fim de cada verbo.

### Integração MCP

Configure o `knudge-mcp` no cliente (`kd self setup claude|cursor|codex|pi`) e use as tools:
`knudge_pre_write` (antes de gravar), `knudge_pre_edit` (antes de editar arquivo),
`knudge_session_end` (fim de sessão) e `knudge_status`.

---

## Instalação

```bash
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/st-all-one/knudge/main/install.sh | bash
```

Instala `kd` + `knudge-mcp` em `~/.local/bin` (checksum SHA-256). Requer `git` no projeto;
embeddings são opcionais. `kd init` funda `.knudge/` e escreve o bloco no `AGENTS.md`.

---

## Anti-patterns

- **Gravar sem buscar** → duplicata. Sempre `kd ask "<rascunho>"` antes.
- **Inventar id** — ids são derivados; use o que o `write`/`ask` retorna.
- **`kd write --type task`** — rejeitado; use `kd task`.
- **Criar aresta por flag de tarefa** — arestas têm via única: `kd write --link`.
- **Esperar que `learn`/`compact`/`prune` mudem o corpus** — eles só propõem.
- **Misturar log e dado** — nunca escreva log em stdout; em `--json`, stdout é só o envelope.
- **Declarar tarefa concluída sem evidência** — `kd task close` exige `--outcome`.
- **Versionar `.idx/`/`cache/`/`contexts/`** — são derivados; o `kd init` cuida do `.gitignore`.

---

## Limitações conhecidas

- Embeddings exigem um servidor local OpenAI-compatible; sem ele, `ask` degrada para BM25.
- `forgotten`/`superseded` não aparecem no `ask` por padrão (`--status` inclui).
- `kd write` rejeita `task`/`epic` (D93/D149).
- Tarefas têm o épico como raiz (`epic ⊃ { issue ⊃ task | task }`); o issue é opcional.
- `learn`/`compact`/`prune` são read-only (propostas) — a aplicação é manual.
- O índice é derivado e reconstruível; trocar o modelo de embedding invalida e re-embeda tudo.

---

## Checklist

Antes de concluir qualquer operação de memória:

- [ ] **Busquei antes de gravar** (`kd ask "<rascunho>"`)?
- [ ] **Statement curto e autocontido**, com `--anchor` do código?
- [ ] **Classificação/status** corretos (`foundational`/`tactical`/`observational`)?
- [ ] **Id não inventado** — copiado da saída?
- [ ] **Arestas via `kd write --link`**?
- [ ] **Tarefa com pai único e profundidade ≤ 4**?
- [ ] **Fechamento com `--outcome`** (evidência)?
- [ ] **`kd sync`** no fim para versionar `notas/` + `eventos/`?
- [ ] **`--json 2>/dev/null`** continua JSON válido (sem log vazando)?
