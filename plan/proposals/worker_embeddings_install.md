# Robustez do worker de embeddings e do `--install` — plano de implementação

> **Status:** proposta (não implementada). Épico executável em
> [`../implementation/21_worker_embeddings_install.md`](../implementation/21_worker_embeddings_install.md).
>
> **Objetivo:** remover as **ambiguidades** do worker de embeddings (**`kd drain service`** —
> E18/D186; ex-`kd maintenance watch-service`) e do script (`knudge-idle.sh`), tornando as ações
> pontuais (`--install`/`--uninstall`) **objetivas e verbosas** (relatam cada passo) e **sem
> confirmação** (o comando explícito já é o aceite). Corrige a **degradação silenciosa** por
> divergência de `embeddings.endpoint`/`model`. A lógica de reconciliação/probe vive no **script**
> (E18/D184); o binário só invoca.
>
> **Versão alvo:** **0.5.0** (junto de E16; a superfície muda: `--yes` sai, D14).

Temas: (1) reconciliação de endpoint/modelo; (2) `--status` com probe; (3) verbosidade/stream;
(4) fim da confirmação; (5) supply-chain; (6) polimento.

---

## 0. Decisões propostas (numeração provisória)

> E16 usa D172–D179 (provisório); este plano segue em D180–D183.

| Id | Decisão |
|---|---|
| **D180** | **Ações explícitas do `watch-service` não pedem confirmação.** `--install`/`--uninstall` (e `--subscribe`/`--unsubscribe`, pela mesma lógica) executam direto: o comando preparado **é** o aceite. O prompt e o `confirm()` saem; `--yes` deixa de existir (D14). |
| **D181** | **Verbosidade de ações pontuais.** O worker relata cada passo (baixar llama.cpp, baixar GGUF, escrever unidades, subir servidor, cadastrar projeto, verificar) e o CLI faz **stream do stderr** do worker (hoje `.output()` descarta o stderr no sucesso); stdout carrega **só** o envelope de dados (R20). |
| **D182** | **Reconciliação e validação de endpoint/modelo.** `--install` reconcilia (ou avisa com o comando exato) `embeddings.endpoint` e `embeddings.model` com o servidor que instalou; `--status` faz **probe** do endpoint efetivo e reporta `ok`/`fora`/`divergente`. |
| **D183** | **Supply-chain do worker.** GGUF com **SHA-256** verificado e **revisão pinada** (não `/resolve/main/`); o instalador do llama.cpp é validado (ou release pinada), nunca `curl … \| sh` cego. Alinha ao endurecimento do `install.sh` (v0.4.0). |

---

## 1. Princípios (do pedido)

1. **Zero ambiguidade.** Todo desvio é detectado e reportado com o **comando exato** para resolver;
   nada degrada em silêncio.
2. **Objetividade.** A mensagem diz o que foi feito, com números (bytes, porta, caminho, tempo).
3. **Verbosidade de ações pontuais.** `--install`/`--uninstall`/`--subscribe` relatam **cada
   passo** em stderr (logs), nunca só a pergunta/resumo; stdout segue só dados (R20).
4. **Ação explícita = aceite.** Comando preparado não pede confirmação.

---

## 2. Achados (evidência no teste real)

### 2.1 O progresso do `--install` é **engolido** (raiz da reclamação)

`commands/maintenance/watch.rs::run_worker` usa `Command::output()` e, no sucesso, devolve apenas
`stdout`; o **stderr** (onde o script escreve `log()`) é **descartado**. Por isso o usuário vê só
a pergunta e `watch-service install: ok` — nenhum log de baixar/instalar/subir. O script **já
registra** os passos (`instalando llama.cpp…`, `baixando modelo GGUF…`, `instalado: …`), mas eles
morrem no buffer. O **stream** canônico é **E18/T01** (wrapper fino); este épico entrega a
**lógica/robustez** do worker.

### 2.2 Divergência de endpoint (degradação silenciosa) — P1

Worker sobe o llama.cpp na **porta 8999** (`idle.conf`); o `config.toml` do projeto aponta para
**8080** (default do schema). `kd drain --digest` com o endpoint errado:
```json
{"indexed":0,"warnings":["provedor de embeddings falhou: ... 127.0.0.1:8080: Connection refused"]}
```
E `watch-service --status` segue dizendo `servidor: ok`. O `manual_guide` (passo 5) lista os
`kd config set` exatos, mas o `--install` **não** os executa nem avisa.

