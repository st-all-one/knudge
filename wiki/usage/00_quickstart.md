# 00 · Quickstart

Do zero a uma memória de projeto funcionando em cinco minutos: instalar, fundar, gravar, buscar e
retomar. No fim desta página você terá o `kd` no PATH, um corpus criado e saberá o ciclo básico.

> **O que você instala:** dois binários — `kd` (a CLI) e `knudge-mcp` (o servidor para agentes).
> A memória vive **dentro de cada projeto**, em `.knudge/`. Nada é enviado para a nuvem.

---

## 0. Quando adotar (sinceramente)

O knudge **não é para todo projeto**. Ele cobra uma disciplina — *buscar antes de gravar* — e só
compensa quando há conhecimento que se acumula e um agente (ou você mesmo) que precisa retomá-lo.
A tabela abaixo é honesta: há casos em que ele é exagero.

| Seu caso | Recomendação | Por quê |
|---|---|---|
| Projeto de vida longa (meses/anos) com agente de IA | **Adote** | O conhecimento se acumula; a retomada compensa a disciplina |
| Você perde tempo redescobrindo a mesma coisa a cada sessão | **Adote** | `kd ask` responde em uma linha o que já se sabe |
| Decisões e fatos espalhados em conversas que ninguém relê | **Adote** | Viram notas versionadas, com o porquê e a evidência |
| Repositório já no git | **Adote** | O corpus versiona junto, sem infraestrutura nova |
| Código que muda e conhecimento que envelhece | **Adote** | Ciclo de vida e drift aposentam o que ficou obsoleto |
| Time pequeno (ou solo) que troca de contexto o tempo todo | **Adote** | `kd rewind` reconstrói "onde eu estava" |
| Protótipo/script descartável de um dia | **Não adote** | O custo de fundar e curar não se paga |
| Corpus minúsculo e estável (< ~50 notas) que você lembra de cor | **Pense duas vezes** | Um README ou um ADR resolve, sem ferramenta |
| Ninguém usa agente de IA e você não quer manter notas | **Não adote** | A memória viraria trabalho manual sem consumidor |
| Você quer RAG sobre PDFs/documentos grandes | **Não adote** | O knudge é memória de **projeto** em Markdown, não indexador de arquivos |
| Você quer um wiki compartilhado com controle de acesso | **Não adote** | É local e versionado por git, sem ACL nem servidor |
| Projeto sem git | **Pense duas vezes** | Funciona, mas perde o versionamento e o merge — a melhor parte |
| Equipe grande editando a mesma nota ao mesmo tempo | **Pense duas vezes** | Funciona, mas exige disciplina de revisão/substituição |

### Checklist rápido

Adote se você marcar **três ou mais**:

- [ ] uso (ou vou usar) um agente de IA neste projeto;
- [ ] o projeto vai durar mais que algumas semanas;
- [ ] já perdi tempo redescobrindo algo que alguém já sabia;
- [ ] há decisões que precisam de registro e contexto;
- [ ] o projeto está no git.

> **Comece pequeno.** `kd init` e alguns `kd write` por duas semanas bastam para sentir o valor. Se
> não estiver usando, desinstalar não deixa resíduo: o corpus é só Markdown no seu repositório.

---

## 1. Requisitos

| Requisito | Para quê | Obrigatório? |
|---|---|---|
| `git` | versionar o corpus e ligar âncoras ao código | Sim, no projeto onde você usa |
| Linux, macOS ou Windows | o `kd` roda nos três | — |
| Rust 1.97+ | só se você compilar do source | Não (há binário pronto) |
| `llama.cpp` + um modelo GGUF | busca semântica (paráfrases) | Não — é opcional (§7) |

Se você só quer o essencial, **pule a §7**. O `kd ask` funciona sem embeddings.

---

## 2. Instalação rápida (Linux e macOS)

```bash
curl --proto '=https' --tlsv1.2 --show-error --fail \
  https://raw.githubusercontent.com/st-all-one/knudge/main/install.sh | bash
```

O instalador:

1. baixa o release pré-compilado e **confere o SHA-256** antes de instalar;
2. instala `kd` e `knudge-mcp` em `~/.local/bin`;
3. adiciona `~/.local/bin` ao PATH (`~/.profile`, `~/.bashrc`, `~/.zshrc`);
4. instala completions de `bash`, `zsh` e `fish`.

