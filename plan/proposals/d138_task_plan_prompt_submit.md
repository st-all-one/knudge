# D138 — `kd task plan` fica só com `--prompt`/`--submit`

> **Status:** decisão fechada e **implementada** (v0.3.0). Origem: revisão do ciclo do plano.
> Revisa **D53/D105**.

## 1. Decisão

`kd task plan` se reduz a duas operações:

- `--prompt [--template feature|bug|refactor]` — imprime o prompt TOON do plano (read-only).
- `--submit --from -|FILE` / `--step` — valida e cria os filhos **atomicamente**.

**Saem as flags:** `--adopt`, `--release`, `--review`, `--reorder`.

- `--adopt`/`--release`/`--review` eram **apelidos de transição de status** (`in_progress`/
  `active`/`closed`), já cobertos por `kd task update <ID> --status` e por `kd task close`
  (com evidência, D55). `--review` ainda **furava** a regra de evidência, fechando sem validators.
- `--reorder` (`blocks`, posição 1-based entre irmãos) sai; a ordem, quando existir, vem do
  **próprio plano submetido** (`PlanStep.blocks`).

## 2. Ressalva de implementação

`TaskAction::Review`/`apply` **não** são exclusivos do `plan`: `kd task close --outcome` também
os usa (`commands/task/mutate.rs:103`). Portanto a remoção é das **flags**; no core ficam mortos
apenas `TaskAction::{Adopt, Release}` e `lifecycle::reorder`. `apply`/`validate_transition`
permanecem (usados pelo `close`/`update`).

## 3. Raio de alcance (quando executar)

- `cli/task.rs` (`TaskPlanArgs`), `commands/task/plan.rs` (dispatch `lifecycle`),
  `task/lifecycle.rs` (remove `Adopt`/`Release` e `reorder`; mantém `Review`/`apply`),
  `task/mod.rs` (exports), `docs/05-task.md`, `prime.rs`, goldens/testes de plano.

## 4. Aceite

- [ ] `kd task plan --adopt|--release|--review|--reorder` → exit 2.
- [ ] `--prompt`/`--submit` intactos (prompt TOON + submissão atômica).
- [ ] Fechamento só por `kd task close` (com evidência).
- [ ] `TaskAction::{Adopt, Release}`/`reorder` removidos; `close` segue usando `Review`/`apply`.
- [ ] Docs, `prime` e goldens atualizados; `make check` verde.
