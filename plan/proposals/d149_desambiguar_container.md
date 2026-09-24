# D149 — fim do `type=container`; o grupo é derivado de `scope=epic`

> **Status:** decisão fechada (registrada em `plan/03_decisoes-fechadas.md`); **implementação
> pendente**. Origem: "container" significava **três coisas**. Revisa **D93/D113/D134**. Sem
> execução.

## 1. Decisão

- **Remove `type=container`** do enum: **11 → 10 tipos** (ficam `fact`, `decision`, `question`,
  `task`, `def`, `error`, `snippet`, `link`, `meta`, `risk`).
- **Um grupo é uma nota com `scope=epic`** — o `type` é **omitido** e **derivado** (efetivo
  `epic`, prefixo do `id`). Não há mais um `type` de agregação.
- **O filtro/eixo `container` vira `scope`** (`--container` → `--scope`, D146; eixo do
  `knowledge map` → `scope`; `container_of` → `scope_of`).

Resultado: **"container" desaparece**. O grupo é **derivado** de `scope=epic`; `scope` é o
nível/filtro; a agregação é o conceito (`results_in`).

## 2. Consequências

- `type` deixa de ser obrigatório quando `scope=epic`; o **tipo efetivo** é `epic` e o `id` usa
  esse prefixo (`epic_<base36>`). Ids históricos `container_*` permanecem (prefixo é histórico —
  D95).
- `is_container`/`is_group` = `scope == epic` (não mais `type`).
- `Graph::is_work_item` **exclui** `scope=epic` (não tem `type` de espécie).
- `ask --type` deixa de aceitar `container`; para grupos, usa-se `--scope epic`.
- Revê **D93** (`plan`/`epic` = `container`), **D113** (`scope` nível × `type` espécie) e **D134**.

## 3. Raio de alcance (quando executar)

- `schema/types.rs` (`NoteType` sem `Container`; prefixo efetivo `epic`), `schema/frontmatter.rs`
  (`type` opcional sob `scope=epic`), `schema/keys.rs` (`REQUIRED_KEYS` condicional),
  `graph/mod.rs` (`is_work_item`), `task/{spec,role,progress,hierarchy}.rs`,
  `lifecycle/clusters.rs` (`scope_of`), `retrieval/filter.rs`, `cli/*` (`--scope`),
  `commands/knowledge/map.rs` (eixo), docs, `prime.rs`, goldens/testes.

## 4. Aceite

- [ ] `type=container` não existe (write rejeita; read tolerante orienta).
- [ ] Um épico é `scope=epic` **sem `type`**; `id` com prefixo `epic_`.
- [ ] `--scope`/eixo `scope` substituem `--container`/eixo `container`.
- [ ] `is_work_item` não conta épicos; `make check` verde; docs/`prime`/goldens atualizados.
