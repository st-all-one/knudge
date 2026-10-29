# D224 — Sem migração e sem retrocompatibilidade: o agente reescreve a memória

- **Status:** Aceita
- **Categoria:** Versionamento e migração (D14/D70)

## Contexto

O plano v2 previa um `kd ai migrate` que convertesse `.agents/memory/*.jsonl` no layout novo, com
leitura de fallback durante uma janela de transição. Isso contradiz D14 (sem retrocompatibilidade,
sem aliases) e D70 (sem migração) e cria duas verdades temporárias: um caminho legado lido em
paralelo, um comando de conversão a manter e testes para os dois formatos. O corpus legado é
**propriedade do agente** (D218), não um corpus histórico do knudge que precise de ser preservado
byte-a-byte.

## Decisão

**Corte limpo, explícito e sem exceções:**

- **Não existe `kd ai migrate`.** O verbo não é registado; invocá-lo é comando desconhecido
  (exit 2, D130/D212). Não há plano de o reintroduzir.
- **`.agents/memory` não é lido.** Não há `default` para esse caminho, não há fallback silencioso,
  não há deteção automática de layout antigo. Os defaults são exclusivamente
  `.knudge/ai/memory` e `.knudge/ai/session-log` (D220).
- **Não há camada de compatibilidade**: nenhum alias de flag, nenhum leitor de formato antigo,
  nenhum shim de transição. O único legado tolerado é o **schema**: notas v2 continuam válidas na
  leitura (D221).
- **O agente reescreve a memória no layout novo.** A migração de dados é uma tarefa do dono da
  fonte: ele lê o seu store antigo e escreve `.knudge/ai/<area>/<agent_id>.jsonl` (D219/D220). O
  knudge não move, copia nem apaga ficheiros do agente.
- Esta decisão **substitui a proposta de migração do plano v2**. A mudança é **breaking** e sai
  documentada no `CHANGELOG` 0.6.3: quem apontava `--source .agents/memory/notes.jsonl` continua a
  poder fazê-lo **explicitamente**, mas nada é descoberto por convenção.

## Consequências

- Uma só verdade e um só layout: o código, os testes e a documentação descrevem um caminho, não
  dois.
- Um utilizador que não migre vê `[no_results]`/lista vazia na área default — comportamento
  honesto (D152), não um erro nem uma leitura silenciosa do store antigo.
- Zero superfície de migração a manter: sem comando, sem flags, sem testes de conversão.
- O custo é do agente, uma vez, e é explícito no seu plano de trabalho; o knudge permanece
  read-only sobre a fonte (D218).
- Revisa D218 (que ainda citava `.agents/memory` como default) e confirma D14/D70 para esta
  superfície.
