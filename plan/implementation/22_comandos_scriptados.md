# E18 — Comandos scriptados (superfície mínima e cross-platform)

> **Épico de evolução (pós-E15).** Consolida o plano aprovado em
> [`../proposals/comandos_scriptados.md`](../proposals/comandos_scriptados.md).
>
> **Versão alvo:** **0.5.0** (junto de E16/E17). Muda a superfície: `watch-service` sai de
> `maintenance` e entra sob `drain` (D186).
>
> **Decisões propostas:** D184 (scripts no repositório, comando = wrapper fino), D185
> (cross-platform), D186 (`watch-service` sob `drain`; `maintenance` só revisão), D187 (`self
> upgrade` real), D188 (auditoria de verbos acionáveis). **Políticas:** R20–R23 (logs), R43
> (dependências), D14 (sem retrocompatibilidade), D69 (self), D131–D133 (worker/agendador), D165
> (verbosidade), D170 (`drain`).

## Objetivo

Manter a lógica de ações **complexas e essencialmente scriptadas** como **scripts shell no
repositório** (`scripts/`), com o binário como **wrapper fino** (resolver + evocar, `curl` +
checksum quando remoto), **sempre verboso** e **cross-platform** (Linux/macOS/Windows). Mover o
worker para o escopo do `drain` e deixar `maintenance` só com a revisão.

## Pré-requisitos

- E01–E15 ✅ (`make check` verde).
- E17 (worker/`--install`) proposto — **E18 reutiliza e generaliza** E17-T03 (stream/verbosidade)
  e E17-T04 (sem confirmação); E17-T05 (supply-chain) é a base de D184.
- `install.sh` endurecido (v0.4.0) como fonte da verdade do install de binário.
- **Prioridade 1:** T04 (superfície `drain`/`maintenance`) — é a mudança de contrato visível.
- **Em seguida:** T01/T02 (padrão de invocação + cross-platform), base dos demais.

## Revisão do plano (achados e ajustes)

1. **`self upgrade` é stub.** `self_cmd.rs::upgrade()` só retorna `invalid_input`; precisa virar
   script real (D187).
2. **`watch-service` já é script, mas o binário o embute e materializa.** A lógica está no
   `knudge-idle.sh`; falta formalizar o **wrapper fino** e o stream de logs (D184).
3. **Superfície.** `maintenance` hoje mistura revisão (`compact`/`learn`/`prune`) com operação
   (`watch-service`); a operação vai para `drain service` (D186).
4. **Supply-chain.** O `curl` do script exige **checksum** (D183/E17-T05) — o wrapper não pode
   baixar e executar às cegas.
5. **Cross-platform.** O `knudge-idle.sh` tem guia manual para Windows; falta o caminho
   automatizado (PowerShell) e a escolha de interpretador por SO (D185).
6. **Auditoria.** Outros verbos acionáveis (ex.: `self setup`, onboard) entram no escopo de D188.
7. **`install.sh` fica na raiz** (é o alvo canônico do `curl | bash`); `kd-upgrade.sh` o chama.
8. **Superfície: um só ponto de verdade.** `16_cli_surface.md` é atualizado no commit de cada
   épico que toca nomes; **E16 só muda comportamento** (não renomeia verbos) — ver
   [`revisao_integrada.md`](../proposals/revisao_integrada.md) §3 (C10).

## Performance e orçamento (herança de E15)

> O wrapper fino é **off-path**: só verbos acionáveis (`drain service`, `self upgrade`) evocam
> script; `ask`/`write`/`prime`/`task` **não** pagam nada. A herança de E15 é manter o piso.

- **Invariante:** resolver/evocar script **não** entra em `Session::open` nem no auto-drain; a
  resolução é local (embutido) por padrão — **sem download por comando**.
- **Orçamento:** e2e `--no-idle` dos verbos frequentes **inalterado**; os verbos acionáveis podem
  custar o tempo do script (aceito — são raros).
- **Protocolo:** `make bench` prova que os verbos frequentes não mudam; smoke por script; o
  `--dry-run` mostra o comando sem executar (teste rápido).
- **Padrões:** `curl`+checksum só quando remoto; nada de dep nova (R43); stdout=dados (R20).

## Sequência de execução

```
Prioridade 1: T04 (superfície drain/maintenance)

A. Fundação do padrão
T01 (wrapper fino + stream) → T02 (cross-platform)

B. Aplicação
T03 (watch-service como script) → T05 (self upgrade) → T06 (auditoria D188)

C. Fecho
T07 (docs/goldens/matriz/CHANGELOG) — incorpora os pontos em aberto
```

## Tarefas

### E18-T01 ☑ D184 — padrão de invocação (wrapper fino + stream)
- **Escopo:** função comum de invocação: resolver o script (embutido/local por padrão;
  `--url`/`curl` com **checksum** quando remoto) → executar com o shell do SO → **stream** de
  stdout/stderr → propagar exit code. stdout = dados (R20); logs em stderr, verbosos.
