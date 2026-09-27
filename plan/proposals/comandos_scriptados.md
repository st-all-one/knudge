# Comandos scriptados (superfície mínima e cross-platform) — plano de implementação

> **Status:** proposta (não implementada). Épico executável em
> [`../implementation/22_comandos_scriptados.md`](../implementation/22_comandos_scriptados.md).
>
> **Objetivo:** tirar do binário a lógica de ações **complexas e essencialmente scriptadas**
> (`kd self upgrade`, worker de embeddings/`watch-service`) e mantê-las como **scripts shell no
> repositório** (`scripts/`). O comando passa a ser um **wrapper fino** que evoca o script (via
> `curl` quando remoto + shell), **sempre expondo logs com verbosidade** — e funcionando em
> **Linux (arch/ubuntu/fedora/…), macOS e Windows**.
>
> **Versão alvo:** **0.5.0** (junto de E16/E17; muda a superfície — `watch-service` sai de
> `maintenance`).

Temas: (1) arquitetura de scripts; (2) cross-platform; (3) superfície `drain`/`maintenance`;
(4) `self upgrade`; (5) auditoria de outros verbos acionáveis.

---

## 0. Decisões propostas (numeração provisória)

> E16 usa D172–D179 e E17 usa D180–D183 (provisório); este plano segue em D184–D188.

| Id | Decisão |
|---|---|
| **D184** | **Scripts acionáveis vivem no repositório** (`scripts/`). O binário **não reimplementa** a lógica: ele **localiza e executa** o script com o shell do SO (embutido/local; `curl` + checksum quando remoto), faz **stream** de stdout/stderr e propaga o exit code. Vale para `self upgrade` e para o worker de embeddings. |
| **D185** | **Cross-platform.** Cada script detecta o SO e cobre Linux (arch/ubuntu/fedora/…), macOS e Windows (PowerShell/`.cmd`); onde não houver suporte nativo, imprime o caminho manual. O wrapper escolhe o interpretador certo por SO. |
| **D186** | **Superfície: `watch-service` sob `drain`.** O worker/agendador passa a ser gerido em `kd drain service <ação>`; `kd maintenance` fica **só com os comandos de revisão** (`compact`/`learn`/`prune`). Revisa D170/D131–D133. |
| **D187** | **`kd self upgrade` real.** Deixa de ser stub (`invalid_input`): baixa e executa o instalador oficial (`install.sh`, release com **checksum SHA-256**) via script em `scripts/`, com logs verbosos e cross-platform. Revisa D69/D165. |
| **D188** | **Auditoria de verbos acionáveis.** Todo verbo que hoje faz trabalho de "script" (setup de cliente, onboard, sync, `doctor --fix`…) é reavaliado: se for essencialmente scriptável, vira script em `scripts/` e o comando faz só a **ação mínima** (resolver + evocar). |

---

## 1. Princípios (do pedido)

1. **Lógica no script, comando fino.** O binário resolve o script e o executa; não duplica a
   receita. Menos superfície Rust para manter e mais fácil de ajustar por SO.
2. **Verbosidade sempre.** O usuário vê **cada passo** (baixar, verificar, instalar, subir,
   registrar) em stderr; stdout = só dados (R20).
3. **Ação explícita = aceite.** Sem confirmação para comandos preparados (D180, E17).
4. **Uma fonte da verdade por script**, versionada em `scripts/` e coberta por teste.

---

## 2. Estado atual (evidência)

- **`kd self upgrade`** existe na superfície (`SelfCommand::Upgrade`) mas é **stub**:
  `self_cmd.rs::upgrade()` retorna `Error::invalid_input("atualização automática não disponível…")`.
- **`install.sh`** (raiz do repo) é o instalador real: modo release (download + **checksum
  SHA-256** + `verify_binaries`) e modo `--from-source`; endurecido na v0.4.0.
- **`scripts/knudge-idle.sh`** é o worker/gerenciador do agendador; o binário o **embute**
  (`include_str!`) e o materializa em cache, mas a lógica já está toda no script.
- **`scripts/`** já existe (`bump-version.sh`, `check_file_length.sh`, `knudge-idle.sh`,
  `package.sh`).
- **`kd maintenance watch-service`** hoje carrega `compact | learn | prune | watch-service`.

---

## 3. Superfície proposta (D186)

```
kd drain --status | --digest [--force]        # fila de embeddings (inalterado)
kd drain service --install | --subscribe | --unsubscribe | --status | --uninstall
                                             # worker/agendador (ex-watch-service)
kd maintenance compact | learn | prune        # só revisão (propostas)
```

> **Decidido (Q9):** a forma é **`kd drain service <ação>`** (subcomando `service`), por ser
> explícito e extensível. `install.sh` permanece na **raiz** (target do `curl | bash`) — Q8.

---

## 4. Scripts

| Script | Papel | SO |
|---|---|---|
| `scripts/knudge-idle.sh` | worker de auto-drain + agendador (systemd/launchd/cron) + servidor de embeddings | Linux/macOS (bash); Windows → caminho manual/PowerShell |
| `scripts/kd-upgrade.sh` (novo) | baixa e executa o `install.sh` oficial (release + checksum) | Linux/macOS (bash); Windows → PowerShell |
| `install.sh` (raiz) | instalador canônico (curl target) | Linux/macOS |

**Invocação (wrapper fino, D184):** resolver o script (embutido/local por padrão; `--url`/`curl`
com **checksum** quando remoto) → executar com o shell do SO → **stream** de stderr/stdout →
propagar exit code. Alinha a E17-T03 (stream) e E17-T05 (supply-chain).

**Cross-platform (D185):** o script detecta o SO; onde o bash não existe (Windows), o wrapper
chama `powershell -File <script>.ps1` (ou imprime o caminho manual). O guia manual do
`knudge-idle.sh` já cobre Windows; falta o caminho automatizado.

---

## 5. Riscos e mitigação

| Risco | Mitigação |
|---|---|
| `curl` do script sem verificação (supply-chain) | **checksum SHA-256** obrigatório (D183/E17-T05); embutido/local por padrão |
| mover `watch-service` quebra scripts/usuários | D14 (sem retrocompatibilidade); `--help` e matriz apontam o novo caminho; CHANGELOG |
| cross-platform incompleto (Windows) | guia manual + `powershell`; testar em CI (smoke por SO) |
| duplicar lógica entre `install.sh` e `kd-upgrade.sh` | `kd-upgrade.sh` **chama** o `install.sh` (não reimplementa) |
| scripts divergirem do binário | teste de smoke por script (execução com `--dry-run`/fixture) |

---

## 6. Não-objetivos

- Reescrever `install.sh` (é a fonte da verdade do install de binário).
- Suportar Windows no worker de agendador além do guia manual (systemd/launchd não existem lá).
- Dep nova (R43); daemon obrigatório (R16).

---

## 7. Pontos em aberto (aguardando indicação do usuário)

> O usuário indicou que ainda tem pontos a acrescentar **antes** do início do código. Registrar
> aqui, um por linha, para entrarem como tarefas de E18.

- _(a preencher)_

---

## 8. Performance (herança de E15)

> O orçamento completo está no épico
> [`../implementation/22_comandos_scriptados.md`](../implementation/22_comandos_scriptados.md)
> §"Performance e orçamento".

- **Off-path:** só verbos acionáveis (`drain service`, `self upgrade`) evocam script; os verbos
  frequentes (`ask`/`write`/`prime`/`task`) **não** pagam nada. Resolução local (embutido) por
  padrão — sem download por comando.
