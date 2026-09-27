# E17 — Robustez do worker de embeddings e do `--install`

> **Épico de evolução (pós-E15).** Consolida o plano aprovado em
> [`../proposals/worker_embeddings_install.md`](../proposals/worker_embeddings_install.md).
>
> **Versão alvo:** **0.5.0** (junto de E16). Muda a superfície: `--yes` sai (D14) e o
> `--install`/`--uninstall` deixam de pedir confirmação (D180).
>
> **Decisões propostas:** D180 (sem confirmação), D181 (verbosidade/stream), D182 (reconciliação
> e probe de endpoint/modelo), D183 (supply-chain do worker). **Políticas:** R20–R23 (logs),
> R33 (degradação graciosa), R43 (dependências), D14 (sem retrocompatibilidade), D79 (identidade
> do índice vetorial), D84 (derivado reconstruível), D131–D133 (worker/agendador).

## Objetivo

Eliminar as ambiguidades do worker de embeddings (**`kd drain service`** — E18/D186; ex-
`kd maintenance watch-service`): ações pontuais **objetivas e verbosas** (relatam cada passo),
**sem confirmação** (o comando explícito é o aceite), com **detecção de divergência** de
`embeddings.endpoint`/`model` (nada degrada em silêncio) e supply-chain endurecida — sem mudar o
contrato de bytes de `notas/`.

## Pré-requisitos

- E01–E15 ✅ (`make check` verde); E16 (qualidade/depreciação) pode correr em paralelo, mas o
  fecho (T08) consolida a versão 0.5.0 de ambos.
- **E18** (comandos scriptados) define **onde** o worker vive (`kd drain service`) e o **wrapper
  fino**/stream (D184); E17 entrega a **lógica de robustez**. A lógica de reconciliação/probe
  vive no **script** (`scripts/knudge-idle.sh`), não no binário (D184).
- Teste real feito no host: worker instalado (systemd), servidor em `8999`, projeto cadastrado.
- **Prioridade 1:** T03 (verbosidade/stream) — é a raiz da reclamação e a correção mais barata.
- **Correção crítica em seguida:** T01 (reconciliação de endpoint/modelo) — remove a degradação
  silenciosa.

## Revisão do plano (achados e ajustes)

1. **O progresso do `--install` morre no buffer.** `run_worker` usa `.output()` e descarta o
   **stderr** no sucesso; o script já loga os passos, mas eles não chegam ao usuário. O stream é
   entregue por **E18/T01** (wrapper fino canônico); T03 é a fatia do worker.
2. **Degradação silenciosa por endpoint divergente.** Worker em 8999 × config em 8080; `--status`
   diz `servidor: ok`. T01/T02 detectam e reportam — **no script** (D184).
3. **Identidade mentirosa do índice.** O modelo do config (msmarco) ≠ modelo servido (granite);
   T01 reconcilia e alinha D79.
4. **Ação explícita = aceite.** D180 remove o prompt e o `--yes`; os testes migram para
   `--dry-run`/`--script`.
5. **Supply-chain.** T05 alinha o worker ao endurecimento do `install.sh` (v0.4.0).
6. **Polimento.** T07 varre P6–P10 (read-only, `is_terminal`, `--every`, `pending=?`, mensagem).
7. **Pontos em aberto.** O usuário indicará mais itens antes do código; o épico reserva §Pontos
   em aberto e uma tarefa de fecho para incorporá-los.

## Performance e orçamento (herança de E15)

> O worker/`--install` é **fora do caminho quente**: nenhuma tarefa deste épico pode rodar em
> `ask`/`write`/`prime`/`task`. A herança de E15 é **manter** o piso (~4–13 ms) intacto.

- **Invariante:** reconciliação (T01) e probe (T02) só executam em `--install`/`--status`
  explícitos, **nunca** no auto-drain nem por comando; o `--status` faz **um** probe
  (`/health` curto), não uma varredura.
- **Orçamento:** e2e `--no-idle` de `ask`/`prime`/`rewind` **inalterado** (≤ ruído); o `--status`
  pode custar 1 round-trip de rede (aceito — é raro).
- **Protocolo:** `make bench` prova que o caminho quente não muda; teste do `--status` mede o
  probe; `--json 2>/dev/null` segue JSON válido (R20).
- **Padrões:** o script é a fonte da verdade (D184); o binário só invoca; stdout=dados,
  stderr=logs; sem dep nova (R43).

