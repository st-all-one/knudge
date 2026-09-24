# D152 — busca sem resultado devolve `[no_results]`

> **Status:** decisão fechada e **implementada** (v0.3.0). Origem: item 6 do `rascunho_melhorias.md` — **a demanda de observabilidade é
> removida**.

## 1. Decisão

- **Não haverá** artefato de observabilidade de busca: sem `.idx/recall_stats.jsonl`, sem
  `--explain-miss`.
- **`kd ask` sem hits** → stdout **`[no_results]`** (literal fixo), em vez de vazio. No `--json`,
  `hits: []` (o envelope já comunica sucesso).
- O sentinela **não colide** com o pipe de hit (`id|statement|score|why`).

## 2. Consequências

- O agente distingue **"busca vazia"** de **erro** sem parsing ambíguo.
- Sem estado novo, sem `Dxx` de artefato; é contrato de saída → **golden**.
- Se um dia a observabilidade voltar, ela é uma decisão nova.

## 3. Raio de alcance (quando executar)

- `commands/ask/query.rs` (saída quando `hits` vazio), `docs/03-ask.md`, `prime.rs`,
  `crates/knudge-cli/tests/golden*`.

## 4. Aceite

- [x] `kd ask "termo inexistente"` → stdout exatamente `[no_results]`; exit 0.
- [x] `--json` com `hits: []`; nenhum log no stdout.
- [x] Golden novo; `make check` verde.
