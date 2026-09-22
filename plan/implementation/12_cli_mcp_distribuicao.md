# E12 — CLI, MCP, hooks e distribuição

> **Fase 4.** A superfície que o LLM e as máquinas consomem: `kd` com **pipe para LLM** e
> **`--json`** para máquinas, mensagens de erro **congeladas**, MCP **reativo a comportamento**,
> hooks de ciclo de vida e a distribuição em binário estático.
>
> **Decisões:** D57, D59, D60, D68, D69, D71, D72, D73, D88, D93, D94.
> **Políticas:** R12, R20, R21, R22, R23, R31, R33, R35 (ver [`14_revisao_tecnica.md`](14_revisao_tecnica.md)).

## Objetivo do épico

Uma interface única e estável, boa para tokens no caminho do LLM e estruturada para
integrações — sem servidor.

## Pré-requisitos

E08, E09.

## Tarefas

### E12-T01 ☑ CLI `kd` completa (superfície v2)
- **Objetivo:** expor a superfície congelada em [`16_cli_surface.md`](16_cli_surface.md) —
  `kd` (= `prime`), `init`, `prime`, `rewind`, `ask`, `write`, `task`, `maintenance`, `config`,
  `forget`, `sync`, `self` — com **pipe** (LLM) e **`--json`** (`{success, command, data?,
  error?, warnings?}`) para máquinas e exit codes.
- **Entregáveis:** parsing e dispatch; dois formatos por comando; `--link` em `write`; `task`
  com `scope` fechado (D93); `strict` lido do config (D94).
- **Decisões:** D57, D69, D71, D88, D93, D94.
- **Aceite:** matriz de aceite (E13-T06) verde; `kd` sem args == `kd prime`; exit codes
  consistentes; JSON válido; nenhuma flag `--strict`.

### E12-T02 ☑ Catálogo de mensagens e EPIPE
- **Objetivo:** catálogo de mensagens de erro/sucesso **congelado por teste**; **EPIPE → exit 0**
  (ex.: saída truncada por `head`).
- **Entregáveis:** catálogo; tratamento de pipe fechado.
- **Decisões:** D72, D73.
- **Aceite:** alterar uma mensagem quebra o teste (regressão detectada); `kd … | head` sai 0.

### E12-T03 ☑ MCP proativo (estreito)
- **Objetivo:** 3 gatilhos — pré-`write` (quase-duplicados, 1ª prioridade), pré-edição de
  arquivo (`kd rewind --files` contínuo), fim de sessão (`kd maintenance learn?` se houve diff e
  zero writes);
  hint é **ponteiro** (`id + statement + score`), cap **3**, dedup por sessão, **modo
  observação** por N sessões.
- **Entregáveis:** `knudge-mcp`; gatilhos; contador de seguimento.
- **Decisões:** D68.
- **Aceite:** hint nunca injeta conteúdo; cap e dedup respeitados; modo observação mensurável.

### E12-T04 ☑ Hooks de ciclo de vida
- **Objetivo:** `pre-record`, `post-record`, `pre-prime`, `pre-prune`, `pre-compact`; stdin
  JSON, mutação de payload, bloqueio, **timeout + process-group kill** e **redaction** de
  segredos.
- **Entregáveis:** `HookRunner` (port) + impl; configuração.
- **Decisões:** D59.
- **Aceite:** hook lento é morto no timeout (sem filhos órfãos); segredo não vaza no log;
  hook pode bloquear a operação.

### E12-T05 ☑ Distribuição
- **Objetivo:** binário **estático**, `kd self completions`, `kd self setup` (recipes
  `claude`/`cursor`/`codex`/`pi`), `kd self upgrade`.
- **Entregáveis:** pipeline de build; scripts.
- **Decisões:** D69.
- **Aceite:** instalar/atualizar em máquina limpa; completions geradas.

### E12-T06 ☑ `init` na CLI
- **Objetivo:** `kd init` aplica marcadores idempotentes + version marker e grava o
  `AGENTS.md` (integra E04-T04).
- **Entregáveis:** subcomando.
- **Decisões:** D57, D60.
- **Aceite:** re-rodar não duplica; version marker antigo atualiza o bloco.

### E12-T07 ☑ stdout vs stderr e logging estruturado
- **Objetivo:** o contrato de pipe/JSON **nunca** é contaminado por log.
- **Entregáveis:** dados só em stdout; logs só em stderr; `--log-level`/`RUST_LOG`/`--quiet`;
  `tracing-subscriber` com `EnvFilter`; campos estruturados de métrica; redação (R22);
  `context_id`/`session_id` como span raiz.
- **Decisões:** D71, D73. **Políticas:** R20, R21, R22, R23.
- **Aceite:** `kd … --json 2>/dev/null` é JSON válido; pipe limpo; segredo não aparece.

### E12-T08 ☑ Envelope de erro, warnings e mapa de exit
- **Objetivo:** máquina e humano entendem a falha sem parsear prosa.
- **Entregáveis:** `error: { code, message, retryable, details? }` + `warnings[]`; código
  estável → exit code (101 reservado a panic); **`strict` (config de projeto, D94) promove
  warning a erro**; `--json` documentado quanto a panic/abort.
- **Decisões:** D71, D72, D73. **Políticas:** R31, R33, R35.
- **Aceite:** matriz código×exit; agente distingue `retryable`; warning não aborta.

## Definition of Done

- [x] Todos os verbos com pipe + `--json` + exit codes.
- [x] Mensagens congeladas e EPIPE tratado.
- [x] MCP restrito aos 3 gatilhos; hooks seguros.
- [x] Distribuição instalável.
- [x] stdout/stderr separados e envelope com código/`retryable`/`warnings[]`.

## Não-objetivos

- FFI/WASM (D68).
- Migração de seeds/mulch (D70).

## Entregue (E12)

- **T01** — 12 verbos wireados ao domínio: `init`, `prime`, `rewind`, `ask` (recall/get/expand),
  `write` (create/update/link), `task` (new/list/show/update/close/plan), `maintenance`
  (doctor/audit/compact/eval/index/learn), `config` (get/set/unset/list), `forget`
  (soft/restore/purge), `sync`, `self`. `Session` (`src/session.rs`) é a única borda que toca
  adaptadores reais.
- **T02** — EPIPE → exit 0 (`output::emit_stdout`); mensagens canônicas em `commands/*`.
- **T03** — motor de gatilhos puro em `knudge-mcp/src/triggers.rs` (3 gatilhos, cap 3, dedup por
  sessão, modo observação). **Transporte JSON-RPC fica em E13** (o core de gatilhos é testado).
- **T04** — `ports::HookRunner` + `adapters::ProcessHookRunner` (timeout + kill do grupo de
  processos, sem shell); orquestração em `commands/hooks.rs` (`pre-record` bloqueia/muta,
  `post-record`, `pre-prune`, `pre-compact`; `pre-prime` reservado porque `prime` é estático).
- **T05** — `kd self completions <bash|zsh|fish>` (scripts embutidos, sem dep nova) e
  `kd self setup <claude|cursor|codex|pi>` (recipe em `.knudge/setup/`); `upgrade` orienta o
  canal de origem.
- **T06** — `kd init` chama `git::onboard` (idempotente) e emite o prompt de fundação.
- **T07** — stdout = dados, stderr = logs (`tracing` + redação); `--json 2>/dev/null` é JSON
  válido.
- **T08** — envelope `{success, command, data?, error{code,message,retryable}, warnings?}`;
  `strict` (config de projeto) promove warnings a erro.
