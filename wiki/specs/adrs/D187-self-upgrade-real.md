# D187 — self upgrade real

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**.

## Decisão

**`self upgrade` real (script).** `kd self upgrade` deixa de ser stub: evoca o `kd-upgrade.sh` (embutido; `--script`/`--url`+`--sha256` sobrescrevem), que baixa o `install.sh` oficial (release + checksum) e o executa com `VERSION`; `--dry-run` mostra o plano. Sem lógica de install em Rust. Fecha E18-T05.

## Impacto

- Fecha E18-T05.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
