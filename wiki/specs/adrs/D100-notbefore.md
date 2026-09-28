# D100 — not_before

- **Status:** Aceita
- **Categoria:** X. Agendamento separado de expiração (D100)

## Contexto

Bloco **X. Agendamento separado de expiração (D100)**.

## Decisão

Introduz **`not_before`** (RFC3339, opcional) como **28ª chave canônica**, logo após `expires_at` e antes de `superseded_by`. Agendamento e expiração são **ortogonais** (D56): `expires_at` remove do corpus, `not_before` só **retém** a tarefa em `blocked` até o instante. A view **estática** (`compute_views`, usada pelo `prime` byte-idêntico — D57) **ignora** `not_before`; a view dinâmica (`compute_views_at(now_ms)`) o considera. Sem bump de `schema_version`: é campo opcional on-read (D15), omitido quando ausente (D05).

## Impacto

- `not_before` como 28ª chave canônica; agendamento ortogonal à expiração; views dinâmicas o consideram, o `prime` estático não.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
