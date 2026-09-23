# 06 — Embeddings (busca semântica, opcional)

O `kd ask` funciona **sem embeddings** (BM25 + âncoras + RRF). A busca semântica é um canal
**derivado** que melhora perguntas em linguagem natural (paráfrases, sinônimos). Sem provedor,
o `ask` degrada para lexical com `warnings[]`.

Se você só quer usar o knudge, **pule este guia**. Ele é necessário apenas para busca semântica.

> **Atalho:** `kd maintenance watch-service --install` faz os passos 1–3 por você (baixa o
> `llama.cpp` e o modelo) e ainda deixa o servidor persistente + worker de auto-drain.
> `--no-deps` não baixa dependências.

## 1. Instalar o `llama.cpp`

O jeito mais simples (Linux/macOS):

```bash
curl -LsSf https://llama.app/install.sh | sh
```

Se preferir um gerenciador de pacotes:

| SO | Comando |
|---|---|
| macOS / Linux (Homebrew) | `brew install llama.cpp` |
| Windows (winget) | `winget install --id ggml.llamacpp -e` |
| Windows (Scoop) | `scoop install llama.cpp` |
| Windows (Chocolatey) | `choco install llama.cpp` |
| Debian/Ubuntu | `sudo apt install llama.cpp` (se disponível no seu repo) |
| Fedora | `sudo dnf install llama.cpp` |
| Arch | `sudo pacman -S llama.cpp` |

## 2. Baixar o modelo (GGUF)

Modelo recomendado: **`ibm-granite/granite-embedding-97m-multilingual-r2`** (384d, Apache-2.0,
multilíngue com PT).

```bash
mkdir -p ~/.config/local/knudge
wget -O ~/.config/local/knudge/granite-97m-r2-Q8_0.gguf \
  https://huggingface.co/mykor/granite-embedding-97m-multilingual-r2-GGUF/resolve/main/granite-embedding-97M-multilingual-r2-Q8_0.gguf
```

Sem `wget`? Use `curl -fL -o <destino> <url>`. O GGUF mora **ao lado do `config.toml` global**
(`~/.config/local/knudge/` no Linux).

## 3. Subir o servidor e configurar

```bash
llama serve \
  -m ~/.config/local/knudge/granite-97m-r2-Q8_0.gguf \
  --embeddings --pooling mean -b 2048 -ub 2048 \
  --host 127.0.0.1 --port 8084

kd config set embeddings.provider http
kd config set embeddings.model ibm-granite/granite-embedding-97m-multilingual-r2
kd config set embeddings.dimensions 384
kd config set embeddings.endpoint http://127.0.0.1:8084/v1/embeddings

kd maintenance index --drain
kd ask "como o servidor não vê o conteúdo das notas"
```

> ⚠️ O `-ub` (µbatch físico) do `llama.cpp` é **512** por default, e o `/v1/embeddings` rejeita a
> nota inteira acima disso — o drain então falha com `indexed=0`. Suba sempre com **`-ub 2048`**
> (≥ o maior corpo de nota).

## 4. Fila assíncrona e lazy

Notas novas ficam `pending`; com `embeddings.mode=lazy` (default), o CLI drena **um lote** ao fim
de cada comando (auto-drain ocioso). O `kd maintenance index --drain` esvazia o resto.

```bash
kd maintenance index --status     # estado da fila (pending)
kd maintenance index --drain      # drena um lote agora (repita para mais)
```

`mode=manual` só drena sob `--drain` explícito. Para desligar o canal:
`kd config set recall.semantic false`.

## 5. Worker de auto-drain (quando você não usa o `kd`)

O `kd maintenance watch-service` instala um agendador de usuário (`systemd --user` no Linux,
`launchd` no macOS) que mantém o servidor **persistente** e drena a fila periodicamente.

```bash
kd maintenance watch-service --install      # agendador + servidor + cadastra este projeto
kd maintenance watch-service --subscribe    # cadastra outro projeto (multi-projeto)
kd maintenance watch-service --unsubscribe  # descadastra (mantém o sistema)
kd maintenance watch-service --status       # saúde (default)
kd maintenance watch-service --uninstall    # remove agendador + servidor
```

- `--install` **baixa `llama.cpp` e o GGUF se faltarem** (script oficial + fallback para
  `brew`/`winget`/`scoop`/`choco`/`apt`/`dnf`/`pacman`/`zypper`); `--no-deps` pula.
- O servidor sobe como `knudge-embed.service` (systemd) / `local.knudge.embed.plist` (launchd),
  com `-b 2048 -ub 2048` — o `--drain` manual e o auto-drain ocioso sempre o encontram.
- Sem `systemd`/`launchd`, o script imprime a linha de cron equivalente.

## Solução de problemas

| Sintoma | Causa provável | Ação |
|---|---|---|
| `indexed=0` no drain | `-ub` default (512) | Suba o llama com `-ub 2048` |
| `warnings[]` "provedor inalcançável" | servidor fora do ar | `kd maintenance watch-service --status`; suba o servidor |
| Notas longas presas em `pending` | `-ub` pequeno ou nota grande | Reinicie o servidor com `-b 2048 -ub 2048` |
| `Connection refused` no `--drain` | nenhum servidor no endpoint | Configure/instale o servidor persistente |

Se um provedor cair no meio do drain, o knudge **não** trava a fila: mantém as notas `pending`
e emite **um** warning (R33/D83).

## Próximo passo

➡️ [Manutenção e handoff](07-manutencao.md)
