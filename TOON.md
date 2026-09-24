# TOON — contrato de bytes

> Subconjunto **documentado** de TOON usado no frontmatter das notas (D74/D75). O parser/emissor
> vive em `crates/knudge-core/src/toon/`. O knudge é *byte-sensitive*: estas regras são
> **contrato**, travadas por golden e proptest (E02).

## 1. Princípios

- **Raw UTF-8.** Sem BOM. Sem normalização implícita de conteúdo (a normalização é explícita em
  `normalize`, D06).
- **Canônico na emissão, tolerante na leitura** (Postel): o emissor tem uma única forma; o
  parser aceita variações seguras (espaços, comentários, aspas desnecessárias).
- **Nada de `null`.** Campos opcionais ausentes são **omitidos** (D05); `null`/`~` são strings
  comuns.
- **Ordem é contrato** (D04/D13): `Frontmatter::to_value` sempre emite na ordem canônica,
  independentemente da ordem de inserção.

## 2. Gramática

```
document      = { entry } ;
entry         = key ":" [ " " value ] newline
              | key ":" newline block ;
block         = { indented_entry | indented_item } ;
indented_entry= indent key ":" [ " " value ] newline ;
indented_item = indent "- " ( value | pair { continuation } ) newline ;
continuation  = indent2 key ":" " " value newline ;
value         = scalar | flow_list | flow_map ;
flow_list     = "[" [ value { "," value } ] "]" ;
flow_map      = "{" [ pair { "," pair } ] "}" ;
pair          = key ":" " " value ;
scalar        = quoted | bare ;
quoted        = '"' { char | escape } '"' ;
escape        = "\" ( '"' | "\" | "n" | "r" | "t" | "0" | "u{" hex "}" ) ;
key           = ( letter | "_" | "-" ) { letter | digit | "_" | "-" } ;
indent        = 2 * " " ;   (* múltiplos de 2; tab é erro *)
```

- Comentário: `#` **fora de aspas**, no começo da linha ou precedido de espaço, até o fim da
  linha. O emissor **nunca** emite comentário.
- Linhas em branco são ignoradas.
- Chave duplicada é **erro**.
- Aninhamento máximo: 32 níveis (proteção contra input hostil).

## 3. Escalares

| entrada | leitura |
|---|---|
| `true` / `false` | `Bool` |
| dígitos (com `-`/`+` opcional) | `Int(i64)` |
| contém `.`/`e`/`E` e parseia finito | `Float(f64)` |
| `"..."` | `Str` (com escapes) |
| qualquer outro | `Str` cru |

Regras de bytes:

- **Inteiros nunca saem como `.0`** (D09): `1.0` é emitido `1` e re-lido como `Int(1)`.
- Aspas só quando necessário. Um `Str` é emitido **cru** se não for vazio, não tiver espaço nas
  pontas, não começar com `- `, não for `true`/`false`, não parecer número e não contiver
  `" \ # , [ ] { }` nem controles. `:` é permitido cru em valores (o parser corta só no primeiro
  `:` de uma entrada).
- Controles viram `\n`, `\r`, `\t`, `\0` ou `\u{XXXX}`.

## 4. Frontmatter e corpo

Formato do arquivo de nota:

```
---
<frontmatter TOON>
---
<corpo>
```

- `split_frontmatter` separa as partes; sem `---` inicial, o frontmatter é vazio.
- **Lista/mapa vazio é omitido** no frontmatter (D05). Documento vazio → **zero bytes** (D11).
- Newline final é normalizado para `LF` (D12).

## 5. Ordem canônica (D04/D13)

```
id, type, statement, created_at, body_hash, schema_version,
tags, source, superseded_by,
references, depends_on, contradicts, supports, extends, replaces, rejects, results_in,
revision, outcomes, classification, anchors, status, scope, checks, evidence
```

As 8 chaves de aresta ficam entre `superseded_by` e `revision` (D98) — **25 chaves** ao todo
(D135 removeu `expires_at`/`not_before`; D142 removeu `confidence`).

`type` é enum fechado de 10 espécies (o `epic` é derivado de `scope=epic` — D149) (ver `00_panorama.md` §4); `scope`, `classification` e
`status` também são fechados. Chave fora dessa lista é **rejeitada no write** e **ignorada com
warning no read** (D16).

## 6. Versionamento e rebuild (D15)

- `schema_version` (atual `1`) é lido **on-read com defaults**; sem aliases (D14).
- Tipo desconhecido: **rejeita a nota** com erro explícito, sem derrubar a leitura (D17/D18).
- O **índice derivado** (`.idx/`) é reconstruível a partir das notas canônicas. Rebuild é
  disparado quando o **formato do índice** muda, nunca quando o frontmatter muda.

## 7. Hash (D95)

- Hash curto = **SHA-256 truncado aos 4 primeiros bytes** (`u32` big-endian).
- `body_hash = hex8(SHA-256(normalize(statement) + LF + normalize(body)))` (D06).
- `id = <prefixo>_<base36(8)>(SHA-256(type + U+001F + normalize(statement)))` (D01/D02).
- `normalize` = **NFC + trim + colapso de whitespace**.

## 8. Corpus / golden

- `crates/knudge-core/src/toon/tests.rs` — round-trip byte-exato do corpus canônico, goldens de
  escapes, zero bytes, `1.0 → 1`, comentários e indentação.
- Proptests: `emit(parse(x)) == x` para mapas/listas de escalares; idempotência de `normalize`.
