# E12 — CLI, MCP, hooks e distribuição

> **Fase 4.** A superfície que o LLM e as máquinas consomem: `kd` com **pipe para LLM** e
> **`--json`** para máquinas, mensagens de erro **congeladas**, MCP **reativo a comportamento**,
> hooks de ciclo de vida e a distribuição em binário estático.
>
> **Decisões:** D57, D59, D60, D68, D69, D71, D72, D73.
> **Políticas:** R12, R20, R21, R22, R23, R31, R33, R35 (ver [`14_revisao_tecnica.md`](14_revisao_tecnica.md)).

## Objetivo do épico

Uma interface única e estável, boa para tokens no caminho do LLM e estruturada para
integrações — sem servidor.

## Pré-requisitos

E08, E09.

## Tarefas

### E12-T01 ☐ CLI `kd` completa
- **Objetivo:** expor todos os verbos (`recall`, `get`, `expand`, `write`, `update`, `link`,
  `forget`, `restore`, `prime`, `diff`, `learn`, `plan`, `compact`, `audit`, `doctor`, `sync`,
  `onboard`, `config`, `eval`, `embed`), com **pipe** (LLM) e **`--json`**
  (`{success, command, error}`) para máquinas e exit codes.
- **Entregáveis:** parsing e dispatch; dois formatos por comando.
- **Decisões:** D71.
- **Aceite:** matriz de aceite (E13-T06) verde; exit codes consistentes; JSON válido.

### E12-T02 ☐ Catálogo de mensagens e EPIPE
- **Objetivo:** catálogo de mensagens de erro/sucesso **congelado por teste**; **EPIPE → exit 0**
  (ex.: saída truncada por `head`).
- **Entregáveis:** catálogo; tratamento de pipe fechado.
- **Decisões:** D72, D73.
- **Aceite:** alterar uma mensagem quebra o teste (regressão detectada); `kd … | head` sai 0.

### E12-T03 ☐ MCP proativo (estreito)
- **Objetivo:** 3 gatilhos — pré-`write` (quase-duplicados, 1ª prioridade), pré-edição de
  arquivo (`prime(files)` contínuo), fim de sessão (`learn()?` se houve diff e zero writes);
  hint é **ponteiro** (`id + statement + score`), cap **3**, dedup por sessão, **modo
  observação** por N sessões.
- **Entregáveis:** `knudge-mcp`; gatilhos; contador de seguimento.
- **Decisões:** D68.
- **Aceite:** hint nunca injeta conteúdo; cap e dedup respeitados; modo observação mensurável.

### E12-T04 ☐ Hooks de ciclo de vida
- **Objetivo:** `pre-record`, `post-record`, `pre-prime`, `pre-prune`, `pre-compact`; stdin
  JSON, mutação de payload, bloqueio, **timeout + process-group kill** e **redaction** de
  segredos.
- **Entregáveis:** `HookRunner` (port) + impl; configuração.
- **Decisões:** D59.
- **Aceite:** hook lento é morto no timeout (sem filhos órfãos); segredo não vaza no log;
  hook pode bloquear a operação.

### E12-T05 ☐ Distribuição
- **Objetivo:** binário **estático**, `completions`, `setup` (recipes `claude`/`cursor`/
  `codex`/`pi`), `upgrade`.
- **Entregáveis:** pipeline de build; scripts.
- **Decisões:** D69.
- **Aceite:** instalar/atualizar em máquina limpa; completions geradas.

### E12-T06 ☐ `onboard` na CLI
- **Objetivo:** `kd onboard` aplica marcadores idempotentes + version marker e grava o
  `AGENTS.md` (integra E04-T04).
- **Entregáveis:** subcomando.
- **Decisões:** D57, D60.
- **Aceite:** re-rodar não duplica; version marker antigo atualiza o bloco.

### E12-T07 ☐ stdout vs stderr e logging estruturado
- **Objetivo:** o contrato de pipe/JSON **nunca** é contaminado por log.
- **Entregáveis:** dados só em stdout; logs só em stderr; `--log-level`/`RUST_LOG`/`--quiet`;
  `tracing-subscriber` com `EnvFilter`; campos estruturados de métrica; redação (R22);
  `context_id`/`session_id` como span raiz.
- **Decisões:** D71, D73. **Políticas:** R20, R21, R22, R23.
- **Aceite:** `kd … --json 2>/dev/null` é JSON válido; pipe limpo; segredo não aparece.

### E12-T08 ☐ Envelope de erro, warnings e mapa de exit
- **Objetivo:** máquina e humano entendem a falha sem parsear prosa.
- **Entregáveis:** `error: { code, message, retryable, details? }` + `warnings[]`; código
  estável → exit code (101 reservado a panic); `--strict` promove warning a erro; `--json`
  documentado quanto a panic/abort.
- **Decisões:** D71, D72, D73. **Políticas:** R31, R33, R35.
- **Aceite:** matriz código×exit; agente distingue `retryable`; warning não aborta.

## Definition of Done

- [ ] Todos os verbos com pipe + `--json` + exit codes.
- [ ] Mensagens congeladas e EPIPE tratado.
- [ ] MCP restrito aos 3 gatilhos; hooks seguros.
- [ ] Distribuição instalável.
- [ ] stdout/stderr separados e envelope com código/`retryable`/`warnings[]`.

## Não-objetivos

- FFI/WASM (D68).
- Migração de seeds/mulch (D70).
