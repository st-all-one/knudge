# Reforma da superfície do `kd` — plano de implementação

> **Status:** planejamento (nenhum código alterado ainda).
> Objetivo: tornar o `kd` **explícito** (diz o que quer e o que fez), dar **help embutido** a
> cada verbo, **promover** `doctor` e `drain` a verbos de topo, `prime` compacto por padrão e
> **remover redundância** da superfície — sobretudo em torno do `body`. Nenhuma chave TOON muda;
> o contrato `--json` só ganha campos aditivos.

Temas: (1) doctor; (2) drain; (3) prime compacto; (4) verbosidade; (5) help embutido;
(6) revisão de redundância; (7) documentação viva/estática.

---

## 0. Decisões propostas (numeração provisória)

| Id | Decisão |
|---|---|
| **D163** | `kd doctor` de topo; sem flag = validação completa (+ auditoria); `--fix` corrige o reversível; `--explain` detalha; `--audit` **deixa de existir** |
| **D164** | `kd help` == `kd --help`; `kd help <verbo>` mostra o help do verbo |
| **D165** | `init`, `self`, `config`, `sync` e `maintenance` **verbosos** (o que fizeram) e `maintenance` com `próximos:` |
| **D166** | `prime` **compacto por padrão** (essencial); `--long` = completo + gramática TOON |
| **D167** | **help embutido** por verbo: ensina, exemplifica e explicita o escopo (`--universe`) |
| **D168** | revisão de **redundância** (`body`, aliases) e um só vocabulário por conceito |
| **D169** | **documentação viva** (`prime`/`help`) e **estática** (`docs/`, `SKILL.md`, `AGENTS.md`, `llms.txt`, matriz) em sincronia |
| **D170** | `kd drain` de topo (absorve `kd knowledge digest`); sem flag = `--help` (não executa); `--digest [--force]` = digestão com log mínimo (`--force` apaga `.idx/` e redigeri tudo, último recurso); `--status` = estado rico + recomendação |

---

## 1. Fase 1 — `kd doctor` de topo (D163)

### 1.1 Superfície (`crates/knudge-cli/src/cli/`)
- `mod.rs`:
  - `Command::Doctor(DoctorArgs)` (novo), `name()` → `"doctor"`.
  - `DoctorArgs { fix: bool, explain: bool }` (`--fix`, `--explain`).
  - `dispatch` em `commands/mod.rs` ganha `Command::Doctor(args) => doctor::run(session, args)`.
- `maintenance.rs`: remover a variante `Doctor { fix, audit }` de `MaintenanceCommand`.
  `kd maintenance` passa a expor só `compact | learn | prune | watch-service`.

### 1.2 Comando (`crates/knudge-cli/src/commands/doctor/mod.rs`)
1. Montar uma vez: `store`, `events`, `index = session.index()?`, `graph = session.graph()?`,
   `root`, `thresholds`, `now_ms`, `lock_stale_ms = 30_000`.
2. `DoctorInput` completo (fs, root, project_root, store, events, config, graph, now_ms,
   lock_stale_ms, thresholds).
3. `let report = if args.fix { doctor_fix(&input)? } else { doctor(&input)? };`
4. `AuditInput` (fs, root, project_root, store, graph, index, now_ms, lock_stale_ms, thresholds)
   → `let audit = audit(&input)?;` **depois** do fix (locks/resíduos relidos do disco).
5. Texto:
   - uma linha por check: `ok|warn|fail <check> <detail>`;
   - `auditoria: N problema(s)` (ou `auditoria: limpo`);
   - se `--explain`, um bloco por achado: `[tipo] <título>` + `esperado:` + `encontrado:` +
     `ação:`;
   - `próximos:` com 1 linha de ação por categoria com problema.
6. JSON (aditivo): `{healthy, checks[], fixed[], audit{clean,total,integrity_issues,
   supersession_cycle_details,dependency_cycle_details,broken_anchor_details,duplicate_pairs,
   missing_edge_details,stale_lock_details}, suggestions[], explain[]?}`.
7. `warnings` do doctor no `Output`.

### 1.3 `commands/maintenance/mod.rs`
- Remover `DoctorMode`, `doctor_cmd`, `audit_report`, `doctor_report` e imports que só eles usam
  (`AuditInput`, `DoctorInput`, `audit`, `doctor`, `doctor_fix`, `EdgeKind`, `json`).
- Arquivo encolhe; `run` só despacha `compact|learn|prune|watch-service`.

