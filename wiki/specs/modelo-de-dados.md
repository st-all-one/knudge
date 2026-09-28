# Modelo de dados

Como o knudge representa conhecimento: o **frontmatter canônico**, a **identidade endereçada por
conteúdo**, os **enums fechados** e as extensões semânticas (claims e proveniência). É o contrato
de bytes (D04/D06/D95) — mudar isto exige golden e, se for decisão, um `Dxx`.

- Código: `crates/knudge-core/src/schema/`
- Gramática/serialização: [`TOON.md`](TOON.md)
- Decisões: D01–D13, D95, D98, D100, D135, D142, D149, D207

## A nota

Uma nota é **frontmatter TOON + corpo Markdown**, serializada em `notas/<tipo>/<id>.md`
(D150). A verdade é o arquivo; tudo o mais (índice, grafo, derivados) é projeção.

- **Sem nota parcial** (D05): ou a nota é válida e existe, ou não existe. Campos opcionais são
  **omitidos**, nunca `null`/vazio.
- **Estrito na forma, tolerante na operação** (D05/D16/D17): chave desconhecida é rejeitada no
  write e vira *warning* no read; tipo desconhecido **rejeita a nota** (por-nota, sem derrubar a
  leitura inteira).

## Identidade endereçada por conteúdo

| Elemento | Fórmula | Decisão |
|---|---|---|
| `normalize(s)` | NFC + trim + colapso de whitespace | D06/D95 |
| `body_hash` | `hex8(normalize(statement) + LF + normalize(body))` | D06 |
| `id` | `<prefixo>_<base36(8)>(type + U+001F + normalize(statement))` | D01–D03 |
| hash curto | SHA-256 truncado aos **4 primeiros bytes** (`u32` big-endian) | D95 |

Consequências:

- **Idempotência sob retry** (D01): o mesmo conteúdo gera o mesmo `id`; reescrever não duplica.
- **Prefixo histórico** (D02): reclassificar o `type` **não** reescreve o `id` — a linhagem fica
  explícita via `superseded_by`.
- **`body_hash`** cobre `statement` + corpo, então uma edição de corpo muda o hash (e invalida
  vetores/índice).
- O `id` é **NFC** (D95); o fold de acentos (D172) e o stemming (D206) são só do índice derivado —
  `notas/` nunca muda.

## Chaves canônicas

`CANONICAL_KEYS` tem **31** chaves, nesta ordem (D04/D13/D98/D100/D135/D142/D207):

```
id · type · statement · created_at · body_hash · schema_version ·
tags · source · superseded_by ·
references · depends_on · contradicts · supports · extends · replaces · rejects · results_in ·
same_as · broader · narrower · related ·
revision · outcomes · classification · anchors · status · scope · checks · evidence ·
claims · provenance
```

- `REQUIRED_KEYS` (5): `id`, `statement`, `created_at`, `body_hash`, `schema_version`. O `type` é
  obrigatório para espécies mas **omitido** quando `scope=epic` (o tipo efetivo é derivado — D149).
- As **12 arestas** ficam logo após `superseded_by`; `superseded_by` é o ponteiro reverso (id único)
  de `replaces` (D98).
- `expires_at`/`not_before` **saíram** (D135): a expiração é derivada da `classification` e tarefa
  não tem tempo. `confidence` saiu (D142): é sempre derivada (D87).

## Enums fechados

Tipos abertos degradam o retrieval (o LLM inventa categorias), por isso são fechados e evoluir
exige bump de `schema_version` (D14).

