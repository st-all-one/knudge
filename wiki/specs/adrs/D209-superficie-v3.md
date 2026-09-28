# D209 — Superfície v3

- **Status:** Aceita
- **Categoria:** 0.5.0 (E16–E19)

## Contexto

Bloco **0.5.0 (E16–E19)**. Linhagem: Revisa D128/D145/D146/D157/D158/D170.

## Decisão

**Superfície v3: fim do verbo `knowledge`** (fecha E19-T12; Q5 da `revisao_integrada.md`). Um vocabulário por conceito, **sem retrocompatibilidade** (D14): `kd knowledge rank` → **`kd ask --rank`**; `kd knowledge tags` → **`kd ask --tags`**; `kd knowledge suggest` → **`kd ask --suggest`**; `kd knowledge map` → verbo de topo **`kd map`**; `kd knowledge promote` → **`kd config promote`**. Os modos de `ask` reusam os filtros de corpus (`--type`/`--class`/`--tag`/`--anchor`/`--around`/`--depth`/`--limit`) e acrescentam `--rank`/`--tags`/`--suggest`/`--universe`/`--top-k`/`--relation`; `--params` aceita `rank`/`tags_vocab`/`suggest`/`universe`/`top_k`/`relation`. `forget` **permanece verbo** (é a aplicação do `prune`, D112); `self` segue enxuto (E18/D187). Verbos de domínio caem de 9 para **8** (`ask`/`write`/`task`/`rewind`/`map`/`doctor`/`drain`/`forget`), com `init`/`prime`/`sync`/`config`/`self` como fundação/meta. Revisa D128/D145/D146/D157/D158/D170. `DIVERGENCES.md` #112.

## Impacto

- Borda registrada em `wiki/specs/DIVERGENCES.md` #112.
- Fecha E19-T12.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
