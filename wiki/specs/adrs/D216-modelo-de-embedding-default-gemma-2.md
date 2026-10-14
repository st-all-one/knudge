# D216 — Modelo de embedding default passa a embeddinggemma-2

- **Status:** Aceita
- **Categoria:** Embeddings (D79–D101)

## Contexto

Bloco **Embeddings via HTTP local (D101)**. O default era `ibm-granite/granite-embedding-97m-multilingual-r2`
(384d, D123). A bateria A/B sobre o corpus real (`bench/ab-embeddinggemma2/RELATORIO.md`, 1241 notas de
`_REF/.knudge`, 5 famílias de consulta, IC95% bootstrap pareado) mostrou que `google/embeddinggemma-2`
supera o granite em todas as famílias, especialmente com os prefixos de instrução de tarefa que o
modelo exige.

## Decisão

**Modelo de embedding default passa a `google/embeddinggemma-2` (768d) com prefixos de tarefa.**
`embeddings.model` → `google/embeddinggemma-2`; `embeddings.dimensions` → **768**; nova chave
`embeddings.prefix` ∈ `auto` (default) | `none` | `embeddinggemma`. `auto` deriva do nome do modelo:
`embeddinggemma` ⇒ prefixos assimétricos (documento `title: {stmt} | text: {body}`; consulta
`task: search result | query: {q}`); qualquer outro modelo ⇒ `none` (granite segue **sem** prefixo).
O prefixo **resolvido** entra na identidade (`EmbeddingMeta`, `fingerprint`, `cache_key`) e no
cabeçalho do `.idx/embeddings.jsonl` (cabeçalhos antigos sem `prefix` leem `none`), então trocá-lo —
assim como modelo/dimensão — **invalida** o índice (D79). Corrige a ausência de validação de
dimensão no adaptador HTTP (`parse_vectors`): vetor de tamanho ≠ `embeddings.dimensions` falha
explícita (R33), em vez de degradar a similaridade em silêncio. Migração do corpus:
`kd drain --digest --force` (D170/D215).

Medição (R@1 do canal semântico, `self` / `kw`): granite 384d **85,3 % / 75,3 %**; gemma-2 768d
sem prefixo 92,2 % / 86,1 %; gemma-2 768d com prefixo `title` **96,1 % / 92,9 %**; gemma-2 384d
(MRL) com prefixo 95,6 % / 92,8 % — truncar para 384 é praticamente *lossless*. O ganho do prefixo
é quase tão grande quanto a troca de modelo.

## Impacto

- `config/schema/keys_embeddings.rs`: `model`/`dimensions` default e a chave `prefix`.
- `embeddings/prefix.rs` (novo): enum `Prefix` + texto de documento/consulta.
- `embeddings/meta.rs`: `Prefix` na identidade, `fingerprint`, `cache_key`, cabeçalho JSON.
- `embeddings/pipeline.rs` + `commands/ask/query.rs`: aplicam prefixo no documento e na consulta.
- `adapters/http.rs`: valida a dimensão do vetor devolvido.
- `scripts/knudge-idle.sh`: GGUF/revisão/SHA do gemma-2.
- `wiki/specs/DIVERGENCES.md` #117.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
