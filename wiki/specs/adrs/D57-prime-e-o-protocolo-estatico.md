# D57 — prime é o protocolo estático

- **Status:** Aceita
- **Categoria:** K. Prime, protocolo e hooks

## Contexto

Bloco **K. Prime, protocolo e hooks**.

## Decisão

**`prime` é o protocolo estático** (token-optimized, byte-idêntico por versão do binário, estilo `help`): tipos, tools, regras, orçamento. `kd` sem argumentos executa `kd prime`. O **estado dinâmico** (situar agentes/rodadas) passa a ser **`kd rewind`**; o handoff 1:1 por `context_id` é `kd rewind --resume <id>`. `kd init` funda `.knudge/` + `AGENTS.md` (marcadores idempotentes). Protocolo e estado ficam **separados**.

## Impacto

- `prime` reescrito: **protocolo estático** (byte-idêntico); estado/handoff migra para **`rewind`**.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
