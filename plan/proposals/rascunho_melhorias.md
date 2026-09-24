# Rascunho — melhorias de uso diário (atualizado)

> **Status:** rascunho vivo (não é decisão). Origem: uso do `kd` ao longo de um dia + a avaliação
> `knudge × seeds × mulch` (`knudge-ts/eval/REPORT2.md`). **Atualizado após o ciclo D134–D152**:
> as 7 observações iniciais foram absorvidas por decisões fechadas ou viraram tarefas.
>
> Regras invioláveis (AGENTS.md): `make check` verde; `src/` ≤ 300 linhas; sem
> `unwrap/expect/panic/unsafe`; sem chave TOON nova sem `Dxx`; núcleo puro via portas; "propor,
> nunca agir em silêncio" (D47).

## 1. Os 7 pontos originais × destino

| # | Observação | Destino |
|---|---|---|
| 1 | `prime` caro (protocolo de 1202 tok) | **Aberto** — superfície mudou (D134–D146); revisitar `--compact` |
| 2 | Precisão do `ask` / mistura trabalho+conhecimento | **D146** (mistura) + **D151** (canais no `--json`, recalibração offline) |
| 3 | `learn` ruidoso (paths irrelevantes) | **Confirmado** (tarefa) — filtro de path |
| 4 | Atrito de setup dos embeddings | **D148** (cache versionado) + **Confirmado** (preflight, simplificado) |
| 5 | Higiene de docs (contagem de verbos) | **Confirmado** (propagação) — superfície mudou muito |
| 6 | Observabilidade local de busca | **D152** — demanda removida; busca vazia → `[no_results]` |
| 7 | Validação de escala/concorrência | **Confirmado** (tarefa) — agora cobre o layout D150 |

## 2. Decisões fechadas no ciclo (D134–D152)

| Dxx | Tema | Doc |
|---|---|---|
| D134 | `epic` é raiz, `issue` opcional, `scope=plan` sai | `d134_epic_raiz_issue_opcional.md` |
| D135 | `--anchor` único link externo; `expires_at`/`not_before` removidos (28→26) | `d135_ancora_unica_agendamento.md` |
| D136 | um agente: fim da posse; `--since` fora de `task list`; `mode`/`actor` podados | `d136_agente_unico_sem_posse.md` |
| D137 | `show` completo + `list --full-content` | `d137_task_show_completo.md` |
| D138 | `task plan` só `--prompt`/`--submit` | `d138_task_plan_prompt_submit.md` |
| D139 | `graph` enxuto + `plan.md` como floresta | `d139_graph_enxuto_floresta.md` |
| D140 | posicional = conteúdo; `--summary`; `--body` fora; stdin/heredoc | `d140_posicional_conteudo.md` |
| D141 | `task new --params`/`--batch` (cria, re-parenta, liga) | `d141_task_batch.md` |
| D142 | poda de `write`: `--checks`/`confidence` saem; `--outcome` fica | `d142_write_poda.md` |
| D143 | ponto de partida do conhecimento + princípio do escopo | `d143_escopo_conhecimento.md` |
| D144 | escopo obrigatório em `learn`/`compact`/`prune`/`task list` | `d144_escopo_obrigatorio.md` |
| D145 | fim do `eval`; `index` → `knowledge digest` | `d145_fim_eval_digest.md` |
| D146 | `ask` só conhecimento + superfície enxuta | `d146_ask_conhecimento.md` |
| D147 | `--params` universal + stdin/heredoc universal | `d147_params_universal.md` |
| D148 | cache vetorial versionado (`.knudge/emb_cache.jsonl`) | `d148_cache_vetorial_versionado.md` |
| D149 | fim do `type=container`; grupo derivado de `scope=epic` | `d149_desambiguar_container.md` |
| D150 | mapa material versionado (`notas/<tipo>/` + `MAP.md` + hubs) | `d150_mapa_material.md` |
| D151 | `ask` expõe canais no `--json`; recalibração offline | `d151_ask_channels.md` |
| D152 | busca vazia → `[no_results]`; observabilidade removida | `d152_no_results.md` |

## 3. Abertos e tarefas confirmadas

### 3.1 `prime --compact` — **aberto** (P0, esforço S)
`PrimeArgs` só tem `--long`; o `prime` cresceu. Com a superfície redefinida (D134–D152), um
`--compact` (~200–300 tok) para sessões curtas ainda fecha a lacuna. Sem `Dxx` (modo de saída de
D57); golden novo.

### 3.2 `learn`: filtro de path — **confirmado** (P0, esforço S)
`write_gaps` (`maintenance/learn.rs`) propõe para qualquer path (`.gitattributes`, `.knudge/`…).
Predicado `is_ignorable_path` (dotfiles, `.knudge/`, `.git/`, lockfiles, extensões não-código) +
exigir **≥2 eventos** fora do allowlist. Sem `Dxx`; teste de unidade.

### 3.3 Embeddings: preflight acionável — **confirmado, simplificar ao máximo** (P1, esforço S)
Provedor `http` inalcançável → **um** warning com o comando exato (`kd knowledge digest --status`,
`kd maintenance watch-service --install`), reaproveitando a sonda curta existente. Nada além
disso. Sem `Dxx` (R33).

### 3.4 Higiene de docs — **confirmado** (P2, propagação)
Contagem de verbos divergia (11/12/13); a superfície mudou (D145/D146). Fixar a fonte em
`16_cli_surface.md` e propagar a `README`/`ARCHITECTURE`/`llms.txt`. Corrigir os **bugs de doc**:
`why` (`anchor_match`/`universal`, não `anchor`/`lexical`) e `--via extends` (não `refines`).
Script leve opcional (`scripts/check_docs.sh`).

### 3.5 Escala/concorrência — **confirmado** (P2, esforço M)
`tests/stress.rs` não cobre corpus grande. Cenário determinístico (fakes) com N=10k: rebuild do
índice, latência do `ask`, catch-up da fila — e o **layout D150** (`notas/<tipo>/`). Sem `Dxx`.

## 4. Fora de escopo (não fazer agora)

- `custom_types`/labels livres (enum fechado é decisão).
- Nova chave TOON para prioridade (derivar por impacto, D109).
- `tokio full`/`reqwest`/DB/servidor HTTP (R16/R43).
- Migração de corpus / aliases de campo (D14).

## 5. Perguntas em aberto

1. `prime --compact` entra agora ou fica para depois da implementação de D134–D152?
2. A recalibração de ranking (D151) fica só na bancada `bench/` ou vira um golden versionado no repo?

## 6. Ordem de execução sugerida (quando houver código)

1. **D135** — contrato de bytes (chaves, `TOON.md`, goldens, views).
2. **D149** — tipos (remove `container`, grupo derivado).
3. **D134** — hierarquia (`scope` 4→3, adjacência relaxada).
4. **D150** — layout (`notas/<tipo>/`) + mapa material.
5. **D136/D139/D138/D141** — superfície de tarefa.
6. **D140/D147** — posicional/conteúdo + `--params`.
7. **D143/D144/D146/D151/D152** — escopo, `ask` e saída.
8. **D142/D145/D148** — poda de `write`, `digest`, cache versionado.
9. **D137** — `show`/`list --full-content` (consome os campos reorganizados).

Cada etapa fecha com `make check` verde.