### 1.4 Sugestões (`próximos:`) — mapa achado → comando
| Achado | Sugestão |
|---|---|
| `duplicates` | `kd maintenance compact --universe` |
| `broken_anchors` | `kd write --update <ID> --anchor <PATH>` (ou remover) |
| `missing_edges` | `kd write --link <FROM:ARESTA:TO>` |
| `stale_locks` | `kd doctor --fix` |
| `integrity`/ciclos | `kd doctor --explain` + revisão manual |
| fila de embeddings pendente | `kd drain --status` (ver Fase 2) |
| inconsistência persistente do índice vetorial | `kd drain --digest --force` (último recurso) |

### 1.5 Testes e docs
- Migrar chamadas `maintenance doctor` → `doctor` e `--audit` → (default) em:
  `cli.rs:1473`, `real_usage.rs:380/382/440`, `regressions_031.rs:48/52/71/85/86/268/269/283`,
  `legacy_migration.rs:101/114/132/169/188`.
- Novo teste: `doctor` de topo faz auditoria por padrão; `--explain` traz `esperado`/`encontrado`;
  `kd maintenance` sem `doctor` dá exit 2 (comando desconhecido).
- `17_matriz_aceitacao.md`: linha `kd maintenance doctor` → `kd doctor [--fix] [--explain]`.
- `16_cli_surface.md` §2/§9: tabela de verbos e manutenção.
- `prime` (texto vivo) e goldens (Fase 2).

---

## 2. Fase 2 — `kd drain` de topo (D170)

Absorve `kd knowledge digest` (D145). Três modos, mutuamente exclusivos:

| Invocação | Papel |
|---|---|
| `kd drain` | **helper**: `kd drain` sem flag **não executa nada** — equivale a `kd drain --help` (ensina os modos e o que fazer) |
| `kd drain --digest [--force]` | **digestão em lote** até esvaziar/estagnar, com **log mínimo**; `--force` é o **último recurso** — apaga `.idx/` (derivado) e redigeri todas as notas do zero |
| `kd drain --status` | **estado rico**, sem drenar: o que falta e o que é recomendado |

### 2.1 Superfície
- `cli/mod.rs`: `Command::Drain(DrainArgs)`; `DrainArgs { digest: bool, status: bool, force: bool }`; `name()` →
  `"drain"`. `--force` só é válido com `--digest` (incompatível com `--status`).
- `cli/knowledge.rs`: remover `KnowledgeDigestArgs` e a variante `KnowledgeCommand::Digest`.
- `cli/knowledge.rs` exports (`pub use`) ajustados em `cli/mod.rs`.

### 2.2 Comando (`crates/knudge-cli/src/commands/drain.rs`)
- Mover a lógica de `commands/knowledge/digest.rs` para cá e apagar o módulo antigo.
- `--status` (e o `--help`) monta o **estado rico**:
  - `enabled` (`embeddings.enabled`), `provider`, `mode`, `dimensions`;
  - contagem por estado carregando `EmbeddingIndex::load` + `store.list_ids()` +
    `state_of`: `indexed`, `pending`, `stale`;
  - `recommendation` textual:
    - `provider = none` → "embeddings desligados; nada a fazer";
    - sem índice → "nada digerido ainda; rode `kd drain --digest`";
    - `pending + stale > 0` → "há N pendentes/estragados; rode `kd drain --digest`";
    - tudo `indexed` → "fila limpa".
- `--digest`: laço `embedder::drain_once` até `pending == 0` ou sem progresso (guarda contra loop);
  **log mínimo** (`--json` traz `batches`, `indexed`, `cache_hits`); sem ruído por nota.
- `--digest --force`: remove o diretório `.idx/` (100% derivado: retrieval/embeddings/cache/contexts) e roda a
  digestão de **todas** as notas desde o zero; é a medida última quando o índice vetorial divergir de
  forma não reparável. `--force` sem `--digest` é `invalid_input` (2). Como só apaga derivado (D84),
  não pede confirmação, mas **avisa** em `warnings` o que foi removido e quanto foi redigerido.
- `kd drain` sem flag é equivalente a `--help` (não drena). `--status` imprime o estado rico. Ambos degradam graciosamente (R33) com `warnings` quando o provedor está fora.
- Reusar `embedder::{build, meta, drain_once}` (sem duplicar).
- Arquivo ≤ 300 linhas; se passar, dividir `drain/{state.rs,run.rs}`.

