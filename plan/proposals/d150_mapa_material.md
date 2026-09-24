# D150 — o mapa de conhecimento material (versionado)

> **Status:** decisão fechada e **implementada** (v0.3.0). Origem: `TMP/` (526 notas planas em
> ~30 min) + `knowledge map` transitório. Abordagens **A+B+D**, **versionado**.

## 1. O problema (evidência do `TMP`)

526 notas planas em `notas/`, nome `id` opaco: 262 `task`, 80 `fact`, 76 `error`, 59 `container`,
31 `def`, 12 `decision`, 5 `link`, 1 `question`. A estrutura **existe** (`scope`, `results_in`,
marcador de pai, tags, âncoras) mas **não está materializada**; o humano não tem ponto de entrada.

**Fato que ajuda:** o `id` carrega o tipo (`task_`, `decision_`…), então `notas/<tipo>/<id>.md` é
**derivável do id** — sem ler a nota.

## 2. Decisão: A + B + D, tudo versionado

- **B — pastas por tipo.** `notas/<tipo>/<id>.md`; o path é derivado do prefixo do `id` (grátis).
  Grupos (pós-D149) ficam em `notas/epic/`. Reduz 526 arquivos para ~10 pastas.
- **A — `MAP.md` centralizador** na raiz de `notas/`: árvore de grupos + clusters (com
  `--semantic`), links e contagens. Materializa o **mapa** como documento.
- **D — MOC/hub notes:** uma **nota real** por grupo/cluster que `references` os membros
  (versionada, buscável, entra no `ask`). O `MAP.md` aponta para os hubs.
- **Versionado** (não derivado): `notas/`, `MAP.md` e os hubs vão para o git (D148 já aceita
  espaço por velocidade). O churn do `MAP.md`/hubs é aceito.

## 3. Consequências

- O humano tem **ponto de entrada** (pastas + `MAP.md` + hubs); a IA segue lendo notas.
- `note_path` passa a `notas/<tipo>/<id>.md` (derivável do id). Rebuild/purge/`list_ids` adaptam-se.
- Hub notes são `references` (aresta existente) — **sem chave nova**; o mapa vira grafo real.
- Revê o layout do store (`store/mod.rs`) e o `onboard` (`LAYOUT_DIRS`).

## 4. Pendências de implementação

| # | Questão | Recomendação |
|---|---|---|
| O2 | Quem gera: `kd knowledge map --write`? | sim (a view já existe; materializa `MAP.md` + hubs) |
| O3 | Hubs cobrem clusters semânticos (fase 2)? | sim, com `--semantic` |
| O4 | Hub é `type=meta` ou um `type` novo? | `meta` (conhecimento sobre o sistema), sem tipo novo |

## 5. Aceite

- [ ] `notas/<tipo>/<id>.md`; `note_path` derivável do id; rebuild/purge funcionam.
- [ ] `kd knowledge map --write` gera `MAP.md` + hub notes (`references`).
- [ ] `MAP.md`/hubs versionados; `ask` acha os hubs.
- [ ] `make check` verde; docs/`prime`/goldens atualizados.
