# 02 — Modelo de dados e contrato de bytes

O modelo é o **contrato estável** entre você e o knudge. Mudanças de bytes exigem
`schema_version`/rebuild (D15). Especificação completa: [`../specs/modelo-de-dados.md`](../specs/modelo-de-dados.md)
e [`../specs/TOON.md`](../specs/TOON.md).

## `Note`, `Frontmatter` e `Value`

```rust
use knudge_core::store::Note;
use knudge_core::schema::Value;

let note: Note = kd.store().read("fact_01m81b6h")?;
let id = note.id()?;                       // &str (histórico)
let statement = note.frontmatter.statement()?;
let tags = note.frontmatter.string_list("tags")?;
let revision = note.revision();            // contador de versões (não é CAS — D48)
let rendered = note.render();              // bytes canônicos (TOON)

// leitura tolerante: chave desconhecida vira aviso, não erro
let (note, warnings) = Note::parse_with_warnings(bytes)?;
```

- `Note { frontmatter: Frontmatter, body: String }`.
- `Frontmatter::set/get/remove/keys/validate/to_value/from_value/parse`.
- `Value` é um enum fechado: `Str | Int | Float | Bool | List | Map` (mapa preserva ordem).
- `Note::parse` **rejeita** frontmatter malformado/tipo desconhecido; `parse_with_warnings`
  devolve avisos.

## Enums fechados

| Enum | Variantes |
|---|---|
| `NoteType` | `Fact, Decision, Question, Task, Def, Error, Snippet, Link, Meta, Risk` (+ `Epic` derivado, não gravável — D149) |
| `Classification` | `Foundational, Tactical, Observational` |
| `Status` | `Active, InProgress, Blocked, Closed, Superseded, Forgotten` |
| `Scope` | `Epic, Issue, Task` |
| `EdgeKind` | 12 arestas: `References, DependsOn, Contradicts, Supports, Extends, Replaces, Rejects, ResultsIn, SameAs, Broader, Narrower, Related` |

`type` é **enum fechado**: valor desconhecido rejeita a nota na escrita e orienta na leitura
(doc 12). **Nunca** crie um tipo aberto onde o knudge fixou enum — isso quebra o contrato.

## IDs e hashes (D01/D06/D95)

```rust
use knudge_core::schema::{body, id};

// normalize = NFC + trim + colapso de espaços
let normalized = body::normalize("  Café   com leite  "); // "Café com leite"

// id endereçado por conteúdo: <prefixo>_<base36(8)>
let note_id = id::note_id(NoteType::Fact, "Café com leite");

// hash de corpo (statement + body) para frescura/rebuild
let hash = body::body_hash(&statement, &note.body);

// diretório material por tipo (D150): notas/<type_dir>/<id>.md
let dir = id::type_dir(&note_id);
```

- `id` é **histórico**: reclassificar o `type` não reescreve o id (D02).
- `schema::hash::{short_hash, hex8, base36_8}` reutilizáveis se você derivar ids próprios.
- Nunca invente id: use `id::note_id` (o id é a chave de dedup/idempotência).

## Chaves canônicas

- `CANONICAL_KEYS: [&str; 31]` fixa a **ordem** de emissão do frontmatter.
- `REQUIRED_KEYS: [&str; 5]`.
- Opcionais vazios são **omitidos** (nunca `null`).
- `SCHEMA_VERSION = 2` (claims + ontologia, D207).

Se você serializar frontmatter por conta própria, **use** `toon::emit` e `CANONICAL_KEYS`; não
implemente um emissor paralelo.

## TOON (frontmatter)

```rust
use knudge_core::toon::{self, emit, parse, split_frontmatter, detect_version};

let (front, body) = split_frontmatter(&raw)?;   // separa o bloco TOON do corpo
let (fm, warnings) = parse(&front)?;            // tolerante a chave desconhecida
let raw = emit(&fm);                            // emissão canônica (ordem fixa)
let version = detect_version(&front);           // Some(2)
```

Regras: `eol=lf` no repositório mantém `body_hash` estável entre plataformas; CRLF e Unicode são
normalizados; a emissão é **byte-exata**.

## Claims, proveniência e slots (D191/D207)

```rust
use knudge_core::schema::{Claim, Provenance, expected_slots, missing_slots};

let claim = Claim::new("cache", "usa", "LRU"); // SPO
let prov = Provenance::default();
let faltando: Vec<Slot> = missing_slots(NoteType::Decision, &note.body);
```

- **claims** (`{subject, relation, object}`) habilitam contradição precisa e ontologia.
- **provenance** (`{entity, activity, agent}`) é PROV-lite.
- **slots** mínimos por espécie são validação **soft** (aviso em `write`; erro só em `strict`).

## Recomendações

- **Trate o schema como ABI.** Se você persistir notas fora do knudge, respeite `CANONICAL_KEYS` e
  `normalize`; senão seus ids não casarão com o corpus.
- **Não armazene confiança/derivados.** Confiança é calculada (doc 09); `.idx/` é descartável.
- **Use `parse_with_warnings`** na leitura de dados do usuário; `parse` só quando você controla a
  entrada.
- **Versionamento:** para evoluir campos sem bump de schema, use `x-*`? Não — o knudge **rejeita**
  chave desconhecida na escrita (D14/D16). Prefira `meta`/`body`.