### Variações úteis

```bash
# Fixar uma versão exata
curl ... | VERSION=v0.5.0 bash

# Instalar em outro diretório
curl ... | INSTALL_DIR=/usr/local/bin bash

# Compilar do source (precisa de Rust 1.97+)
./install.sh --from-source
```

### Windows

Use **Git Bash** ou **WSL** com o comando acima. Para o servidor MCP e o worker de embeddings no
Windows nativo, veja a nota de SO em §7 e o [guia de embeddings](18_embeddings.md).

---

## 3. Deixar o `kd` disponível (PATH)

Se o instalador avisou que o PATH mudou, abra um shell novo ou rode:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Para valer sempre, confirme que a linha está no seu `~/.bashrc`/`~/.zshrc`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

---

## 4. Verificar a instalação

```bash
kd self version     # mostra a versão
which kd knudge-mcp # confirma que os dois estão no PATH
kd prime            # imprime o protocolo ("help da IA")
kd --help           # ajuda geral de todos os comandos
```

Resultado esperado:

```
kd 0.5.0
/home/voce/.local/bin/kd
/home/voce/.local/bin/knudge-mcp
```

---

## 5. Primeiros passos (o ciclo)

Dentro do seu projeto (na raiz do repositório git):

```bash
# 1) fundar a memória do projeto
kd init

# 2) buscar antes de gravar
kd ask "como o gateway limita requisições" --brief

# 3) gravar o que aprendeu
kd write --summary "O gateway limita 100 rps por chave" --type fact \
  --tag gateway --anchor src/gateway.rs

# 4) planejar/executar trabalho
kd task new --summary "Migrar para o schema V2" --scope epic --anchor plan/v2.md
kd task list --ready --sort impact

# 5) retomar contexto entre sessões
kd rewind --budget 2000

# 6) versionar a memória
kd sync --message "notas: decisão do rate limit"
```

O `kd init` cria a estrutura em `.knudge/`, escreve um bloco gerenciado no `AGENTS.md` do projeto e
instala a **skill do agente** em `.agents/skill/kd/SKILL.md` — assim o agente já sabe buscar antes
de gravar. Detalhes em [`03 · kd init`](03_init.md).

O ciclo central é sempre:

```
kd ask → kd write → kd task → kd sync
(buscar)  (gravar)   (executar) (commit)
```

**Regra de ouro: busque antes de gravar.** O `kd ask "<rascunho>"` evita duplicata e mostra a nota
que talvez você só precise atualizar.

---

## 6. Desinstalar

```bash
# binários + completions (não toca no corpus nem na config global)
./install.sh --uninstall

# se instalou via make:
make uninstall

# remover o worker/servidor de embeddings:
kd drain service --uninstall
```

O corpus vive em `.knudge/` de cada projeto. Desinstalar o binário **não apaga nota nenhuma**;
para aposentar conhecimento, use [`kd forget`](14_forget.md).

---

## 7. Embeddings (busca semântica) — opcional

Sem embeddings, o `kd ask` usa busca textual + âncoras e funciona bem. Com embeddings, ele também
encontra **paráfrases e sinônimos** (`configuração` ↔ `configuracao`, "servidor não vê o conteúdo"
↔ "conteúdo isolado").

### 7.1 Instalação rápida (recomendada): worker + servidor

Um comando instala o agendador, o servidor persistente e cadastra o projeto atual:

```bash
kd drain service --install
```

O que ele faz:

- baixa o `llama.cpp` e o modelo GGUF **se faltarem** (revisão pinada + SHA-256 verificado);
- sobe o servidor de embeddings como serviço de usuário (`systemd --user` no Linux, `launchd` no
  macOS), escutando em `127.0.0.1:8889`;
- cadastra este projeto para o drain automático periódico;
- mostra o plano antes de agir (`--dry-run`) e recusa instalar sem `systemd`/`launchd` (imprime a
  linha de `cron` equivalente).

Comandos do worker:

```bash
kd drain service --status       # saúde: agendador, servidor, fila por projeto
kd drain service --subscribe    # cadastra OUTRO projeto (multi-projeto)
kd drain service --unsubscribe  # descadastra este projeto (mantém o sistema)
kd drain service --reconcile    # alinha endpoint/modelo do projeto ao worker
kd drain service --uninstall    # remove agendador + servidor (preserva o GGUF)
kd drain service --install --dry-run   # só mostra o plano
```

