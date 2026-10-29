# Plano — Recap de atividade de IA e handoff (`kd rewind --with-activity`)

> Implementação sequencial dos 3 pontos acordados: (1) filtro quantitativo em `kd ai`,
> (2) recap derivada, (3) apêndice de atividade no `kd rewind`.

## 0. Invariantes (R01–R44, AGENTS.md)

- `knudge-core` nunca acessa FS/relógio direto — só portas. Adaptadores na borda.
- Sem `unwrap`/`expect`/`panic`/`unsafe` em `src/`; arquivo ≤ 300 linhas.
- Degradação graciosa (R33): fonte ausente de IA = sucesso vazio + warning, nunca erro.
- Determinismo: ordenação `(created_at, agent_id, id)`, dedup por `id`.
- `kd ai` permanece read-only; nenhum escreve em `.knudge/ai/`.
- Embeddings canônicos (`notas/`) intocados — sem índice vetorial para IA memory.

## 1. Ponto 1 — Filtro quantitativo + temporal em `kd ai`

**Objetivo:** permitir recorte por quantidade (e tempo, opcional) na listagem e no recall,
sem mudar o default de 20 (já aplicado).

### 1.1 CLI (`crates/knudge-cli/src/cli/ai.rs`)

Adicionar a `AiArgs`:
- `--since <TS>` (timestamp ms, opcional) — filtra `created_at >= since`.
- `--until <TS>` (timestamp ms, opcional) — filtra `created_at <= until`.
- `--all-agents` (bool) — desliga o filtro por agente default (ver §2).
- Manter `--agent-id`/`--agent-role` existentes.

### 1.2 Comando (`crates/knudge-cli/src/commands/ai.rs`)

- `ordered_notes`: aplicar `since`/`until` sobre `created_at` antes do `truncate(limit)`.
- `search`: passar `since`/`until` para `Filter` (ou filtrar hits pós-recall por `created_at`).
- Parsing de timestamp via `crate::commands::parse::timestamp_opt` (já usado no rewind).

### 1.3 Testes

- Regressão: `kd ai memory --since <ts> --limit 5` devolve só entradas no intervalo.
- `--until` análogo.
- Combinação `--since`+`--until`.

**Entrega:** make check verde; goldens de `prime` inalterados (novas flags opcionais).

## 2. Ponto 2 — Recap derivada (`kd ai ... --recap`)

**Objetivo:** agregar o session-log em um resumo compacto e determinístico, em vez de listar.

### 2.1 CLI

Adicionar a `AiArgs`:
- `--recap` (bool) — modo de agregação (só faz sentido em `session-log`, mas aceito em `memory`).
- `--all-agents` (bool) — inclui todos os agentes; default = agente "atual".

### 2.2 Semântica de "agente atual"

O knudge não rastreia agente ativo. Definição determinística:
- Agente atual = `agent_id` com o maior `created_at` no shard consultado
  (empate → lexicograficamente menor).
- Se não houver entradas, omite o filtro e exibe todos (com aviso).
- Ao final do recap, listar os agentes disponíveis e como consultá-los:
  `kd ai session-log --agent-id <id> --recap`
  e `kd ai session-log --all-agents --recap`.

### 2.3 Módulo de recap

Criar `crates/knudge-cli/src/commands/ai/recap.rs` (ou `crates/knudge-core/src/...`?).
Decisão: manter na CLI (é apresentação), core fornece dados.

Função `recap(args, corpus) -> (String, serde_json::Value)`:
1. Selecionar notas filtradas por agente (default atual) e ordenadas por `(created_at, agent_id, id)`.
2. Janela quantitativa: últimas `N` (default 30) entradas — `--limit` reusa N.
3. Agregar deterministicamente:
   - `last_decision`: última entrada com `type=decision`.
   - `last_verification`: última entrada cujo statement/body contenha marcador de verificação
     (heurística: statement inicia com "verificação"/"teste"/"check" ou `type=fact` com tag `verify`).
   - `last_error`: última entrada com `type=error`.
   - `files_touched`: conjunto ordenado de anchors/paths citados nas entradas da janela.
   - `commands`: comandos distintos (dedup por `id`) nas entradas da janela, mais recentes primeiro.
