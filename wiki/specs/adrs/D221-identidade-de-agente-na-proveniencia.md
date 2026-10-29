# D221 — Identidade de agente na proveniência (schema 3)

- **Status:** Aceita
- **Categoria:** Eixo de memória / proveniência (D207)

## Contexto

D207 introduziu `provenance` como mapa PROV-lite `{entity, activity, agent}` — suficiente para
saber **qual agente lógico** produziu a nota, mas não **qual instância** (sessão/processo), nem se
era o agente principal ou um subagente, nem quem o criou. Com shards por escritor (D220), a
identidade passa a ser estrutural: o shard nomeia a instância, e a nota tem de a registar para que
o merge entre shards e o filtro por agente sejam possíveis sem consultar o caminho.

## Decisão

`schema_version` sobe para **3** (`SCHEMA_VERSION = 3`). Notas v2 continuam válidas: `validate`
aceita **2 e 3** e a leitura aplica defaults.

A identidade vive em `provenance`, **sem novas chaves de topo**:

| Campo | Papel | Regra |
|---|---|---|
| `agent` | nome lógico (filtro histórico, D218) | inalterado |
| `agent_id` | instância (sessão/processo) | ≤ 128 chars, sem caracteres de controlo |
| `agent_role` | `main` \| `subagent` | enum **fechado** |
| `parent_agent_id` | criador do subagente | recomendado só em `subagent`; ≤ 128, sem controlo |

- **Defaults de leitura** (nunca reescrevem a nota): `agent_id` = `agent_id` ou `agent` ou
  `"legacy"`; `agent_role` = `main`; `parent_agent_id` = ausente.
- **Ordem de serialização** preserva os bytes v2: `entity`, `activity`, `agent`, `agent_id`,
  `agent_role`, `parent_agent_id` (campos ausentes omitidos, nunca `null` — D13). O emissor só
  inclui identidade **presente**, pelo que uma nota v2 faz round-trip byte-idêntico.
- **Validação** (`malformed_agent_identity_is_rejected`): tamanho e caracteres de controlo são
  rejeitados na escrita; `agent_role` fora do enum é erro (D212 lista as opções).
- `agent_id` **não entra no `id`** da nota: o id continua content-addressed (D01) e uma nota
  reescrita por outra instância mantém a identidade do conteúdo, não a do escritor.

Os campos são transportados por `Meta`/`Filter` (`agent_id`, `agent_role`, `parent_agent_id`) e
persistidos no índice (`retrieval-v5`, D222/D218). `kd ai` e `kd ask` ganham
`--agent-id`/`--agent-role` (repetível ou vírgula, D210) e `--params` transporta os mesmos nomes.

## Consequências

- Uma nota responde “quem, qual instância, em que papel, criada por quem” sem depender do nome do
  ficheiro.
- O bump 2→3 é **aditivo**: nenhuma nota antiga é reescrita e a ordem canônica das chaves
  existentes não muda (DIVERGENCES #120).
- `"legacy"` é o único default sintético: notas sem identidade ficam filtráveis, mas nunca são
  mutadas para “ganhar” um `agent_id`.
- Um índice anterior não conhece a identidade: `retrieval-v4` é rejeitado e reconstruído —
  derivado, sem perda (D15).
- `parent_agent_id` continua **recomendado**, não obrigatório, em `subagent`: o knudge não impõe
  topologia de agentes.