### 2.3 Divergência de modelo — P2

Default do schema `embeddings.model = ibm-granite/granite-embedding-97m-multilingual-r2`; o config
global do usuário tem `sentence-transformers/msmarco-MiniLM-L12-cos-v5`; o worker baixa **granite**.
Como `EmbeddingMeta` é a **identidade** do índice (D79), o cabeçalho do `.idx/embeddings.jsonl`
registra o modelo do **config**, não o que o servidor serve → **identidade mentirosa**: trocar o
modelo no config não invalida vetores gerados por outro modelo.

### 2.4 `--status` não valida o endpoint configurado — P3

Reporta `servidor: ok` (health em `$PORT`) sem comparar com o `embeddings.endpoint` efetivo.

### 2.5 Supply-chain do worker não endurecida — P4

- `install_llama`: `curl -LsSf "$LLAMA_INSTALL_URL" | sh` (sem verificação).
- `MODEL_URL`: espelho de terceiro (`huggingface.co/mykor/…`) em `/resolve/main/` (**mutável**),
  sem **SHA-256**.
- Contraste: `install.sh` principal foi endurecido na v0.4.0 (HTTPS + checksum + `verify_binaries`).

### 2.6 `uninstall` deixa o GGUF (~100 MB) — P5

`cmd_uninstall` remove unidades/agentes + `idle.conf` + script, mas não `$MODEL`; o prompt diz
"config + binário", omitindo o modelo.

### 2.7 Polimento — P6–P10

- **P6:** `run()` materializa o script embutido em **toda** ação, inclusive `--status` (read-only).
- **P7:** `confirm()` não checa `is_terminal()`; o comentário diz "stdin não-TTY ⇒ cancela", mas
  um pipe com `s\n` prossegue. (Com D180 o `confirm()` sai — item resolvido de raiz.)
- **P8:** `--every` só é validado no caminho launchd; para systemd, `$EVERY` cru vai para
  `OnUnitActiveSec=`.
- **P9:** `--status` mostra `pending=?` quando `kd drain --status` falha, sem motivo.
- **P10:** `drain --digest` reporta `indexed=0` quando o auto-drain do `write` já indexou; a
  mensagem "nada digerido ainda" (do `--status`) vs `indexed=N` confunde.

---

## 3. Riscos e mitigação

| Risco | Mitigação |
|---|---|
| D180 (sem confirmação) faz `--install` rodar em ambiente de teste | testes passam a usar `--dry-run`/`--script` fake; `--uninstall` idem |
| D181 (stream) mistura log com stdout | stream só de **stderr**; stdout continua só o envelope (R20) |
| D182 atualiza config do usuário sem pedir | `--install` **avisa** com o comando; só `--reconcile` (ou flag explícita) altera config |
| D183 pinar modelo/revisão quebra mirrors | usar o repo oficial do modelo; documentar a revisão pinada |
| remover `--yes` quebra scripts | D14 (sem retrocompatibilidade); documentar no CHANGELOG e na matriz |
| `--uninstall` remover GGUF compartilhado | `--keep-model` (default preserva; remoção explícita) |

---

## 4. Não-objetivos

- Daemon/processo de fundo obrigatório (R16) — o worker segue opcional.
- Trocar o modelo default por um de maior qualidade (fica para E16/T09).
- `criterion`/dep nova (R43).
- Confirmação para verbos **destrutivos do corpus** (`forget`/`prune` seguem com aceite explícito,
  D112) — D180 é só para o `watch-service`.

---

## 5. Pontos em aberto (aguardando indicação do usuário)

> O usuário indicou que ainda tem pontos a acrescentar **antes** do início do código. Registrar
> aqui, um por linha, para entrarem como tarefas de E17.

- _(a preencher)_

---

## 6. Performance (herança de E15)

> O orçamento completo está no épico
> [`../implementation/21_worker_embeddings_install.md`](../implementation/21_worker_embeddings_install.md)
> §"Performance e orçamento".

- **Fora do caminho quente:** reconciliação/probe só em `--install`/`--status` explícitos; nunca
  no auto-drain. O `--status` faz **um** probe (`/health` curto).
- **Invariante:** e2e `--no-idle` de `ask`/`prime`/`rewind` **inalterado**; o binário só invoca o
  script (D184).
