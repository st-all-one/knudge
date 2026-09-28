# D135 — --anchor é o único link externo; fim de --source, --expires-at e

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Revisa D44/D56/D57/D100/D119.

## Decisão

**`--anchor` é o único link externo; fim de `--source`, `--expires-at` e `not_before`.** A âncora é a única conexão canônica entre o `.knudge/` e arquivos externos (conhecimento e tarefa); o flag `--source` sai de `kd write`/`kd task new` e o fallback `source` de `program_of` (D119) sai junto. `expires_at` e `not_before` são **removidos por completo** (flags e as duas chaves canônicas — `CANONICAL_KEYS` 28 → 26): a expiração passa a ser sempre derivada da `classification` (D44) e **tarefa não tem tempo** — só `created_at` — existindo como ação em aberto; some `BlockReason::Scheduled` e as views estática/dinâmica (`compute_views_at`) colapsam. Revisa D44/D56/D57/D100/D119.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
