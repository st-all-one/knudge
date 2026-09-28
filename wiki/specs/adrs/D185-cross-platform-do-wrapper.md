# D185 — Cross-platform do wrapper

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**Cross-platform do wrapper.** `shell_spec(script, os)` (pura) decide o shell: Unix (Linux/macOS/BSD) → `bash <script>`; Windows → `powershell -NoProfile -File <script>` para `.ps1`, recusando script Unix (`invalid_input`, 2) com o guia manual. `plan_command`/`shell_prefix` refletem o SO no `--dry-run`; `after_help` de `drain service` e matriz SO × script em `docs/15-embeddings.md`; smoke no CI (`windows-latest`, `cargo test --bin kd commands::script`). Fecha E18-T02.

## Impacto

- Fecha E18-T02.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
