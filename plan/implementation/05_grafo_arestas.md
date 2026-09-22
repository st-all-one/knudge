# E05 — Grafo e arestas

> **Fase 0.** Arestas **explícitas são a fonte primária**; a extração por regex é apenas
> **sugestão revisável**. Aqui entram o vocabulário fechado, a integridade referencial e a
> detecção de ciclos de supersessão.
>
> **Decisões:** D45, D46, D49, D50, D51, D84, D98.
> **Políticas:** R04 (ver [`14_revisao_tecnica.md`](14_revisao_tecnica.md)).

## Objetivo do épico

Um grafo confiável: arestas declaradas e auditáveis, sem falsos positivos permanentes, sem
dangling, sem ciclos que derrubem conhecimento válido.

## Pré-requisitos

E02, E03.

## Tarefas

### E05-T01 ☑ Vocabulário fechado de arestas
- **Objetivo:** enum fechado `references`, `depends_on`, `contradicts`, `supports`, `extends`,
  `replaces`, `rejects`, `results_in`; `link(from, kind, to)` cria aresta explícita.
- **Entregáveis:** enum; `link()`; serialização das arestas no frontmatter.
- **Decisões:** D49, D51.
- **Aceite:** kind fora do enum é rejeitado; `link` cria aresta e atualiza o grafo.

### E05-T02 ☑ Extração como sugestão revisável
- **Objetivo:** extração conservadora (ids, wikilinks, padrões verbais) roda **no write** e
  grava **sugestões** — nunca aresta definitiva; `expand` ignora sugestões; `audit` lista.
- **Entregáveis:** extrator; armazenamento de sugestões (derivado); `audit` de sugestões.
- **Decisões:** D49, D50.
- **Aceite:** sugestão não altera `expand`; falso positivo é revisável/descartável.

### E05-T03 ☑ Integridade do grafo
- **Objetivo:** `referential-integrity` (sem dangling), bidirecionalidade
  `replaces ↔ superseded_by` (D46) e diagnóstico no `doctor`.
- **Entregáveis:** validadores de integridade.
- **Decisões:** D46.
- **Aceite:** aresta para id inexistente é reportada; `replaces` sem par é reportado.

### E05-T04 ☑ Detecção de ciclos de supersessão
- **Objetivo:** SCC iterativo (Tarjan/DFS); **membros de ciclo não demovem**; `expand` não
  entra em loop.
- **Entregáveis:** detector de ciclos; marcação no rebuild.
- **Decisões:** D45.
- **Aceite:** ciclo A→B→A detectado; membros preservados; `expand` termina.

## Definition of Done

- [x] `expand` confia só no explícito.
- [x] Integridade e ciclos travados por testes com grafos adversariais.
- [x] Sugestões são separadas e auditáveis.

## Não-objetivos

- Clustering semântico (E10).
- Decay de âncoras (E09/E10).
