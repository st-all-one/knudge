# 18 · Embeddings — busca semântica (opcional)

## O que é

O `kd ask` funciona **sem embeddings** (busca textual + âncoras). A busca semântica é um canal
extra que melhora perguntas em linguagem natural (paráfrases, sinônimos). Sem provedor, o `ask`
degrada para textual, com um aviso.

> Se você só quer usar o knudge, **pule este guia**. Ele é necessário apenas para a busca semântica.

## Quando usar

- **Use** quando as pessoas perguntam com palavras diferentes das notas ("o servidor não vê o
  conteúdo" ↔ "conteúdo isolado").
- **Não use** se o seu corpus é pequeno e você busca por termos exatos — a busca textual basta.

---

## 1. Instalação rápida (recomendada): worker + servidor

Um comando instala tudo e mantém o servidor no ar:

```bash
kd drain service --install
```

O que ele faz:

- baixa o `llama.cpp` e o modelo GGUF **se faltarem** (revisão pinada + SHA-256 verificado);
- sobe o servidor de embeddings como serviço de usuário (`systemd --user` no Linux, `launchd` no
  macOS), escutando em `127.0.0.1:8889`;
- cadastra este projeto para o drain automático periódico;
- mostra o plano com `--dry-run` e recusa instalar sem `systemd`/`launchd` (imprime a linha de
  `cron` equivalente).

```bash
# 1. instalar
kd drain service --install

# 2. ver a saúde
kd drain service --status

# 3. indexar o que ficou pendente
kd drain --digest
```

Administração do worker:

```bash
kd drain service --subscribe    # cadastrar outro projeto
kd drain service --unsubscribe  # descadastrar este projeto
kd drain service --reconcile    # alinhar endpoint/modelo ao worker
kd drain service --uninstall    # remover (preserva o GGUF)
```

---

## 2. Instalação manual (sem worker)

Útil para servidor único, container ou quando você quer controlar o processo.

### 2.1 Instalar o `llama.cpp`

```bash
curl -LsSf https://llama.app/install.sh | sh
# ou pelo gerenciador de pacotes:
brew install llama.cpp
winget install --id ggml.llamacpp -e
scoop install llama.cpp
apt install llama.cpp
dnf install llama.cpp
```

### 2.2 Baixar o modelo (GGUF)

Modelo recomendado: `granite-embedding-97m-multilingual-r2` (384 dimensões, multilíngue com PT).

```bash
mkdir -p ~/.config/local/knudge
wget -O ~/.config/local/knudge/granite-97m-r2-Q8_0.gguf \
  https://huggingface.co/mykor/granite-embedding-97m-multilingual-r2-GGUF/resolve/45ce642d3fab2033d167ec09641a159010f7d9d9/granite-embedding-97M-multilingual-r2-Q8_0.gguf
sha256sum ~/.config/local/knudge/granite-97m-r2-Q8_0.gguf
# esperado: 25155b89638e501ac33495fa278d551d7545e1e2f62722a499bba1f064c080f2
```

O GGUF fica **ao lado do `config.toml` global**.

### 2.3 Subir o servidor e apontar o projeto

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

### 2.4 Desligar quando quiser

```bash
kd config set --key recall.semantic --value false
```

---

## 3. Fila e modos

Notas novas ficam **pendentes**. Com `embeddings.mode=lazy` (padrão), o CLI indexa um lote ao fim de
cada comando; o `kd drain --digest` esvazia o resto.

```bash
# 1. o que está pendente
kd drain --status

# 2. indexar agora
kd drain --digest

# 3. refazer do zero (último recurso)
kd drain --digest --force
```

Com `mode=manual`, só o `--digest` explícito indexa.

---

## 4. Cache e trabalho em equipe

O vetor de uma nota é uma função pura do `(modelo, conteúdo)`. Por isso o cache de embeddings pode
ser **versionado**, para que um clone reindexe **sem chamar o modelo**.

```bash
# 1. versionar o cache
kd config set --key embeddings.version_cache --value true

# 2. commitá-lo junto com as notas
kd sync --message "notas + cache"

# 3. num clone novo, reindexar sem inferência
git pull
kd drain --digest
```

A chave lógica é `(conteúdo, modelo)`: vetores de outro modelo são ignorados. Premissa do trabalho
em equipe: **todos usam o mesmo modelo e as mesmas configurações**.

---

## 5. Suporte por sistema operacional

O **wrapper** do worker é multiplataforma; o **script embutido** é Unix.

| SO | Shell | Worker |
|---|---|---|
| Linux (Arch/Ubuntu/Fedora/…) | `bash` | `kd drain service --install` (systemd `--user`) |
| macOS | `bash` | `kd drain service --install` (launchd) |
| Windows | PowerShell | embutido é Unix → use `--script worker.ps1` ou a §2 |

No Windows, `kd drain service` com o script embutido recusa (exit 2) e aponta este guia. Para
automatizar, forneça um script `.ps1`:

```powershell
# worker.ps1 (exemplo): sobe o servidor e drena a fila
$ErrorActionPreference = "Stop"
$gguf = "$env:USERPROFILE\.config\knudge\model.gguf"
Start-Process llama-server -ArgumentList "--model", $gguf, "--port", "8889", "-b", "2048", "-ub", "2048"
kd drain --digest
```

```powershell
kd drain service --status --script .\worker.ps1
```

---

## 6. Configuração de embeddings

| Chave | Default | Para quê |
|---|---|---|
| `embeddings.provider` | `http` | `http`/`lightweight`/`none` |
| `embeddings.model` | granite | Identidade do modelo |
| `embeddings.dimensions` | `384` | Dimensão do vetor |
| `embeddings.similarity` | `cosine` | Métrica |
| `embeddings.mode` | `lazy` | `lazy`/`manual` |
| `embeddings.batch` / `embeddings.max_pending` | `32` / `1000` | Lote e limite da fila |
| `embeddings.cache` / `version_cache` | `true` / `false` | Cache (versionado ou não) |
| `embeddings.endpoint` / `timeout_ms` / `retries` | `:8889` / 30000 / `2` | Servidor HTTP |
| `recall.semantic` / `semantic_weight` | `true` / `30.0` | Canal semântico |

---

## 7. Solução de problemas

| Sintoma | Causa provável | Ação |
|---|---|---|
| `indexed=0` no drain | `-ub` pequeno (512) | suba o servidor com `-ub 2048` |
| Aviso "provedor inalcançável" | servidor fora do ar | `kd drain service --status`; suba o servidor |
| Notas longas presas em `pending` | `-ub` pequeno ou nota grande | reinicie com `-b 2048 -ub 2048` |
| `Connection refused` no `--digest` | nada escutando em `:8889` | `kd drain service --install` ou §2 |
| Worker não instala | sem `systemd`/`launchd` | use a linha de `cron` impressa ou a §2 |
| Sem resultados semânticos | canal desligado | `kd config set --key recall.semantic --value true` |

Se o provedor cair no meio do drain, o knudge **não** trava a fila: mantém as notas pendentes e
emite **um** aviso.

---

## 8. Veja também

➡️ [`kd drain`](12_drain.md) · [`kd config`](13_config.md) · [`kd ask`](05_ask.md) ·
[Quickstart](00_quickstart.md#7-embeddings-busca-semântica--opcional)
