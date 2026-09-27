# Embeddings — busca semântica (opcional)

## O que é

O `kd ask` funciona **sem embeddings** (BM25 + âncoras + RRF). A busca semântica é um canal
**derivado** que melhora perguntas em linguagem natural (paráfrases, sinônimos). Sem provedor, o
`ask` degrada para lexical com `warnings[]`.

Se você só quer usar o knudge, **pule este guia**. Ele é necessário apenas para busca semântica.

## Em 30 segundos

```bash
kd drain service --install    # baixa llama.cpp + GGUF, sobe servidor + worker
kd ask "como o servidor não vê o conteúdo das notas"
```

`--install` age direto (ação explícita = aceite, D180; `--no-deps` não baixa dependências).

## Nível 1 — instalação manual do provedor

### 1.1 Instalar o `llama.cpp`

```bash
curl -LsSf https://llama.app/install.sh | sh
```

Ou por gerenciador: `brew install llama.cpp`, `winget install --id ggml.llamacpp -e`,
`scoop install llama.cpp`, `choco install llama.cpp`, `apt install llama.cpp`,
`dnf install llama.cpp`, `pacman -S llama.cpp`.

### 1.2 Baixar o modelo (GGUF)

Modelo recomendado: **`ibm-granite/granite-embedding-97m-multilingual-r2`** (384d, Apache-2.0,
multilíngue com PT).

```bash
mkdir -p ~/.config/local/knudge
wget -O ~/.config/local/knudge/granite-97m-r2-Q8_0.gguf \
  https://huggingface.co/mykor/granite-embedding-97m-multilingual-r2-GGUF/resolve/45ce642d3fab2033d167ec09641a159010f7d9d9/granite-embedding-97M-multilingual-r2-Q8_0.gguf
# confira o SHA-256 (revisão pinada — D183):
sha256sum ~/.config/local/knudge/granite-97m-r2-Q8_0.gguf
# 25155b89638e501ac33495fa278d551d7545e1e2f62722a499bba1f064c080f2
```

Sem `wget`? Use `curl -fL -o <destino> <url>`. O GGUF mora **ao lado do `config.toml` global**.
O `--install` faz o download de uma **revisão pinada** e **verifica o SHA-256** antes de usar.

### 1.3 Subir o servidor e configurar

```bash
llama serve \
  -m ~/.config/local/knudge/granite-97m-r2-Q8_0.gguf \
  --embeddings --pooling mean -b 2048 -ub 2048 \
  --host 127.0.0.1 --port 8084

kd config set --key embeddings.provider --value http
kd config set --key embeddings.model --value ibm-granite/granite-embedding-97m-multilingual-r2
kd config set --key embeddings.dimensions --value 384
kd config set --key embeddings.endpoint --value http://127.0.0.1:8084/v1/embeddings

kd drain --digest
```

> ⚠️ O `-ub` (µbatch físico) do `llama.cpp` é **512** por default e o `/v1/embeddings` rejeita a
> nota inteira acima disso — o drain então falha com `indexed=0`. Suba sempre com **`-ub 2048`**
> (≥ o maior corpo de nota).

## Nível 2 — fila e modos

Notas novas ficam `pending`; com `embeddings.mode=lazy` (default), o CLI drena **um lote** ao fim
de cada comando (auto-drain ocioso). O `kd drain --digest` esvazia o resto.

```bash
kd drain --status     # estado da fila (pending)
kd drain --digest      # drena um lote agora (repita para mais)
```

`mode=manual` só drena sob `--digest` explícito. Para desligar o canal:

```bash
kd config set --key recall.semantic --value false
```

## Nível 3 — cache versionado e multi-dev (D148/D153)

O vetor é função pura de `(modelo, body_hash)`: o cache de embeddings é **versionável** (opt-in).

```bash
kd config set --key embeddings.version_cache --value true
```

Com `version_cache=true`, o cache mora em **`.knudge/emb_cache.jsonl`** (fora do `.idx/`, que segue
100% derivado), recebe `merge=union` no `.gitattributes` e fica **sem eviction/TTL**. Um clone com
o mesmo modelo reconstrói o índice de notas + cache **sem chamar o modelo**:

```bash
git pull                       # notas/ + eventos/ + .knudge/emb_cache.jsonl (union)
kd drain --digest    # reindexa do cache, zero inferência
```

A chave lógica é `(body_hash, model)`: entradas de outro modelo são ignoradas no *lookup* (rede de
segurança para troca acidental) e a mesma chave com vetores divergentes desempata por `created_ms`
(nunca "o primeiro do arquivo"). Premissa: os devs usam **o mesmo modelo e as mesmas configs**.

