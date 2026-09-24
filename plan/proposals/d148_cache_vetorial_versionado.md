# D148 — o cache vetorial é versionado (caminho B)

> **Status:** decisão fechada (registrada em `plan/03_decisoes-fechadas.md`); **implementação
> pendente**. Origem: revisão de `.idx/`. Revê **D15/D34/D83** ("derivado ≠ barato de
> reconstruir"). Caminho **B** e local **(ii)** confirmados. Sem execução.

## 1. Decisão

O **cache de embeddings** deixa de ser descartável e passa a ser **versionado** (opt-in), porque o
vetor é **função pura de `(modelo, body_hash)`** — determinístico e seguro de compartilhar.

- **O que versiona:** o cache (`body_hash → {vector, model, created_ms}`), **não** o índice
  `embeddings.jsonl` (que se reconstrói de notas + cache **sem chamar o modelo**).
- **Config:** `embeddings.version_cache = true` (opt-in; default `false` mantém o comportamento
  atual).
- **Merge:** `merge=union` + dedup (como os eventos, D31) — entradas são keyed por `body_hash`.
- **Sem eviction quando versionado:** LRU/TTL apagariam conteúdo versionado → desligados (ou teto
  muito alto) sob `version_cache=true`.
- **Validação:** o `model` por entrada já protege troca de modelo (o que não casa vira *miss*).
- **Tamanho:** aceito conscientemente — espaço (barato) por velocidade de dev (caro). Emite
  **aviso** acima de um teto (como `WARN_BYTES` do índice).

## 2. Onde mora (confirmado: **(ii)**)

Hoje `DERIVED_PATTERNS` (`git/exclude.rs`) exclui `/.knudge/.idx/` inteiro e `.gitattributes`
marca `/.knudge/.idx/** binary linguist-generated`. **Decisão: mover o cache para
`.knudge/emb_cache.jsonl`** (fora do `.idx/`), separando "derivado" de "versionado". O `.idx/`
segue **100% derivado** (excluído); o cache versionado fica em `.knudge/` e recebe
`merge=union eol=lf` no `.gitattributes`.

## 3. Consequências

- Clone com o mesmo modelo **não re-embeda**: reconstrói o índice de notas + cache (lookup por
  `body_hash`), zero inferência.
- Troca de modelo re-embeda só o que não casa (o índice invalida pelo `meta` — D79).
- O cache versionado cresce com o corpus (~150 MB crus / ~300–450 MB em JSONL para 100k notas) —
  trade aceito.
- Revê **D15** (derivado), **D34** (exclusão do derivado) e **D83** (cache descartável).

## 4. Raio de alcance (quando executar)

- `embeddings/cache.rs` (path/config; desligar eviction sob `version_cache`),
  `embeddings/pipeline.rs` (`cache_path`, tetos), `config` (`embeddings.version_cache`),
  `git/exclude.rs` (não excluir o cache versionado), `git/attributes.rs` (`merge=union` no cache),
  `git/onboard.rs` (layout), `docs/06-embeddings.md`, `prime.rs`, goldens/testes.

## 5. Aceite

- [ ] Com `version_cache=true`, o cache mora em `.knudge/emb_cache.jsonl`, **não** é excluído e
      recebe `merge=union` no `.gitattributes`; o `.idx/` continua excluído.
- [ ] Clone (mesmo modelo) reconstrói o índice **sem** inferência (teste com embedder fake que
      conta chamadas).
- [ ] Eviction/TTL não apagam o cache versionado.
- [ ] Troca de modelo invalida só o que não casa.
- [ ] `make check` verde; docs/`prime`/goldens atualizados.
