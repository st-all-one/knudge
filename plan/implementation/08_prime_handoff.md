# E08 — Prime, handoff, diff e learn

> **MVP.** O `rewind` é o **handoff de estado entre agentes/rodadas**. Aqui entram a família
> `rewind`, o **orçamento de tokens sem tokenizer**, o auto-context-scope, o `context_id`
> endereçável, e as operações de passado (`diff`) e sugestão (`learn`), além de
> `task`/`compact`.
>
> **Decisões:** D33, D40, D41, D47, D52, D53, D57, D58, D82, D88.
> **Políticas:** R04, R15, R33 (ver [`14_revisao_tecnica.md`](14_revisao_tecnica.md)).

## Objetivo do épico

Um agente novo (ou a próxima rodada) **situa-se em ~30 tokens** e pode retomar o contexto
exato anterior sem re-busca; o sistema sugere consolidação e planos como **views**, não como
verdade materializada.

## Pré-requisitos

E06, E07.

## Tarefas

### E08-T01 ☑ Família `rewind`
- **Objetivo:** `rewind` → manifest ~30 tokens; `rewind --scope` → container/domínio;
  `rewind --files` → working set ancorado; ranking por trust-tier
  (`star*100 + foundational*50 + tactical*20 + observational*10`).
- **Entregáveis:** três modos; ranking por tier.
- **Decisões:** D57.
- **Aceite:** tamanho do manifest; ranking determinístico por tier.

### E08-T02 ☑ Orçamento de tokens sem tokenizer
- **Objetivo:** `estimateTokens = ceil(len/4)` (D40), default **4000**; prioridade
  tipo → classificação → score → timestamp; **trunca o último item** e ignora sobra < 100
  tokens. Sem dependência de tokenizador (D82).
- **Entregáveis:** `rewind --budget`; aplicador de orçamento.
- **Decisões:** D40, D82.
- **Aceite:** orçamento nunca estourado; golden de truncamento e de sobra mínima.

### E08-T03 ☑ Auto-context-scope e auto-flip
- **Objetivo:** derivar o escopo de `git status -uall` + active work; **flip** para manifest
  quando `>100 notas` ou `>5 containers`.
- **Entregáveis:** detector de escopo; heurística de flip.
- **Decisões:** D41.
- **Aceite:** cenário sintético dispara o flip; escopo coerente com o diff.

### E08-T04 ☑ `context_id` e handoff 1:1
- **Objetivo:** o `rewind` emite um **`context_id`** endereçável; `rewind --resume <id>` devolve
  o **mesmo contexto**, sem re-busca (anti-drift); footer curto de session-close.
- **Entregáveis:** geração/armazenamento do `context_id` (derivado); `rewind --resume`.
- **Decisões:** D58, D88.
- **Aceite:** mesmo `context_id` → bytes idênticos; handoff entre rodadas reproduzível.

### E08-T05 ☑ `diff`
- **Objetivo:** `diff(since, until, scope)` sobre eventos.
- **Entregáveis:** `diff()`.
- **Decisões:** D21 (eventos).
- **Aceite:** diff determinístico por intervalo; sem depender do git global.

### E08-T06 ☑ `learn`
- **Objetivo:** sugerir notas a partir de **eventos + `anchors`** (determinístico), não do
  diff global. Exposto como `kd maintenance learn` (ver `16_cli_surface.md` §9).
- **Entregáveis:** `learn`; sugestões revisáveis.
- **Decisões:** D33.
- **Aceite:** sugestões derivam só de eventos/âncoras; nada é escrito sem confirmação.

### E08-T07 ☑ `kd task` e container como view derivada
- **Objetivo:** `kd task` (D93) cria/gerencia `plan`/`epic`/`issue`/`task`; `plan`/`epic` são
  **views** (`type=container`; id + eventos de filiação), com **backref por marcador** no corpo;
  ciclo de vida `submit/adopt/reorder/release/outcome/review`; hierarquia fechada
  `plan ⊃ epic ⊃ issue ⊃ task` (prof. máx. 4); `blocks` 1-based; sem self-reference.
- **Entregáveis:** `kd task`; validações de hierarquia; backref.
- **Decisões:** D52, D53, D93.
- **Aceite:** container não materializa verdade própria; profundidade, `scope` e `blocks`
  validados; pai↔filho íntegro.

### E08-T08 ☑ `compact` como proposta
- **Objetivo:** `compact(scope)` **propõe** (concat / keep_latest / merge_outcomes); o agente
  aceita ou rejeita.
- **Entregáveis:** gerador de propostas; aplicação sob aceite.
- **Decisões:** D47.
- **Aceite:** nenhuma fusão sem aceite; proposta mostra o antes/depois.

### E08-T09 ☑ Orçamento e streaming de saída
- **Objetivo:** `rewind`/pipe não montam saídas gigantes em memória.
- **Entregáveis:** escrever direto no `BufWriter` do stdout; `with_capacity` quando o tamanho é
  conhecido; cap/`try_reserve` nos buffers; `warnings[]` em vez de abortar.
- **Decisões:** D40, D82. **Políticas:** R04, R15, R33.
- **Aceite:** saída grande não cresce memória linearmente; orçamento respeitado.

## Definition of Done

- [x] `rewind` situa o estado e respeita orçamento sem tokenizer.
- [x] Handoff 1:1 por `context_id` garantido por teste de bytes.
- [x] `task`/`compact` são views/propostas, nunca verdade imposta.
- [x] Saída em streaming, sem acúmulo em memória.

## Não-objetivos

- Clustering (E10).
- Embeddings para descoberta de links (E11).