Ver a fila e forçar a indexação:

```bash
kd drain --status          # o que está pendente
kd drain --digest          # indexa agora (repita para mais)
kd drain --digest --force  # apaga o índice derivado e refaz do zero (último recurso)
```

### 7.2 Instalação manual (sem worker)

Útil para servidor único, container ou quando você quer controlar o processo.

**a) Instalar o `llama.cpp`**

```bash
curl -LsSf https://llama.app/install.sh | sh
# ou pelo gerenciador de pacotes:
brew install llama.cpp
winget install --id ggml.llamacpp -e
scoop install llama.cpp
apt install llama.cpp
dnf install llama.cpp
```

**b) Baixar o modelo (GGUF)**

Modelo recomendado: `granite-embedding-97m-multilingual-r2` (384 dimensões, multilíngue com PT).

```bash
mkdir -p ~/.config/local/knudge
wget -O ~/.config/local/knudge/granite-97m-r2-Q8_0.gguf \
  https://huggingface.co/mykor/granite-embedding-97m-multilingual-r2-GGUF/resolve/45ce642d3fab2033d167ec09641a159010f7d9d9/granite-embedding-97M-multilingual-r2-Q8_0.gguf
sha256sum ~/.config/local/knudge/granite-97m-r2-Q8_0.gguf
# esperado: 25155b89638e501ac33495fa278d551d7545e1e2f62722a499bba1f064c080f2
```

O GGUF fica **ao lado do `config.toml` global** (`~/.config/local/knudge/`).

**c) Subir o servidor e apontar o projeto**

```bash
llama serve \
  -m ~/.config/local/knudge/granite-97m-r2-Q8_0.gguf \
  --embeddings --pooling mean -b 2048 -ub 2048 \
  --host 127.0.0.1 --port 8889

kd config set --key embeddings.provider --value http
kd config set --key embeddings.model --value ibm-granite/granite-embedding-97m-multilingual-r2
kd config set --key embeddings.dimensions --value 384
kd config set --key embeddings.endpoint --value http://127.0.0.1:8889/v1/embeddings

kd drain --digest
```

> ⚠️ **Sempre use `-ub 2048`.** O `llama.cpp` usa `512` por padrão e o endpoint rejeita notas
> longas acima disso; o drain falha com `indexed=0`. `-b 2048 -ub 2048` cobre o maior corpo de nota.

Para desligar o canal vetorial a qualquer momento:

```bash
kd config set --key recall.semantic --value false
```

Detalhes, multi-dev e solução de problemas: [guia de embeddings](18_embeddings.md).

---

## 8. Troubleshooting de instalação

| Sintoma | Causa provável | Ação |
|---|---|---|
| `kd: command not found` | PATH não atualizado | `export PATH="$HOME/.local/bin:$PATH"` e abra um shell novo |
| `permission denied` ao instalar | destino sem permissão | `INSTALL_DIR=~/.local/bin` ou use `sudo` no diretório do sistema |
| `checksum mismatch` | download corrompido | rode de novo; confirme `VERSION`/rede |
| `rustc` antigo | MSRV | instale Rust **1.97+** (`rustup update`) |
| Completions não funcionam | shell não recarregado | abra um shell novo ou `source` o arquivo de completion |
| `kd` funciona mas `knudge-mcp` não | só um binário no PATH | confirme `which knudge-mcp`; reinstale |
| `indexed=0` no `kd drain` | `-ub` pequeno (512) | suba o servidor com `-ub 2048` |
| "provedor inalcançável" | servidor de embeddings fora do ar | `kd drain service --status`; suba o servidor |
| `Connection refused` no `--digest` | nada escutando em `:8889` | `kd drain service --install` ou suba o `llama serve` |
| Worker não instala | sem `systemd`/`launchd` | use a linha de `cron` impressa ou a instalação manual (§7.2) |

Ajuda geral: [`kd doctor`](10_doctor.md) diagnostica o corpus; o
[guia de troubleshooting](19_troubleshooting.md) cobre o resto.

---

## 9. Próximo passo

➡️ [01 · Filosofia](01_filosofia.md) — entenda o que o knudge é e por quê, em linguagem simples.