### 2.3 JSON (aditivo)
```json
{ "enabled": true, "provider": "http", "mode": "lazy", "dimensions": 384,
  "indexed": 10, "pending": 3, "stale": 1,
  "recommendation": "há 4 pendentes/estragados; rode `kd drain --digest`" }
```
`--digest` acrescenta `batches`, `cache_hits`; com `--force` acrescenta `rebuilt: true` e
`removed: ["retrieval", "embeddings", "cache", "contexts"]`; `--status` não muta `.idx/`.

### 2.4 Testes e docs
- Migrar `knowledge digest --status|--drain` → `drain --status|--digest` em:
  `cli.rs:2045/2465/2502/2972`, `real_usage.rs:190`, e quaisquer outros (grep `"digest"`).
- Novos testes: `drain` sem flag imprime o help e não escreve em `.idx/`; `--digest` esvazia e não loopa; `--status` mostra
  `indexed/pending/stale` + recomendação; `provider = none` explica; `--digest --force` apaga `.idx/` e
  torna a digerir todas as notas (fila volta a `indexed`); `--force` sem `--digest` → exit 2.
- `17_matriz_aceitacao.md`: linha de `kd knowledge digest` → `kd drain`.
- `docs/15-embeddings.md`, `docs/07-knowledge.md`, `docs/09-maintenance.md`, `docs/troubleshooting.md`.
- `prime`, `SKILL.md`, `llms.txt`, `README.md`, `AGENTS.md`, `16_cli_surface.md`.
- Auto-drain ocioso (`commands/idle.rs`) segue chamando `embedder::drain_once` — só mensagens citam
  `kd drain`.

---

## 3. Fase 3 — `prime` compacto por padrão (D166)

- `commands/prime.rs`: `PrimeFormat::{Compact, Long}`; `run` default = `Compact`; `--long` = `Long`.
  - **Compact** (essencial, byte-idêntico por versão): `ask` (com `--anchor` e flags críticas),
    `write` (com `--anchor` + **corpo obrigatório**), `rewind`, `task`, `sync`, ciclo e a regra de
    `ID`; sem a seção longa de schema.
  - **Long**: o protocolo atual completo + gramática TOON/schema (`long_section`).
- `PrimeArgs`: trocar `--long` por `--long` mantido; documentar que o default encolheu.
- `init` usa `PrimeFormat::Compact` como prompt inicial.
- Regenerar goldens: `crates/knudge-cli/tests/golden/prime.txt` e `json_prime.json`
  (`golden.rs` deve ganhar versão `Compact` e, se útil, `Long`).
- Atualizar `docs/02-prime.md`, `AGENTS.md`, `SKILL.md`, `llms.txt`.

---

## 4. Fase 4 — verbosidade explícita (D165)

Princípio: **stdout = o que fiz** (passo a passo) e **stderr = logs**; `--json` continua o
contrato. Cada verbo abaixo ganha texto explícito e, onde couber, `próximos:`.

| Comando | Texto explícito novo |
|---|---|
| `kd init` | lista do que foi criado/alterado: `notas/`, `eventos/`, `.idx/`, `.locks/`, `config.toml`, `AGENTS.md`, `.gitattributes`, `.git/info/exclude`, `.agents/skill/kd/SKILL.md` (marcando novo vs. já existia) |
| `kd self version` | `kd <versão> (<commit/target?>)` + nota de onde achar help |
| `kd self completions` | `completions <shell> geradas (N bytes)`; onde instalar |
| `kd self setup` | `recipe <cliente> gravada em <path>` + próximo passo |
| `kd self upgrade` | erro orientando o canal de origem (mantém) |
| `kd config get/set/unset/list` | `chave = valor (projeto|global) — <path do arquivo>` |
| `kd sync` | `sync <branch>: N arquivo(s) commitados (<hash curto>)`; se nada, `nada a sincronizar` |
| `kd maintenance compact/learn/prune` | contagem por `kind` + `próximos:` (aplicar via `write`/`forget`) |
| `kd maintenance watch-service` | resumo do plano/estado + `próximos:` |

- Logs `info` estruturados via porta `Logger` (nunca corpo/segredo — R21–R23).
- Testes: ajustar asserts que esperam texto curto (`real_usage.rs`, `cli.rs`, goldens de `init`).

---

## 5. Fase 5 — help embutido por verbo (D167)

- Habilitar o help nativo: `disable_help_subcommand = true` **removido** → `kd help` e
  `kd help <verbo>` (D164).