## Sequência de execução

```
Prioridade 1 (começa já): T03 (verbosidade/stream do worker)

A. Correção crítica
T01 (endpoint/modelo) → T02 (status/probe) → T04 (sem confirmação)

B. Segurança e polimento
T05 (supply-chain) → T06 (uninstall + GGUF) → T07 (polimento P6–P10)

C. Fecho
T08 (docs/goldens/matriz/CHANGELOG) — incorpora os pontos em aberto
```

## Tarefas

### E17-T01 ☐ D182 — reconciliação de `embeddings.endpoint` e `embeddings.model`
- **Escopo:** **no script** (`knudge-idle.sh`, D184), após instalar o servidor na porta `$PORT`,
  comparar o `embeddings.endpoint` e o `embeddings.model` **efetivos** do projeto (e/ou global)
  com o servidor instalado. Divergência ⇒ `warn` + `próximos:` com o `kd config set --key …
  --value …` exato; `--reconcile` (ou flag explícita) aplica a correção. Alinha o modelo ao GGUF
  baixado (D79). O binário só invoca (wrapper fino).
- **Perf:** só roda em `--install`/`--reconcile`; **não** é chamado no caminho quente.
- **Depende de:** T03 (para o aviso aparecer no stream).
- **Aceite:** teste de divergência (endpoint e modelo) emite o comando exato; `--reconcile`
  alinha e reindexa com intenção; `notas/` intacto.

### E17-T02 ☐ D182 — `--status` valida o endpoint efetivo (probe)
- **Escopo:** **no script**, `--status` faz probe do endpoint configurado (`/health` derivado ou
  `POST /v1/embeddings` curto) e reporta `endpoint: ok | fora | divergente (config=…, worker=…)`;
  mantém o `servidor: ok` do health do worker.
- **Perf:** 1 probe por `--status` (raro); nunca no caminho quente.
- **Depende de:** T01.
- **Aceite:** teste com endpoint divergente marca `divergente`; endpoint ausente marca `fora`;
  endpoint correto marca `ok`; `--json` ganha o campo (aditivo).

### E17-T03 ☑ D181 — verbosidade e stream do worker (prioridade 1)
- **Escopo:** o **stream do stderr** é implementado por **E18/T01** (wrapper fino canônico);
  aqui o **script** cobre cada passo com números: baixar llama.cpp, baixar GGUF (tamanho),
  escrever unidades, subir servidor (porta), cadastrar projeto, verificar. stdout continua só
  dados (R20).
- **Feito:** o script já loga cada passo via `log()` (stderr); o wrapper `commands/script.rs`
  (E18/T01) faz **stream do stderr** e captura o stdout. Teste
  `cli::watch_service_streams_stderr_and_keeps_stdout_as_data` prova o repasse.
- **Perf:** stream de stderr via E18/T01; nenhum custo no caminho quente.
- **Depende de:** nada.
- **Aceite:** `--install` (e `--uninstall`/`--subscribe`) exibem os passos em stderr; teste com
  `--script` fake que escreve em stderr verifica o repasse; `--json 2>/dev/null` segue JSON válido.

### E17-T04 ☑ D180 — remover confirmação (e `--yes`)
- **Escopo:** `Action::asks()` deixa de existir; o prompt e `confirm()` saem; `WatchServiceArgs.yes`
  sai (D14). Ações explícitas executam direto.
- **Feito:** `asks()`/`question()`/`confirm()` removidos de `watch.rs`; `--yes` removido de
  `WatchServiceArgs`; teste `watch_service_declined_does_nothing` removido e
  `watch_service_subscribe_runs_local_script` migrado (sem `--yes`). Docs (`09-maintenance`,
  `15-embeddings`) e superfície/matriz atualizadas.
- **Perf:** remoção de código (`asks`/`confirm`) — não afeta o caminho quente.
- **Depende de:** T03 (verbosidade garante visibilidade do que ocorre).
- **Aceite:** `--install`/`--uninstall`/`--subscribe`/`--unsubscribe` executam sem prompt; testes
  migrados para `--dry-run`/`--script` fake (remover `watch_service_declined_does_nothing`);
  `--yes` ⇒ uso (2); linha da matriz e `--help` atualizados.