4. Renderizar texto compacto (≤ ~400 tokens) + JSON com os campos acima + `agents_available`.

### 2.4 Formato de saída (texto)

```
[recap] agente=omp-01j...  (30 entradas, janela=30)
decisão: <statement>
verificação: <statement>
erro: <statement ou "nenhum">
arquivos: a.rs, b.rs
comandos: cargo test, cargo clippy
outros agentes: kd ai session-log --agent-id <id> --recap
```

### 2.5 JSON

```json
{
  "agent_id": "...",
  "window": 30,
  "last_decision": {...} | null,
  "last_verification": {...} | null,
  "last_error": {...} | null,
  "files_touched": [...],
  "commands": [...],
  "agents_available": ["id1", "id2"]
}
```

### 2.6 Testes

- Recap com 1 agente, 5 entradas → campos corretos.
- `--all-agents` agrega todos.
- Fonte ausente → sucesso vazio + warning, lista de agentes vazia.
- Determinismo: mesma entrada → mesmo output (golden).

## 3. Ponto 3 — Apêndice no `kd rewind --with-activity`

**Objetivo:** um único comando devolve conhecimento + tarefas + atividade recente.

### 3.1 CLI (`crates/knudge-cli/src/cli/rewind.rs`)

Adicionar:
- `--with-activity` (bool).
- `--activity-agent <ID>` (opcional; default = agente atual, mesma regra do §2.2).
- `--activity-limit <N>` (default 30).

### 3.2 Comando (`crates/knudge-cli/src/commands/rewind.rs`)

- Se `--with-activity`: carregar a fonte de session-log (`Source::jsonl_dir` ou `--source`? default convencional),
  derivar corpus, executar a recap do §2.3, e anexar ao `out.text` como bloco final
  (budgetado via `budget::apply` com o orçamento restante).
- Avisos da leitura de IA propagados em `warnings`.
- Se fonte ausente: omitir bloco + warning (R33), não falhar.

### 3.3 Interação com budget

- O rewind já aplica budget ao texto principal. O apêndice deve ser adicionado **após** o budget principal,
  com budget próprio (ex.: 20% do total ou `--activity-budget`), truncando se necessário.
- Marcar no JSON: `"activity": { ... recap json ... } | null`.

### 3.4 Testes

- `kd rewind --with-activity` em projeto sem `.knudge/ai/` → sucesso, `activity: null`, warning.
- Com shards presentes → bloco anexado, JSON com recap.
- Budget: apêndice não excede o limite.

## 4. Ordem de implementação

1. Ponto 1 (filtros) — base para 2 e 3.
2. Ponto 2 (recap) — reutilizado por 3.
3. Ponto 3 (rewind apêndice).

Cada ponto: implementar → testes → `make check` → commit.

## 5. Riscos e mitigações

- **"Agente atual" heurístico** pode surpreender em multi-agente; mitigado por listar agentes
  e instruir `--agent-id`/`--all-agents`.
- **Recap heurístico de "verificação"** (substring) pode ser frágil; mitigado por também
  aceitar tag `verify` e documentar o critério.
- **Acoplamento rewind↔ai**: manter em módulo compartilhado (`commands/ai/recap.rs`) para
  não duplicar lógica.
- **Performance**: recap lê shards a cada rewind; aceitável (read-only, uma passagem).
  Se virar gargalo, cachear em memória por sessão.

## 6. Decisões a registrar (Dxx)

- D (novo): default de agente no recap = mais recente por `created_at`; `--all-agents` para global.
- D (novo): janela do recap por quantidade (default 30), não por tempo.
- D (novo): `kd rewind --with-activity` é opt-in; fonte ausente degrada graciosamente.

## 7. Checklist de conclusão

- [ ] Ponto 1: flags `--since`/`--until` em `kd ai`, testes verdes.
- [ ] Ponto 2: `--recap`, `--all-agents`, listagem de agentes, golden.
- [ ] Ponto 3: `--with-activity` no rewind, budget, warning em fonte ausente.
- [ ] `make check` verde em cada ponto.
- [ ] `MODULE.md`/docs atualizados se novos módulos.
- [ ] `wiki/CHANGELOG.md` atualizado.
- [ ] `plan/implementation/17_matriz_aceitacao.md` atualizado.