- Em `cli/{mod,task,knowledge,maintenance,rewind}.rs`:
  - `#[command(arg_required_else_help = true)]` (ou equivalente) para que verbo sem argumentos
    mostre o help em vez de erro seco;
  - `long_about` + `after_help` por verbo: 1 exemplo mínimo, "quando NÃO usar", e a
    **explicitação de escopo** (`map`/`rank`/`learn`/`compact`/`prune`/`task list` exigem
    `--scope`/filtro ou `--universe`);
  - `after_help` global com o ciclo `ask → write → task → sync`, corpo e âncoras.
- `kd ask`/`kd write` sem argumentos: passar a mostrar o **help** (hoje o `ask` devolve uso com
  exit 2 — D130). Decidir: manter exit 2 com texto de ajuda, ou exit 0. **Proposta:** manter o
  exit 2 (contrato D130) mas trocar o texto pelo help completo do verbo.

---

## 6. Fase 6 — redundância (D168), com foco em `body`

Auditoria da superfície; remover aliases/vocabulário duplicado:
- `write`: remover `visible_alias = "anchors"` (o canônico é `--anchor`); conferir se existe
  `--with-body` residual (o canônico é `--full-content`, D146).
- Um só nome por conceito:
  - corpo posicional em `write`/`task new`;
  - `--full-content` em `ask`/`task list`;
  - corpo por id em `ask --id` / `task show`;
  - lote em `--params`/`--batch`.
- Corpo documentado **num só lugar** (`prime` + `after_help`), sem repetição por verbo.
- Revisar `--params` vs flags (D147) só para documentar precedência, sem remover.
- Testes de regressão para cada alias removido (deve dar exit 2).

---

## 7. Fase 7 — documentação (D169)

Checklist por arquivo (grep `maintenance doctor`, `--audit`, `knowledge digest`):

- `plan/implementation/16_cli_surface.md` (verbos, §9, §13, §16) e `17_matriz_aceitacao.md`.
- `docs/`: `00-quickstart`, `01-conceitos`, `02-prime`, `03-init`, `07-knowledge`,
  `09-maintenance`, `12-sync`, `13-self`, `15-embeddings`, `troubleshooting`, `README`.
- `SKILL.md`, `AGENTS.md`, `llms.txt`, `README.md`.
- Goldens de `prime` e novos testes de help.
- `CHANGELOG.md` (`[Não publicado]`).

---

## 8. Ordem e critérios de aceite

1. **Fase 1 (doctor)** — `make check` verde; `kd doctor`/`--fix`/`--explain` testados; `--audit`
   ausente; matriz/goldens atualizados.
2. **Fase 2 (drain)** — `kd drain`/`--digest [--force]`/`--status`; `knowledge digest`
   removido; testes migrados.
3. **Fase 3 (prime)** — default compacto; `--long`; goldens regenerados.
4. **Fase 4 (verboso)** — init/self/config/sync/maintenance com texto explícito.
5. **Fase 5 (help)** — `arg_required_else_help` + `after_help` por verbo; `kd help`.
6. **Fase 6 (redundância)** — aliases removidos com teste de regressão.
7. **Fase 7 (docs)** — vivos e estáticos em sincronia.

**Guardrails (cada fase):**
- `make check` verde (fmt + clippy `-D warnings` + test + gate 300 linhas).
- Sem `unwrap/expect/panic/unsafe`; `src/` ≤ 300 linhas; stdout=dados, stderr=logs.
- `--json` só aditivo; nenhuma chave TOON nova; goldens atualizados com intenção.
- `--help` de todo verbo lista os comandos novos; `kd help` == `kd --help`.
- Ao concluir cada fase: marcar em `plan/implementation/` e atualizar `CHANGELOG.md`.

---

## 9. Riscos e mitigação

| Risco | Mitigação |
|---|---|
| `doctor` fica lento (auditoria O(N²) de dedup) | aplicar Onda 3 de [`otimizacoes_performance.md`](otimizacoes_performance.md) antes |
| remover `maintenance doctor` quebra scripts | sem alias (D14); documentar no `CHANGELOG` e na matriz |
| `prime` compacto quebra cache byte-idêntico | versão em `data.version`; regenerar goldens no mesmo commit |
| verbosidade vaza para `--json` | todo texto extra JSON vive em `data`; stdout não muda no modo máquina |
| `--digest` loop infinito | parar quando `pending == 0` ou sem progresso entre lotes |
| `--digest --force` apaga derivado sem volta | é 100% derivado/reconstruível (D84); `warnings` listam o removido e o redigerido |
| help grande demais poluir | `after_help` curto; detalhe no `prime`/`docs` |