### E17-T05 ☐ D183 — supply-chain do worker
- **Escopo:** verificar **SHA-256** do GGUF; pinar a **revisão** do modelo (sem `/resolve/main/`);
  validar o instalador do llama.cpp (checksum ou release pinada) em vez de `curl … | sh` cego;
  preferir o repo oficial do modelo.
- **Perf:** verificação de checksum é O(tamanho do arquivo), só no `--install` (raro).
- **Depende de:** T01 (modelo reconciliado).
- **Aceite:** download aborta com checksum inválido (teste com `--script`/fixture); revisão
  pinada documentada; `--dry-run` mostra a URL+hash.

### E17-T06 ☐ P5 — `uninstall` e o GGUF
- **Escopo:** o prompt/texto do `--uninstall` menciona o modelo; `--keep-model` preserva (default
  documentado) e a remoção vai para o trash. Nunca `rm`.
- **Perf:** operação de arquivo, rara; sem impacto no caminho quente.
- **Depende de:** T04.
- **Aceite:** `--uninstall` relata o destino do GGUF; `--keep-model` mantém o arquivo; teste.

### E17-T07 ☐ P6–P10 — polimento
- **Escopo:**
  - **P6:** só materializar o script embutido para ações que executam o worker (não `--status`).
  - **P7:** resolvido por D180 (`confirm()` sai); ajustar comentários.
  - **P8:** validar `--every` (formato) antes de escrever a unit, para **todos** os agendadores.
  - **P9:** `--status` explica `pending=?` (motivo da falha do `kd`).
  - **P10:** `drain --digest` distingue "nada pendente" de "indexado agora".
- **Perf:** validar `--every`/read-only **antes** de escrever a unit evita trabalho descartável.
- **Depende de:** T01–T06.
- **Aceite:** testes de `--every` inválido (exit 2, sem escrever unit); `--status` read-only não
  escreve staging; mensagens sem `?` cru.

### E17-T08 ☐ Fecho — docs, goldens, matriz e CHANGELOG
- **Escopo:** `docs/15-embeddings.md`, `docs/09-maintenance.md`, `SKILL.md`, `llms.txt`,
  `16_cli_surface.md`, `17_matriz_aceitacao.md`, `DIVERGENCES.md` (se borda), `CHANGELOG.md`
  (`[0.5.0]`), `Cargo.toml` (via `make update-version`); incorpora os **pontos em aberto**.
- **Depende de:** todas.
- **Aceite:** `make check` + `make ci` verdes; grep por `--yes`/prompt obsoleto; versão em sincronia.

## Definition of Done

- [ ] `make check` verde em cada tarefa; `make ci` verde ao fechar.
- [ ] Worker sob **`kd drain service`** (E18/D186); `--install`/`--uninstall` **verbosos** (cada
      passo em stderr, stream por E18/T01) e **sem confirmação** (D180/D181).
- [ ] Divergência de `embeddings.endpoint`/`model` **detectada e reportada** com comando exato
      (D182); `--status` faz probe.
- [ ] Supply-chain do worker endurecida (D183): GGUF com SHA-256 + revisão pinada; llama.cpp
      validado.
- [ ] P5–P10 resolvidos; `--yes` inexistente; testes migrados.
- [ ] Nenhum `src/` > 300 linhas; zero `unwrap/expect/panic/unsafe`; stdout = dados (R20).

## Não-objetivos

- Daemon/processo de fundo obrigatório (R16).
- Trocar o modelo default por outro de maior qualidade (fora de escopo).
- Confirmação para verbos destrutivos do corpus (`forget`/`prune` seguem D112).
- Dep nova sem ganho (R43).

## Riscos

| Risco | Mitigação |
|---|---|
| D180 faz `--install` rodar em testes | migrar para `--dry-run`/`--script` fake |
| D181 mistura log com stdout | stream só de stderr; stdout = envelope (R20) |
| D182 altera config do usuário | avisa com o comando; só `--reconcile` altera |
| D183 pinar quebra mirror | repo oficial + revisão documentada |
| remover `--yes` quebra scripts | D14; CHANGELOG + matriz |
| `--uninstall` remover GGUF compartilhado | `--keep-model` default preserva |

## Pontos em aberto (aguardando indicação do usuário)

> O usuário indicou que ainda tem pontos a acrescentar **antes** do início do código. Cada ponto
> vira uma tarefa `E17-Txx` (ou nota) aqui.

- _(a preencher)_