- **Feito:** `commands/script.rs` (`Source::{Embedded,Local,Remote}`, `resolve`, `run`):
  materializa o embutido, roda `bash` com **stderr herdado (stream)** e stdout capturado, e
  recusa `--url` sem `--sha256` (SHA-256 verificado antes de executar). `watch.rs` virou
  wrapper fino (usa o módulo); `--sha256` adicionado a `WatchServiceArgs`; dep `sha2` no CLI.
  Testes: `cli::watch_service_streams_stderr_and_keeps_stdout_as_data`,
  `cli::watch_service_propagates_script_failure`, `cli::watch_service_remote_requires_checksum`
  + vetor SHA-256.
- **Perf:** off-path; resolver local é O(1); download só remoto e explícito.
- **Depende de:** E17-T03/T05 (reutiliza).
- **Aceite:** teste com script fake (stderr/stdout) prova o stream e o exit code; `--json
  2>/dev/null` segue JSON válido; sem download sem checksum.

### E18-T02 ☑ D185 — cross-platform
- **Escopo:** detecção de SO no wrapper; Linux (arch/ubuntu/fedora/…) e macOS via `bash`; Windows
  via `powershell -File <script>.ps1` ou caminho manual. Cada script documenta o suporte por SO.
- **Feito:** `shell_spec(script, os)` (pura) decide `bash` em Unix e
  `powershell -NoProfile -File` para `.ps1` no Windows; script Unix no Windows ⇒ `invalid_input`
  com o guia manual. `plan_command`/`shell_prefix` refletem o shell do SO no `--dry-run`;
  `after_help` do `drain service` diz o suporte. Testes unitários por SO
  (`shell_spec_bash_on_unix`, `shell_spec_powershell_on_windows`,
  `shell_spec_rejects_unix_script_on_windows`, `is_powershell_is_case_insensitive`); job
  `windows` no CI roda `cargo test --bin kd commands::script`.
- **Perf:** detecção de SO é O(1) no startup do verbo acionável; nada no caminho quente.
- **Depende de:** T01.
- **Aceite:** matriz SO × script documentada (`docs/15-embeddings.md`); smoke por SO no CI
  (job `windows`) + testes unitários determinísticos; `--help` diz o que é suportado. ✔

### E18-T03 ☑ D184 — `watch-service` como script + wrapper fino
- **Escopo:** `scripts/knudge-idle.sh` é a fonte da verdade; o binário só resolve/evoca (sem
  lógica de agendador em Rust). Consolida E17-T03/T04 (stream, sem confirmação).
- **Feito:** o script é embutido por `include_str!` (`commands/drain/service.rs`) e o wrapper
  (`commands/script.rs`) resolve (`Embedded`/`Local`/`Remote`+checksum) e evoca; nenhuma lógica de
  agendador em Rust. `--dry-run` mostra o comando completo; stream de stderr entregue (T01).
- **Perf:** `service.rs` reduzido a wrapper; nada no caminho quente.
- **Depende de:** T01/T02.
- **Aceite:** `watch.rs` reduzido a wrapper; teste de invocação; `--dry-run` mostra o comando. ✔

### E18-T04 ☑ D186 — superfície: `watch-service` sob `drain`; `maintenance` só revisão
- **Escopo:** criar `kd drain service <ação>` (`--install|--subscribe|--unsubscribe|--status|--uninstall`);
  remover `watch-service` de `maintenance`; `maintenance` = `compact|learn|prune`. `--help`/`prime`/
  matriz/`docs` atualizados. Revisa D170.
- **Feito:** `DrainArgs` ganhou o subcomando opcional `service` (`args_conflicts_with_subcommands`);
  `WatchServiceArgs`/`DrainCommand` moveram para `cli/health.rs`; `commands/drain.rs` virou
  `commands/drain/{mod,service}.rs`; `MaintenanceCommand::WatchService` removido; `prime`/goldens,
  matriz, `16_cli_surface.md`, `docs/` e `llms.txt`/`SKILL.md`/`README.md` atualizados.
- **Perf:** renomear/remover verbo não muda o custo dos verbos frequentes.
- **Depende de:** T03.
- **Aceite:** `kd maintenance watch-service` ⇒ uso (2); `kd drain service --status` funciona;
  testes migrados (`cli::drain_service_*`); linhas da matriz e `prime` atualizadas.

### E18-T05 ☑ D187 — `self upgrade` real
- **Escopo:** `scripts/kd-upgrade.sh` (novo) que **chama** o `install.sh` oficial (release +
  checksum), verbose e cross-platform; `kd self upgrade` evoca o script (sem lógica em Rust).
  Revisa D69/D165.