| Enum | Valores | Nota |
|---|---|---|
| `NoteType` | `fact`, `decision`, `question`, `task`, `def`, `error`, `snippet`, `link`, `meta`, `risk` (**10 armazenáveis**) | `Epic` é **derivado** de `scope=epic`, nunca gravado (D149). |
| `Scope` | `epic`, `issue`, `task` | nível do item de trabalho (D134). |
| `Classification` | `foundational`, `tactical`, `observational` | rege shelf-life (D44). |
| `Status` | `active`, `in_progress`, `blocked`, `closed`, `superseded`, `forgotten` | `VISIBLE` = `active`/`in_progress`/`blocked`/`closed` (D176). |
| `EdgeKind` | `references`, `depends_on`, `contradicts`, `supports`, `extends`, `replaces`, `rejects`, `results_in`, `same_as`, `broader`, `narrower`, `related` (**12**) | `is_ontology`/`is_symmetric`/`inverse`/`is_supersession` (D207). |

## Extensões semânticas (D207)

- **Ontologia leve (SKOS-lite):** `same_as`/`broader`/`narrower`/`related`. `broader`↔`narrower`
  são inversos; `same_as`/`related` são simétricos. Inferência derivada em `graph/ontology.rs`
  (classes de equivalência, clausura, ciclo).
- **`claims`:** lista de mapas `{subject, relation, object}` (SPO), strings trimadas, ≤120
  escalares, relação de mundo aberto. Permite contradição **precisa** (`claim_conflicts`).
- **`provenance`:** mapa PROV-lite `{entity, activity, agent}`.

Ambos são **aditivos** (`schema_version` 1→2); notas v1 seguem válidas e o rebuild é
byte-idêntico.

## `outcomes`, `anchors`, `checks`, `evidence`

- **`outcomes[]`** (D48/D103): evidência de execução (`status`/`duration`/`agent`/`notes`/
  `recorded_at`) para **qualquer** nota. A confirmação é **derivada** (D87/D189).
- **`anchors`** (D86/D135): único link externo canônico (`path`); o `content_hash` é **derivado**
  em `.idx/anchors.jsonl`, nunca no frontmatter.
- **`checks`** (D54/D99): conceito de **tarefa** — o catálogo de validators (`.knudge/validators.toml`).
- **`evidence`** (D55): fechamento por evidência de tarefa.

## Data contract por tipo (soft, D191)

O `write` e o `doctor` conferem **slots mínimos de corpo** por espécie (ex.: `decision`→
Alternativas/Por quê/Consequência; `error`→Causa/Correção; `risk`→Probabilidade/Impacto),
casados por cabeçalho/rótulo com fold de diacríticos. **Sem chave nova e sem bump de
`schema_version`**: é aviso em `write` (`warnings[]`) e `invalid_input` só sob `behavior.strict`.

## Validação

`Frontmatter::validate()` (`schema/frontmatter.rs`) checa chaves conhecidas, tipos, limites
(`statement ≤ 120` escalares Unicode — D08) e a forma de `claims`/`provenance`. O `doctor` usa o
mesmo núcleo via o check `integrity`/`body`.

## Contrato de bytes (resumo)

- Ordem canônica sempre (nota nova insere na ordem — D13); opcionais omitidos, nunca `null`.
- Inteiros normalizados na decodificação; nunca emitir `.0` (D09).
- Raw UTF-8; escapar só `"`, `\` e controles (D10). Lista vazia → arquivo zero bytes (D11).
- `SCHEMA_VERSION = 2` (D207).

## Onde vive

| Aspecto | Arquivo |
|---|---|
| Chaves e ordem | `schema/keys.rs` |
| Enums | `schema/types.rs` |
| Arestas | `schema/edge.rs` |
| Frontmatter + `validate` | `schema/frontmatter.rs` |
| Normalização/hash | `schema/body.rs`, `schema/hash.rs` |
| ID | `schema/id.rs` |
| Claims/proveniência | `schema/claims.rs`, `schema/provenance.rs` |
| Slots | `schema/slots/` |
| Estatísticas de outcomes | `schema/outcomes.rs` |

## Testes

`schema/tests.rs` e `schema/tests/semantic.rs`; proptest em normalize/IDs; golden de bytes em
`crates/knudge-cli/tests/golden/`. `DIVERGENCES.md` #96 (fold), #110 (claims/ontologia).
