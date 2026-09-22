# E02 — Contrato de bytes: TOON, schema e IDs

> **Fase 0.** O maior buraco apontado em `01_gaps-plan-rs.md` §2.1: ordem de chaves, `null` vs
> ausente, zero bytes, newline, hash e contagem. Como o knudge é *byte-sensitive* (frontmatter,
> hashes, índice derivado), cada regra precisa ser decidida **e travada por golden**.
>
> **Decisões:** D01, D02, D03, D04, D05, D06, D07, D08, D09, D10, D11, D12, D13, D14, D15, D74, D75, D95.
> **Políticas:** R02, R04 (ver [`14_revisao_tecnica.md`](14_revisao_tecnica.md)).

## Objetivo do épico

Implementar o **schema canônico em código**, o **parser/emitter TOON próprio** e os
**normalizadores** (Unicode, timestamp, números) e o **gerador de IDs endereçados por
conteúdo** — com round-trip byte-exato garantido por corpus/golden.

## Pré-requisitos

E01.

## Tarefas

### E02-T01 ☑ Schema canônico em código
- **Objetivo:** structs com **ordem de declaração canônica** (preservada, ex.: `IndexMap`),
  enum fechado de `type`, campo obrigatório/opcional, `schema_version`.
- **Entregáveis:** `core/schema`; mapas tipo→prefixo; docs de cada chave (tabela de
  `00_panorama.md` §4).
- **Decisões:** D04, D05, D13.
- **Aceite:** round-trip preserva a ordem; nota nova insere na ordem canônica; chave
  desconhecida não tem representação.

### E02-T02 ☑ Parser/emitter TOON próprio
- **Objetivo:** subconjunto documentado de TOON; **raw UTF-8**; escapar apenas `"`, `\` e
  controles; inteiros normalizados (nunca `.0`); **lista vazia → arquivo zero bytes**;
  newline final normalizado; fallback e detecção de versão de frontmatter.
- **Entregáveis:** parser/emitter; `TOON.md` (gramática); corpus + goldens.
- **Decisões:** D09, D10, D11, D12, D74, D75.
- **Aceite:** round-trip **byte-exato** no corpus; proptest de idempotência; golden de zero
  bytes e de escapes.

### E02-T03 ☑ Normalização e hash do corpo
- **Objetivo:** `normalize(body)` = **NFC + trim + colapso de whitespace**; `body_hash` =
  hash de **`statement` + corpo**, hex8 — congelado por teste.
- **Entregáveis:** função pura `normalize`; `body_hash`; doc "o que entra no hash".
- **Decisões:** D06.
- **Aceite:** variações que devem colidir colidem; editar o `statement` muda o hash; golden.

### E02-T04 ☑ Timestamps
- **Objetivo:** `created_at`/datas em **UTC com milissegundos e sufixo `Z`**; parser/emitter
  determinísticos.
- **Entregáveis:** tipo de tempo no core + serialização.
- **Decisões:** D07.
- **Aceite:** round-trip e ordenação lexicográfica == cronológica; golden.

### E02-T05 ☑ Contagem de `statement ≤ 120`
- **Objetivo:** contar **escalares Unicode** (não bytes, não UTF-16, não grafemas).
- **Entregáveis:** validador de tamanho.
- **Decisões:** D08.
- **Aceite:** casos com emoji/CJK/combining marcam o limite corretamente; golden.

### E02-T06 ☑ IDs endereçados por conteúdo
- **Objetivo:** `<prefixo>_<base36(8)>` derivado de `hash(type + statement)`; prefixo por
  tipo; **mesmo conteúdo → mesmo id** (write idempotente); colisão detectada; **reclassificar
  tipo não reescreve o id**.
- **Entregáveis:** gerador de ID; validador de formato; tabela tipo→prefixo.
- **Decisões:** D01, D02, D03.
- **Aceite:** idempotência (mesma entrada → mesmo id); prefixo correto por tipo; goldens de
  colisão e de reclassificação.

### E02-T07 ☑ Versionamento e rebuild
- **Objetivo:** política de `schema_version` **on-read com defaults**, rebuild do índice só
  quando o **formato do índice** muda; **sem aliases e sem retrocompatibilidade** (greenfield).
- **Entregáveis:** checagem de versão; procedimento de rebuild documentado.
- **Decisões:** D14, D15.
- **Aceite:** nota com `schema_version` atual lê com defaults; mudança de formato do índice
  dispara rebuild documentado.

## Definition of Done

- [x] Round-trip é byte-exato no corpus TOON.
- [x] `normalize`/`body_hash`/timestamp/contagem travados por golden.
- [x] IDs idempotentes e prefixados.
- [x] `TOON.md` e a tabela de schema publicadas.

## Não-objetivos

- Persistência de notas (E03) e índice (E06).
- Aliases de campo (recusados por D14).
