# D222 — `created_at` é o eixo temporal; `--sort id|time`

- **Status:** Aceita
- **Categoria:** Eixo de memória / ordem temporal (D07/D26)

## Contexto

A ordem da fonte JSONL era sempre por `id` (D218). Isso é determinístico, mas `id` é
content-addressed (D01) e **não diz nada sobre quando** a nota foi registada: listar a memória ou
reordenar hits por recência era impossível. O JSONL já carrega `created_at` (UTC ms, D07) e um
shard por agente (D220) permite que cada escritor mantenha o seu relógio coerente.

## Decisão

- **`created_at` em milissegundos é o eixo temporal** da memória. O escritor mantém-no
  **monotónico por agente**: uma nota nova de um agente nunca tem `created_at` menor que a última
  que ele próprio escreveu (empate no mesmo ms é aceitável e desempatado adiante).
- A ordem determinística de leitura/ordenação é **`(created_at, agent_id, id)`**: tempo primeiro,
  depois a instância (dois agentes no mesmo ms), depois o conteúdo.
- `kd ai` ganha **`--sort id|time`**, default **`id`** (comportamento anterior preservado). `time`
  aplica `(created_at, agent_id, id)`; `id` mantém a ordem por id.
- No **recall** (`kd ai` com query, e `kd ask`) o default continua **score**; `--sort time`
  **reordena apenas os hits já limitados** — não muda o conjunto recuperado nem o corte de
  `--limit`, só a ordem de apresentação.
- `created_at` e a identidade (D221) aparecem no `--json` de `kd ai` e nos hits de `kd ask --json`
  (`created_at`, `agent`, `agent_id`, `agent_role`, `parent_agent_id`; `null` quando ausente).
  O topo do JSON ganha `{area, source, sort, query, data}` + `warnings[]`.
- `--params` aceita `sort`, `agent_id`, `agent_role`, `area`, `source`, `limit` e `query`; o objeto
  define a consulta inteira (D147).

## Consequências

- “O que aconteceu por último” é respondível sem um log paralelo: o tempo é o da nota, não o do
  ficheiro nem o do índice.
- Reordenar não re-ranqueia: `--sort time` sobre hits limitados é uma operação de apresentação, o
  que mantém `score` como contrato do recall (D81).
- Um `created_at` em falta lê `0` na ordenação (nunca inventa um instante); notas antigas ficam no
  início da linha do tempo.
- O relógio é uma porta (`Clock`, D07/R03): o core não chama `SystemTime::now`, e o monotónico é
  responsabilidade do escritor — o knudge não bloqueia um escritor com relógio errado.
- O índice persiste `created_ms` no `Meta` (`retrieval-v5`), então a ordenação e o `--sort time`
  não relêem a fonte.
