# D223 — Dedup por `BTreeMap` e conflito determinístico entre shards

- **Status:** Aceita
- **Categoria:** Eixo de memória / leitura concorrente (D18/D26)

## Contexto

Com um shard por agente (D220), o mesmo `id` pode aparecer em mais de um shard: um agente reescreveu
uma nota que outro já tinha, ou dois agentes derivaram a mesma nota content-addressed. A leitura
precisa de uma regra **total** e **reprodutível** — sem depender da ordem do sistema de ficheiros,
do escalonamento ou do shard chegar primeiro. Também precisa de custo aceitável: memória de agente
cresce e a lista de shards não é pequena.

## Decisão

- **Dedup com `BTreeMap<String, Note>`**: uma passagem por shard, inserção ordenada, saída final
  ordenada por `id`. Custo **O(N log N)** e ordem determinística sem `HashMap` (proibido no core,
  R03/DIVERGENCES #15).
- **Dentro do mesmo shard:** vence a **última linha** (D26). O shard é append-only (D219), logo a
  linha posterior é a revisão mais recente daquele escritor.
- **Entre shards:** vence o maior tuplo **`(created_at, agent_id, caminho do shard)`** — tempo,
  depois instância, depois origem. O caminho entra como último desempate para que dois shards com o
  mesmo `agent_id` (ex.: `a.jsonl` e `a-1234abcd.jsonl`) ainda tenham ordem estável. Comparação
  estrita: empate total mantém o primeiro lido, que é determinístico porque os shards são lidos em
  ordem lexicográfica.
- **Linha malformada:** *skip + warning* com o **caminho do shard** e o **número da linha**
  (D18); nunca falha a leitura. A linha parcial no fim do ficheiro segue a mesma via (D219).
- **Leitura concorrente sem lock:** cada linha é escrita completa; o leitor pode ler enquanto um
  escritor faz append. Uma linha parcial no fim é ignorada com warning; as linhas completas
  anteriores são válidas. Nada de lock de leitura nem de snapshot consistente do diretório.
- Ficheiro ou diretório ausente ⇒ corpus vazio, exit 0 (nunca é erro).

## Consequências

- O resultado de uma leitura é função do **conteúdo** dos shards, não do momento nem da ordem de
  descoberta — rebuilds do índice devolvem o mesmo corpus.
- Conflitos reais (dois agentes editaram a mesma nota) não são escondidos: o perdedor é apenas
  ignorado naquela leitura, e o warning nomeia o shard onde a linha foi descartada.
- Sem lock, o custo de leitura concorrente é zero e não há risco de roubar lock de escritor
  (DIVERGENCES #88).
- A última linha do shard pode ser revista por um append posterior; a leitura seguinte já a vê —
  monotonicidade é por agente (D222), não global.
- `DIVERGENCES.md` (#118) trava o desempate e o warning de linha malformada.
