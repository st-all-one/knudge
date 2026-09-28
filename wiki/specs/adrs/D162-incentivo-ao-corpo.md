# D162 — Incentivo ao corpo

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Revisa D57/D60.

## Decisão

**Incentivo ao corpo: protocolo, skill, init e doctor.** `prime` ganha seção **CORPO** com template (`Por quê:`/`Evidência:`/`Consequência:`) e regra (obrigatório quando o statement sozinho não permite agir — `decision`/`error`/`risk`); `kd init`/`onboard` cria/atualiza `.agents/skill/kd/SKILL.md` (skill otimizada para uso real, idempotente, com version marker) e referencia no `AGENTS.md`; `doctor` reporta **notas sem corpo** e **sem lastro** (corpo + `outcome` + âncora) como aviso (não derruba `healthy`); gate de corpo para `decision` fica disponível via `validators.toml` (D156), sem default. Revisa D57/D60.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
