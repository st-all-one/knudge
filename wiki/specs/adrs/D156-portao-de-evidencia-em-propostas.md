# D156 — Portão de evidência em propostas

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**Portão de evidência em propostas.** `validators.toml` ganha `kind="gate"` (conteúdo, não config): stdin `{op,before,after}` → stdout `{passed,score_before,score_after}`. Config `proposals.gate` (nomes, vírgula), `proposals.min_delta`, `proposals.enforce`. `learn`/`compact --verify` anexam o veredito **read-only** (`gate=passed|failed` no pipe; `gate` no `--json`); com `enforce=true`, o `pre-record` roda o portão e bloqueia (`conflict`, exit 4). Gate ausente/timeout/JSON inválido degrada com aviso (R33); a decisão pura (`accept`) fica no core. Reusa D59/D99/D47.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
