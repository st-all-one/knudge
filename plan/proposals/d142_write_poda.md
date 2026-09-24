# D142 (proposta) — poda de `kd write`

> **Status:** decisão fechada (registrada em `plan/03_decisoes-fechadas.md`); **implementação
> pendente**. Origem: revisão de `docs/04-write.md`. Revisa **D87**. `--outcome` mantido. Sem
> execução.

## 1. Decisões confirmadas

1. **`--checks` sai de `kd write`.** É conceito de **tarefa** (D54): `resolve_checks`/
   `validators::run` **só** rodam em `kd task close` (`commands/task/mutate.rs:114`), então o
   `checks` de uma nota de conhecimento é gravado e **nunca executado**. Ação: remover de
   `write`; **documentar** em `task`; incluir `checks` no `--params`/`--batch` (D141 — já
   previsto no parser compartilhado).
2. **`confidence` sai por completo** — flag **e** chave canônica (**26 → 25**). A confiança
   **derivada** (D87) **não lê** o `confidence` declarado; o único uso é o merge manter o maior
   (`write/mod.rs:240`), que com o default `0.7` é **no-op**. Fica só a confiança derivada.
   `doctor --fix` remove a chave de notas antigas (a chave vira desconhecida → warning no read,
   D16). Revisa **D87**.

## 2. `--outcome` — mantido

O schema tem **duas** chaves distintas:

| Chave | O que é | Quem escreve |
|---|---|---|
| `outcomes` | resultado **declarado** (`success\|partial\|failure\|abandoned`) | `kd write --outcome` / `kd task close --outcome` |
| `evidence` | resultado dos **validators** (`pass\|fail\|skip` + metadados) | `kd task close` (automático) |

`--outcome` escreve `outcomes[]`. **Decisão: manter `--outcome`** — é fiel à chave `outcomes`;
`--evidence` colidiria com a chave `evidence` (validators), que é outra coisa. `--outcome`
permanece funcional em `write` (D103: vale para qualquer nota e alimenta a confiança derivada via
`confirmation`).

## 3. Redundância `write` × `task`

Não é total: o modelo é o **mesmo** (nota) e o `scope` separa — **com `scope` = trabalho; sem
`scope` = conhecimento**. A superfície é podada por eixo: `--checks` → só `task`; `confidence` →
morto nos dois; `--outcome` → ambos (D103); `--class` (shelf-life) → `write`; `--status` → ambos.

## 4. Raio de alcance (quando executar)

- `cli/mod.rs` (`WriteArgs`: remove `checks`/`confidence`), `commands/write_cmd.rs`,
  `write/draft.rs` (`checks`/`confidence`), `write/mod.rs` (merge deixa de comparar confidence),
  `schema/keys.rs` (remove `confidence` de `CANONICAL_KEYS` e `REQUIRED_KEYS`), `schema/frontmatter.rs`,
  `health/doctor/fix.rs` (strip de `confidence`), `docs/04-write.md`/`05-task.md`, `prime.rs`,
  goldens/testes.

## 5. Aceite

- [ ] `kd write --checks` / `--confidence` → exit 2 (flags inexistentes).
- [ ] `CANONICAL_KEYS` com 25; `confidence` ausente de `REQUIRED_KEYS`.
- [ ] `checks` documentado em `task` e aceito no `--params`/`--batch`.
- [ ] `doctor --fix` remove `confidence` de nota antiga, sem mudar id/`body_hash`.
- [ ] `make check` verde; docs/`prime`/goldens atualizados.