- **Feito:** `scripts/kd-upgrade.sh` (embutido por `include_str!`) baixa o `install.sh` oficial
  (ou usa um local), verifica `KNUDGE_INSTALL_SHA256` quando definido e executa com `VERSION`;
  nunca `curl … \| sh` cego; `--dry-run` imprime o plano (stdout). `kd self upgrade` virou wrapper
  fino (`commands/self_cmd/upgrade.rs`) com `--dry-run`/`--version`/`--script`/`--url`/`--sha256`
  (`UpgradeArgs`); o stub saiu. Testes `cli::self_upgrade_invokes_script`,
  `cli::self_upgrade_dry_run_shows_plan`, `cli::self_upgrade_remote_requires_checksum` (+ unit
  `self_cmd::upgrade::embedded_upgrade_script_is_present`).
- **Perf:** `self upgrade` é raro; chama `install.sh` (não reimplementa).
- **Depende de:** T01/T02.
- **Aceite:** `self upgrade` deixa de ser stub; teste com `--script`/fixture prova a invocação;
  `--dry-run` mostra o plano; release verifica checksum. ✔

### E18-T06 ☑ D188 — auditoria de verbos acionáveis
- **Escopo:** reavaliar `self setup`, onboard (`init`), `sync`, `doctor --fix` e afins: se
  essencialmente scriptável, mover para `scripts/` e reduzir o comando à ação mínima; senão,
  registrar a decisão de manter nativo (com motivo).
- **Feito:** auditoria escrita na proposta
  ([`../proposals/comandos_scriptados.md`](../proposals/comandos_scriptados.md) §9) com o veredito
  por verbo; **nenhuma migração adicional**. Permanecem nativos
  `init`/`sync`/`doctor --fix`/`self setup`/`self completions`/`hooks`/`forget`/`prune` (domínio com
  portas, `--json` e determinismo); só orquestração de SO/rede (`knudge-idle.sh`, `kd-upgrade.sh`,
  `install.sh`) e utilitários de dev vivem em `scripts/`.
- **Perf:** auditoria é decisão de plano, não código quente.
- **Depende de:** T01–T05.
- **Aceite:** decisão escrita por verbo (script/nativo + motivo); o que migrar tem teste de smoke. ✔

### E18-T07 ☑ Fecho — docs, goldens, matriz e CHANGELOG
- **Escopo:** `docs/09-maintenance.md`, `docs/15-embeddings.md`, `docs/13-self.md`, `SKILL.md`,
  `llms.txt`, `16_cli_surface.md`, `17_matriz_aceitacao.md`, `README.md`, `CHANGELOG.md`
  (`[0.5.0]`), `Cargo.toml` (via `make update-version`); incorpora os **pontos em aberto**.
- **Depende de:** todas.
- **Aceite:** `make check` + `make ci` verdes; grep por `maintenance watch-service` obsoleto;
  versão em sincronia.
- **Feito:** docs/`README` sincronizados (D184–D188); nenhum `maintenance watch-service`
  obsoleto; `CHANGELOG` `[0.5.0]`; `make update-version VERSION=v0.5.0`.

## Definition of Done

- [ ] `make check` verde em cada tarefa; `make ci` verde ao fechar.
- [x] `drain service` é **script** em `scripts/`; o binário é **wrapper fino** (resolver + evocar),
      com **stream** de logs (D184). `self upgrade` pendente (T05).
- [x] Cross-platform documentado e testado (Linux/macOS/Windows) (D185).
- [x] `watch-service` sob `drain service`; `maintenance` só `compact|learn|prune` (D186).
- [x] `self upgrade` real com checksum (D187); auditoria de verbos acionáveis escrita (D188).
- [ ] `install.sh` permanece na raiz (target do `curl | bash`); `kd-upgrade.sh` o invoca.
- [ ] Nenhum `src/` > 300 linhas; zero `unwrap/expect/panic/unsafe`; stdout = dados (R20).

## Não-objetivos

- Reescrever `install.sh` (fonte da verdade do install de binário).
- Suportar Windows no agendador além do guia manual (systemd/launchd não existem lá).
- Daemon obrigatório (R16); dep nova (R43).

## Riscos

| Risco | Mitigação |
|---|---|
| `curl` do script sem verificação | checksum SHA-256 obrigatório (E17-T05); embutido/local por padrão |
| mover `watch-service` quebra usuários/scripts | D14; `--help`/matriz/CHANGELOG apontam o novo caminho |
| cross-platform incompleto | guia manual + `powershell`; smoke por SO |
| duplicar lógica entre `install.sh` e `kd-upgrade.sh` | `kd-upgrade.sh` chama o `install.sh` |
| scripts divergirem do binário | teste de smoke por script (`--dry-run`/fixture) |

## Pontos em aberto (aguardando indicação do usuário)

> O usuário indicou que ainda tem pontos a acrescentar **antes** do início do código. Cada ponto
> vira uma tarefa `E18-Txx` (ou nota) aqui.

- _(a preencher)_