## Nível 4 — worker persistente (systemd/launchd)

O `kd drain service` instala um agendador de usuário que mantém o servidor
**persistente** e drena a fila periodicamente.

```bash
kd drain service --install      # agendador + servidor + cadastra este projeto
kd drain service --subscribe    # cadastra outro projeto (multi-projeto)
kd drain service --status       # saúde (default)
kd drain service --uninstall    # remove agendador + servidor (preserva o GGUF)
```

- `--install` **baixa `llama.cpp` e o GGUF se faltarem**; `--no-deps` pula.
- **Supply-chain (D183):** o GGUF vem de **revisão pinada** (nunca `/resolve/main/`) com
  **SHA-256 verificado**; o instalador oficial do `llama.cpp` é verificado por SHA-256 **antes** de
  executar (senão cai para o gestor de pacotes). `kd drain service --install --dry-run` mostra a
  URL, a revisão e o hash.
- `--uninstall` **preserva o GGUF** por padrão; `--remove-model` move-o para o lixo recuperável
  (nunca `rm`); `--keep-model` documenta o default.
- O servidor sobe como `knudge-embed.service` (systemd) / `local.knudge.embed.plist` (launchd),
  com `-b 2048 -ub 2048`.
- Sem `systemd`/`launchd`, o script imprime a linha de cron equivalente.
- **Reconciliação (D182):** `--install` avisa se `embeddings.endpoint`/`model` divergirem do
  worker (mostrando o `kd config set …` exato); `kd drain service --reconcile` alinha e reindexa.
  `kd drain service --status` reporta `endpoint: ok | fora | divergente (config=…, worker=…)`.

### Suporte por SO (D185)

O **wrapper** é cross-platform; o **script embutido** (`knudge-idle.sh`) é Unix.

| SO | Shell do wrapper | Worker |
|---|---|---|
| Linux (Arch/Ubuntu/Fedora/…) | `bash` | `kd drain service --install` (systemd `--user`) |
| macOS | `bash` | `kd drain service --install` (launchd) |
| Windows | PowerShell | embutido é Unix → use `--script worker.ps1` ou o caminho manual |

No Windows, `kd drain service` com o script embutido **recusa (exit 2)** e aponta este guia.
Para automatizar, forneça um script `.ps1`:

```powershell
# worker.ps1 (exemplo): sobe o llama.cpp e drena a fila
$ErrorActionPreference = "Stop"
$gguf = "$env:USERPROFILE\.config\knudge\model.gguf"
Start-Process llama-server -ArgumentList "--model", $gguf, "--port", "8889", "-b", "2048", "-ub", "2048"
kd drain --digest
```

```powershell
kd drain service --status --script .\worker.ps1
```

## Configuração de embeddings

| Chave | Default | Para quê |
|---|---|---|
| `embeddings.provider` | `http` | `http`/`lightweight`/`none` |
| `embeddings.model` / `embeddings.revision` | granite / `main` | Identidade do modelo |
| `embeddings.dimensions` | `384` | Dimensão do vetor |
| `embeddings.similarity` | `cosine` | Métrica |
| `embeddings.mode` | `lazy` | `lazy`/`manual` |
| `embeddings.batch` / `embeddings.max_pending` | `32` / `1000` | Lote e backpressure |
| `embeddings.cache` / `version_cache` | `true` / `false` | Cache (versionado ou não) |
| `embeddings.cache_max_bytes` / `cache_ttl_days` | 32 MiB / `30` | Teto/TTL (não-versionado) |
| `embeddings.endpoint` / `timeout_ms` / `retries` | `:8889` / 30000 / `2` | Provedor HTTP |
| `recall.semantic` / `semantic_weight` / `semantic_top_k` | `true` / `30.0` / `50` | Canal vetorial |

## Solução de problemas

| Sintoma | Causa provável | Ação |
|---|---|---|
| `indexed=0` no drain | `-ub` default (512) | Suba o llama com `-ub 2048` |
| `warnings[]` "provedor inalcançável" | servidor fora do ar | `kd drain service --status`; suba o servidor |
| Notas longas presas em `pending` | `-ub` pequeno ou nota grande | Reinicie com `-b 2048 -ub 2048` |
| `Connection refused` no `--digest` | nenhum servidor no endpoint | Configure/instale o servidor persistente |

Se um provedor cair no meio do drain, o knudge **não** trava a fila: mantém as notas `pending` e
emite **um** warning.

## Próximo passo

➡️ [`kd map`](07-knowledge.md) · [`kd maintenance`](09-maintenance.md)
