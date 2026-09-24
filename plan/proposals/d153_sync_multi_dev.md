# D153 — sincronização multi-dev (notas = verdade; derivado = reconstruir/union)

> **Status:** decisão fechada e **implementada** (v0.3.0). Origem: revisão do uso concorrente (2–3 devs no mesmo projeto). Complementa
> **D26/D28/D31** (eventos `merge=union`), **D148** (cache vetorial versionado) e **D150**
> (layout). **Premissa operacional:** os devs usam o **mesmo modelo e as mesmas configurações**
> de embeddings; o filtro por `model` é rede de segurança, não o caminho principal.

## 1. Decisão

**Princípio:** a **verdade são as notas** (endereçadas por conteúdo, um arquivo por nota →
mergeáveis pelo git). O **índice é derivado** (reconstrói); o **cache vetorial e os eventos são
endereçados por conteúdo** (união + dedup). **Nunca "resolver" um conflito num artefato
derivado — re-derive.**

1. **Notas — merge natural.** `id = hash(type + U+001F + normalize(statement))`: conhecimentos
   diferentes → ids/arquivos diferentes → o git faz merge sozinho. O **único** conflito real é a
   **mesma `statement` com corpos/estados divergentes** (mesmo id = mesmo arquivo). Não é
   auto-mergeado: o arquivo fica com marcadores de conflito, o TOON não parseia, a leitura
   tolerante **pula a nota** e o `doctor` (check `schema`) a lista — o agente resolve/supersede
   (D47).
2. **Índice (`.idx/`) — derivado.** Excluído do git (`DERIVED_PATTERNS`); cada clone reconstrói
   após o `pull`. Sem merge, sem conflito.
3. **Eventos — `merge=union`** (já em `.gitattributes`, D31): append-only, `id` de conteúdo.
4. **Cache vetorial (D148) — `merge=union` + dedup.** Entradas keyed por `body_hash` com `model`
   por entrada; a **chave lógica é `(body_hash, model)`**. O loader é **idempotente** (dedup por
   chave) e **model-aware** (seleciona a entrada do modelo ativo; entrada de outro modelo = miss).
5. **Desempate determinístico.** Se a mesma `(body_hash, model)` aparecer com vetores diferentes
   (provedor não bit-exato), escolher por regra **estável** — `created_ms` maior; empate → o vetor
   lexicograficamente maior — nunca "o primeiro do arquivo".
6. **Sem lock distribuído nem CRDT.** O lock é advisory **local** (mesma máquina); entre clones o
   único sincronizador é o **git** (`kd sync` → commit de `notas/` + `eventos/` + cache).
7. **Premissa de operação.** Mesmo modelo + configs entre devs; o filtro por `model` cobre a troca
   acidental.

## 2. Consequências

- Clone/`pull` com o mesmo modelo: índice reconstruído de **notas + cache**, zero inferência.
- Conflito de nota = conflito de arquivo **legítimo** (mesma afirmação, corpos divergentes);
  proposta, nunca resolução silenciosa.
- `.gitattributes` ganha a linha do cache versionado; `emb_cache.jsonl` **não** entra em
  `DERIVED_PATTERNS`.

## 3. Raio de alcance (quando executar)

- `git/attributes.rs` (`/.knudge/emb_cache.jsonl text eol=lf merge=union`),
  `git/exclude.rs` (garantir que o cache **não** é excluído quando versionado),
  `embeddings/cache.rs` (dedup por chave + seleção por `model` + desempate),
  `embeddings/pipeline.rs`, `docs/06-embeddings.md`, `DIVERGENCES.md` (nova borda + teste),
  `plan/00_panorama.md`, e um **teste multi-clone** (dois `MemFs`/stores → union → dedup).

## 4. Aceite

- [x] `emb_cache.jsonl` recebe `merge=union`; `.idx/` segue excluído.
- [x] Loader dedupa por `(body_hash, model)` e ignora entradas de modelo inativo.
- [x] Mesma chave com vetores diferentes resolve por `created_ms` (determinístico, testado).
- [x] Nota com marcadores de conflito é pulada e reportada pelo `doctor`.
- [x] `make check` verde; docs/`DIVERGENCES.md` atualizados.
