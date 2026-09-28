# D157 — Promoção de conhecimento a regras governadas no AGENTS

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**.

## Decisão

**Promoção de conhecimento a regras governadas no `AGENTS.md`.** Bloco irmão `<!-- knudge:rules:start/end -->` (não é apagado pelo `init`/`onboard`, que reescreve o protocolo D60), com proveniência por linha (`- [id] statement`). `kd knowledge promote recommend|approve|edit|remove|list`; elegíveis `type=meta|decision`, `classification=foundational`, confiança derivada (D87) ≥ `rules.min_confidence` e sem `contradicts` aberto. Teto rígido `rules.max_promoted` (admission control; sem espaço recusa e nomeia quem sai). **Desligado por default** (`rules.enabled=false`); nunca auto-edita; a nota de origem permanece. Reusa D47/D60.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
