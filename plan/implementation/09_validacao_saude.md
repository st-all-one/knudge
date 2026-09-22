# E09 — Validação, saúde e leitura tolerante

> **Fase 2** (com a parte tolerante já no MVP). Fechamento por **evidência**, catálogo de
> **validators**, `audit` e `doctor --fix`, leitura que **não derruba a base** por uma nota
> ruim, âncoras com hash e **staleness sinalizada**, confiança **derivada**.
>
> **Decisões:** D16, D17, D18, D19, D43, D46, D48, D54, D55, D84, D86, D87.
> **Políticas:** R10, R33 (ver [`14_revisao_tecnica.md`](14_revisao_tecnica.md)).

## Objetivo do épico

O corpus **não apodrece em silêncio**: sabe-se o que está quebrado, o que é stale e por quê,
e o reparo do que é reversível é automático.

## Pré-requisitos

E06, E07.

## Tarefas

### E09-T01 ☐ Catálogo de validators e resolução de `checks`
- **Objetivo:** `validators.yaml` executável; resolução
  `checks(task) = explícitos ∪ AGENTS.md ∪ por anchor`; severidade.
- **Entregáveis:** catálogo; resolvedor; severidades.
- **Decisões:** D54.
- **Aceite:** três fontes combinadas; validator ausente/ inválido reportado.

### E09-T02 ☐ Fechamento por evidência e `outcomes[]`
- **Objetivo:** ao fechar task, **rodar validators**, inferir `outcomes[]`
  (`status/duration/agent/notes/recorded_at`) e gravar `evidence`; confirmação é **derivada**.
- **Entregáveis:** fechamento; `evidence` (mapa de resultados); `outcomes[]`.
- **Decisões:** D48, D55.
- **Aceite:** task não fecha sem evidência; confirmação nunca é armazenada, só derivada.

### E09-T03 ☐ `audit()`
- **Objetivo:** relatório de integridade, âncoras quebradas, duplicatas e **arestas sugeridas
  faltantes**; leitura, não mutação.
- **Entregáveis:** `audit()`.
- **Decisões:** D46.
- **Aceite:** relatório determinístico; sugestões não alteram o grafo.

### E09-T04 ☐ `doctor [--fix]`
- **Objetivo:** checks de schema/TOON, integridade, ciclos, âncoras, duplicatas, locks stale,
  config, `body_hash` desatualizado, `eventos.jsonl` malformado e **divergência
  canônico↔derivado**; `--fix` corrige o **reversível** e reporta com orientação.
- **Entregáveis:** checks; modo `--fix`; mensagens com instrução de correção.
- **Decisões:** D19, D84.
- **Aceite:** `--fix` idempotente (rodar 2× não muda nada na segunda); divergência de índice
  detectada e reconstruída.

### E09-T05 ☐ Leitura tolerante e `strict` (config)
- **Objetivo:** chave desconhecida **tolera com warning**; `type` desconhecido **rejeita por
  nota** (não derruba o comando); linha/nota malformada **skip + warning + orientação**;
  **`strict` de config (D94)** opcional para CI.
- **Entregáveis:** leitor Postel; leitura de `[behavior] strict`.
- **Decisões:** D16, D17, D18, D94.
- **Aceite:** uma nota ruim (cada classe) não impede um `recall`; `strict=true` falha como
  esperado.

### E09-T06 ☐ Âncoras com hash e verify-on-hit
- **Objetivo:** `anchors` com `path` + **`content_hash` derivado** (em `.idx/anchors.jsonl`,
  **nunca** no frontmatter); no hit, rechecks do hash; papel **`cited`** invalida,
  **`context`** não; nota stale é **sinalizada, não apagada**.
- **Entregáveis:** registro de hashes; verificação lazy; `stale_reason` granular.
- **Decisões:** D43, D86.
- **Aceite:** renomear/apagar arquivo ancorado marca a nota stale; `context` não invalida;
  nada é removido sem decisão.

### E09-T07 ☐ Confiança e salience derivadas
- **Objetivo:** calcular no `recall` uma confiança derivada
  (`sim × drift × idade + feedback`), com pisos, **nunca armazenada**; separada da `confidence`
  declarada.
- **Entregáveis:** função pura `confidence_score`; salience de decay (E10).
- **Decisões:** D87.
- **Aceite:** proptest de monotonicidade (sim/confirmed ↑; drift/idade ↓); resultado em [0,1].

## Definition of Done

- [ ] Nenhuma nota isolada derruba um comando.
- [ ] `doctor --fix` idempotente e a divergência de derivado é detectada.
- [ ] Stale é sinalizado com motivo; confiança é sempre derivada.

## Não-objetivos

- Purge de inativos por tempo (E10).
- Embeddings semânticos na confiança (E11).
