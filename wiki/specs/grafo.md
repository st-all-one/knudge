# Grafo e inferência

O grafo é uma **projeção** das notas: a fonte da verdade continua sendo `notas/<id>.md`. Aqui
vivem arestas explícitas, integridade, ciclos, autoridade (PageRank/PPR), comunidades, ontologia
leve e TMS.

- Código: `crates/knudge-core/src/graph/`
- Decisões: D45, D46, D49–D51, D98, D177, D192, D193, D207, D208
- Fórmulas: [`matematica.md`](matematica.md) §7 (PageRank/PPR), §8 (Louvain).

## Arestas explícitas (D49/D51/D98)

- As arestas são **chaves de frontmatter de primeiro nível**, nomeadas pelo `EdgeKind`, cada uma
  uma lista de ids (omitida quando vazia): `references`, `depends_on`, `contradicts`, `supports`,
  `extends`, `replaces`, `rejects`, `results_in`, `same_as`, `broader`, `narrower`, `related`.
- `superseded_by` é o **ponteiro reverso** (id único) de `replaces`; a bidirecionalidade é
  validada pela integridade.
- A **extração textual** (`extract.rs`) é **sugestão revisável** em `.idx/suggestions.jsonl` e
  **nunca** entra no grafo (D49/D50). O `expand` percorre só o explícito.

## Integridade e ciclos (D45/D46)

- `integrity.rs` reporta referências quebradas, órfãos e a bidirecionalidade `replaces ↔
  superseded_by`.
- `cycles.rs` detecta ciclos por SCC (`strongly_connected`/`cyclic_components`) sobre `replaces`;
  membros de ciclo **não demovem** (proteção de supersessão, D45).

## Autoridade: PageRank/PPR (D192)

`rank.rs` (puro, sem dep) calcula:

- **PageRank** global e **Personalized PageRank** (PPR) semeado pelo *working set*, sobre as
  arestas de autoridade (`references`/`supports`/`extends`/`replaces`).
- Iteração de potência determinística: damping `0,85`, ≤32 iterações, tolerância L1 `1e-8`, ordem
  canônica dos ids.
- Vira o canal `ppr` da fusão RRF (`recall.ppr_weight`), **default 0,0** (desligado — só compensa
  em corpora com arestas); filtrado por `allowed`.

## Comunidades (D193)

`communities.rs` implementa **Louvain determinístico** (*local moving* + agregação; ordem canônica,
≤8 níveis, ≤64 passos, empate mantém a comunidade corrente) sobre grafo ponderado não-dirigido.
`lifecycle/communities.rs` monta o grafo a partir das arestas explícitas (peso 1) + **âncoras
compartilhadas** (clique; estrela acima de 64 membros) e produz `Community { members, terms }` com
resumo local (`content_terms`, top 8). Exposto em `kd map --communities` e materializado no `MAP.md`.

## Ontologia leve (D207)

`ontology.rs` infere sobre as arestas de ontologia:

- `equivalence_classes` (clausura de `same_as`).
- `broader_ancestors` / `narrower_descendants` (clausura de hierarquia).
- `has_hierarchy_cycle` (detecção de ciclo em `broader`/`narrower`).
- `claim_conflicts` — contradição **precisa** por claims: mesma `(subject, relation)` com objetos
  divergentes. Reportada pelo check `integrity` do `doctor`.

Tudo **derivado** — sem gravar nada, sem bump de `schema_version` além do aditivo 1→2.

## TMS / defeasible (D208)

`tms.rs` deriva, do índice reverso de `depends_on`:

- `retracted(graph)` — premissas retratadas (`forgotten`/`superseded`).
- `defeated_dependents(graph, retracted)` — dependentes **transitivos** de premissas retratadas
  (exclui pré-requisitos).
- `defeated_by_replacement(graph)` — alvos de `replaces`.

Sem apagar nada (D14): o lado substituído/contradito é **derrotado, não removido**. Entra como
motivo de demolição (`DemotionReason::Defeated`) no `maintenance prune` — off-path, só propõe.

## Estrutura do `Graph`

`Graph` mantém nós (por id) e arestas; expõe `parents` (índice reverso O(1) para
`parent`/`has_parent`, E15-T09), `is_work_item` (D120), `has_contradictions`, `belongs_to`.
`ExpandHit` representa a expansão por vizinhança.

## Onde vive

| Aspecto | Arquivo |
|---|---|
| `Graph` e expansão | `graph/mod.rs` |
| Nós/arestas | `graph/link.rs` |
| Integridade/ciclos | `graph/{integrity,cycles}.rs` |
| Sugestões | `graph/{extract,suggestions}.rs` |
| Autoridade | `graph/rank.rs` |
| Comunidades | `graph/communities.rs` |
| Ontologia | `graph/ontology.rs` |
| TMS | `graph/tms.rs` |

## Testes

`graph/tests/` (graph, integrity, cycles, extract, suggestions, rank, communities, ontology, tms) +
proptest (RRF/rank determinístico). `DIVERGENCES.md` #101/#102/#104/#110/#111.
