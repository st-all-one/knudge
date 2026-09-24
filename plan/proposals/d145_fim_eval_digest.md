# D145 — fim do `maintenance eval`; `index` vira `kd knowledge digest`

> **Status:** decisão fechada e **implementada** (v0.3.0). Origem: revisão de `docs/07-manutencao.md`. Revisa **D90**.

## 1. Decisões

1. **`kd maintenance eval` sai por completo.** O comando era **stub**: imprimia `docs: N` e, com
   `--ab`, um warning ("requer um dataset; nenhum configurado"). As funções puras de métrica
   (`embeddings/eval.rs`) **só eram usadas em testes** — nada em produção as chamava (nem o
   próprio stub). Saem: o subcomando, `extra::eval`, o módulo `embeddings/eval.rs` e seus
   re-exports/testes. A avaliação de modelo continua na bancada **externa** `bench/` (foi como o
   `granite` foi escolhido — D123). Revisa **D90**.
2. **`kd maintenance index` vira `kd knowledge digest`.** No escopo de **conhecimento** e com o
   nome que descreve o que faz: **digerir o conteúdo num vetor** (384d). Mantém as ações
   `--status` (estado da fila) e `--drain` (processa um lote agora). A implementação
   (`embedder::pending`/`drain_once`) não muda.

## 2. Consequências

- `kd knowledge` passa a ter **`map`** e **`digest`**.
- `kd maintenance` fica com **`doctor`**, **`compact`**, **`learn`**, **`prune`**,
  **`watch-service`** (sem `eval`/`index`).
- Config `embeddings.*` inalterada; o auto-drain lazy (D131) segue igual.
- `--status`/`--drain` não são "varredura de corpus" — fora do princípio do escopo (D143/D144).

## 3. Raio de alcance (quando executar)

- `cli/maintenance.rs` (remove `Eval`/`Index`), `cli/knowledge.rs` (novo `Digest`),
  `commands/maintenance/mod.rs` (dispatch), `commands/maintenance/extra.rs` (remove `eval`,
  move `index`), `commands/knowledge/digest.rs` (novo) + `commands/knowledge/mod.rs`,
  `embeddings/eval.rs` (remove), `embeddings/mod.rs` (re-exports), `embeddings/tests/eval.rs`
  (remove), `docs/07-manutencao.md`, `prime.rs`, `llms.txt`, goldens/testes.

## 4. Aceite

- [x] `kd maintenance eval` → subcomando inexistente (exit 2).
- [x] `embeddings/eval.rs` removido; nenhum re-export quebrado.
- [x] `kd knowledge digest --status`/`--drain` funcionam como o antigo `index`.
- [x] `make check` verde; docs/`prime`/`llms.txt`/goldens atualizados.
