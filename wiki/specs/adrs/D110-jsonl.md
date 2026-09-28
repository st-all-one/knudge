# D110 — JSONL

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

`kd write --batch -` aplica um lote de rascunhos **JSONL** pelo mesmo dedup (0.75/0.92); item inválido ⇒ `warnings[]` (R33); `--dry-run` só avalia; teto `write.batch_max` (int, 100).

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
