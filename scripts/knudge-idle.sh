#!/usr/bin/env bash
# knudge-idle — worker de auto-drain ocioso e gerenciador do agendador (E11-T03/D131/D132/D133).
#
# O knudge não tem daemon: em `embeddings.mode=lazy` (default) o `kd` drena **um lote** no fim
# de cada invocação. Este script instancia um worker contínuo e seguro para quando você não usa
# o `kd` por muito tempo: mantém o servidor de embeddings local de pé e drena a fila de **cada
# projeto cadastrado** até `pending = 0`.
#
# Agendador: `systemd --user` (Linux) ou `launchd` (macOS). Sem nenhum dos dois, imprime a linha
# de cron equivalente. O servidor de embeddings roda como unidade/agente **persistente**
# (`knudge-embed`), então o `--drain` manual e o auto-drain lazy sempre o encontram; o worker
# só drena (e sobe um servidor efêmero de fallback se o persistente estiver fora).
#
# Uso:
#   knudge-idle install [--project DIR] [--port N] [--model PATH] [--every DUR] [--no-deps] [--dry-run]
#   knudge-idle subscribe [--project DIR]      # cadastra um projeto (multi-projeto)
#   knudge-idle unsubscribe [--project DIR]    # descadastra (mantém o sistema instalado)
#   knudge-idle status                         # saúde: agendador, servidor, fila por projeto
#   knudge-idle uninstall [--keep-model|--remove-model]  # remove o sistema; preserva o GGUF por padrão
#   knudge-idle run [PROJETO...]               # corpo do worker (o agendador chama isto)
#
# O install baixa o llama.cpp (instalador oficial **verificado por SHA-256**; fallback para
# brew/winget/scoop/choco/apt/dnf/pacman/zypper) e o GGUF (revisão pinada + SHA-256) se ausentes;
# --no-deps pula. Supply-chain em D183.
#
# Segurança: idempotente; nunca usa `rm` (move para ${XDG_CACHE_HOME:-~/.cache}/knudge/trash).
# O GGUF mora ao lado do config.toml global (${XDG_CONFIG_HOME:-~/.config}/local/knudge/).
set -uo pipefail

PROG="knudge-idle"
SELF="${BASH_SOURCE[0]:-}"
XDG_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}"
XDG_CACHE="${XDG_CACHE_HOME:-$HOME/.cache}"
CONF_DIR="$XDG_CONFIG/local/knudge"
CONF="$CONF_DIR/idle.conf"
UNIT_DIR="$XDG_CONFIG/systemd/user"
UNIT="knudge-idle"
EMBED_UNIT="knudge-embed"
LA_DIR="$HOME/Library/LaunchAgents"
LA_IDLE_LABEL="local.knudge.idle"
LA_EMBED_LABEL="local.knudge.embed"
LA_IDLE_PLIST="$LA_DIR/$LA_IDLE_LABEL.plist"
LA_EMBED_PLIST="$LA_DIR/$LA_EMBED_LABEL.plist"
BIN_DIR="${KNUDGE_BIN_DIR:-$HOME/.local/bin}"
TRASH="$XDG_CACHE/knudge/trash"
KD="${KNUDGE_KD:-$BIN_DIR/kd}"
LLAMA="${KNUDGE_LLAMA:-$BIN_DIR/llama}"
DEFAULT_MODEL="$CONF_DIR/granite-97m-r2-Q8_0.gguf"
MODEL_ID="ibm-granite/granite-embedding-97m-multilingual-r2"
DEFAULT_PORT=8889
DEFAULT_EVERY=1h
LOG_DIR="$XDG_CACHE/knudge"
LOG="$LOG_DIR/idle.log"
SCHEDULER="" # systemd | launchd | none
# Supply-chain (D183): revisão **pinada** (nunca `/resolve/main/`) e SHA-256 do GGUF;
# instalador do llama.cpp **verificado** (nunca `curl … | sh` cego).
MODEL_REVISION="45ce642d3fab2033d167ec09641a159010f7d9d9"
MODEL_FILE="granite-embedding-97M-multilingual-r2-Q8_0.gguf"
MODEL_SHA256="25155b89638e501ac33495fa278d551d7545e1e2f62722a499bba1f064c080f2"
LLAMA_INSTALL_URL="https://llama.app/install.sh"
LLAMA_INSTALL_SHA256="cccdfcbd1b55bf6003ac3037588c9f5b3b79aa0a75fe991e97bb218ccdb55e4d"
MODEL_URL="https://huggingface.co/mykor/granite-embedding-97m-multilingual-r2-GGUF/resolve/$MODEL_REVISION/$MODEL_FILE"

log() { printf '%s: %s\n' "$PROG" "$*" >&2; }
warn() { printf '%s: aviso: %s\n' "$PROG" "$*" >&2; }
next_cmd() { printf '  próximos: %s\n' "$*" >&2; }
die() {
    log "$*"
    exit 1
}

trash() {
    [ -e "$1" ] || return 0
    mkdir -p "$TRASH"
    mv "$1" "$TRASH/$(basename "$1").$(date +%Y%m%d%H%M%S).$$" 2>/dev/null || true
}

# SHA-256 em hex (supply-chain, D183): usa o primeiro verificador disponível.
sha256_of() {
    local file=$1
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$file" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$file" | awk '{print $1}'
    elif command -v openssl >/dev/null 2>&1; then
        openssl dgst -sha256 "$file" | awk '{print $NF}'
    else
        return 1
    fi
}

# Compara o SHA-256 de um arquivo com o esperado (vazio ⇒ falha: nunca confia sem hash).
verify_sha256() {
    local file=$1 expected=$2 got
    [ -n "$expected" ] || return 1
    got="$(sha256_of "$file")" || return 1
    [ "$got" = "$expected" ]
}

# ---------- guia manual (ambiente inválido / falha) ----------
# Impresso quando não há agendador, o llama.cpp/GGUF não podem ser instalados ou o servidor não
# sobe. Cobre Windows, macOS, Ubuntu, Fedora, Arch e o fallback sem agendador.
manual_guide() {
    cat >&2 <<EOF
--- como subir o servidor de embeddings manualmente ---
1) Instale o llama.cpp (escolha um):
   Linux/macOS (oficial) : curl -LsSf https://llama.app/install.sh | sh
   macOS (Homebrew)      : brew install llama.cpp
   Ubuntu/Debian         : sudo apt update && sudo apt install -y llama.cpp
   Fedora                : sudo dnf install -y llama.cpp
   Arch                  : sudo pacman -S --noconfirm llama.cpp
   Windows (winget)      : winget install --id ggml.llamacpp -e
   Windows (Scoop)       : scoop install llama.cpp
   Windows (Chocolatey)  : choco install -y llama.cpp

2) Baixe o modelo GGUF (~110 MB) para $MODEL:
   curl -fL -o "$MODEL" \\
     $MODEL_URL
   (sem curl: wget -O "$MODEL" "$MODEL_URL")
   Confira o SHA-256 (revisão pinada $MODEL_REVISION):
     $MODEL_SHA256

3) Suba o servidor em primeiro plano (teste):
   llama serve -m "$MODEL" --embeddings --pooling mean -b 2048 -ub 2048 --host 127.0.0.1 --port $PORT
   O -ub 2048 evita que notas longas falhem o /v1/embeddings.

4) Deixe-o persistente (escolha um):
   Linux (systemd --user): crie ~/.config/systemd/user/knudge-embed.service:
     [Unit]
     Description=knudge: servidor de embeddings
     After=network.target
     [Service]
     ExecStart=$LLAMA serve -m $MODEL --embeddings --pooling mean -b 2048 -ub 2048 --host 127.0.0.1 --port $PORT
     Restart=on-failure
     RestartSec=5
     [Install]
     WantedBy=default.target
   depois: systemctl --user daemon-reload && systemctl --user enable --now knudge-embed.service
   macOS (launchd): crie ~/Library/LaunchAgents/local.knudge.embed.plist apontando para o llama
     (modelo em wiki/usage/18_embeddings.md) e carregue: launchctl bootstrap gui/\$(id -u) <plist>
   Windows: atalho/.bat no Startup do usuario ou tarefa no Agendador de Tarefas:
     llama.exe serve -m "%USERPROFILE%\\granite-97m-r2-Q8_0.gguf" --embeddings --pooling mean -b 2048 -ub 2048 --port $PORT
   Sem agendador (qualquer SO):
     nohup llama serve -m "$MODEL" --embeddings --pooling mean -b 2048 -ub 2048 --host 127.0.0.1 --port $PORT >> "$LOG_DIR/embed.log" 2>&1 &

5) Aponte o kd e valide:
   kd config set --key embeddings.provider --value http
   kd config set --key embeddings.endpoint --value http://127.0.0.1:$PORT/v1/embeddings
   kd config set --key embeddings.model --value ibm-granite/granite-embedding-97m-multilingual-r2
   kd config set --key embeddings.dimensions --value 384
   kd drain --digest
   kd drain service --status

Guia completo por SO: wiki/usage/18_embeddings.md
EOF
}

die_with_guide() {
    log "$*"
    manual_guide
    exit 1
}

# ---------- config (multi-projeto) ----------
PORT="$DEFAULT_PORT"
MODEL="$DEFAULT_MODEL"
EVERY="$DEFAULT_EVERY"
PROJECTS=()

load_conf() {
    [ -r "$CONF" ] || return 0
    local k v
    while IFS='=' read -r k v; do
        case "$k" in
            PORT) [ -n "$v" ] && PORT="$v" ;;
            MODEL) [ -n "$v" ] && MODEL="$v" ;;
            EVERY) [ -n "$v" ] && EVERY="$v" ;;
            PROJECT) [ -n "$v" ] && PROJECTS+=("$v") ;;
        esac
    done < <(grep -E '^(PORT|MODEL|EVERY|PROJECT)=' "$CONF" 2>/dev/null)
}

write_conf() {
    mkdir -p "$CONF_DIR"
    {
        printf '# gerado por knudge-idle (%s)\n' "$(date +%Y-%m-%d)"
        printf 'PORT=%s\nMODEL=%s\nEVERY=%s\n' "$PORT" "$MODEL" "$EVERY"
        local p
        for p in "${PROJECTS[@]}"; do
            [ -n "$p" ] && printf 'PROJECT=%s\n' "$p"
        done
    } >"$CONF"
}

has_project() {
    local want=$1 p
    for p in "${PROJECTS[@]}"; do
        [ "$p" = "$want" ] && return 0
    done
    return 1
}

remove_project() {
    local want=$1 kept=() p
    for p in "${PROJECTS[@]}"; do
        [ "$p" = "$want" ] || kept+=("$p")
    done
    PROJECTS=("${kept[@]}")
}

# ---------- reconciliação de config (D182) ----------
# Extrai o valor de uma linha `chave = valor (escopo) — caminho` do `kd config get`
# (remove as aspas que o render TOML adiciona a textos).
extract_value() { sed -n 's/^[^=]*= *\([^ ]*\).*/\1/p' | head -n1 | sed 's/^"//; s/"$//'; }

# Valor efetivo de uma chave: projeto > global > vazio (o kd usa o default embutido).
effective_config() {
    local proj=$1 key=$2 v
    v="$(cd "$proj" 2>/dev/null && "$KD" config get --key "$key" 2>/dev/null | extract_value)"
    [ -n "$v" ] && { printf '%s\n' "$v"; return 0; }
    v="$(cd "$proj" 2>/dev/null && "$KD" config get --key "$key" --global 2>/dev/null | extract_value)"
    [ -n "$v" ] && { printf '%s\n' "$v"; return 0; }
    return 0
}

# Compara endpoint/model efetivos com o worker instalado (D182). Divergência ⇒ aviso com o
# comando exato; `apply=1` (--reconcile) aplica no config do projeto.
reconcile_config() {
    local proj=$1 apply=$2
    local want_endpoint="http://127.0.0.1:$PORT/v1/embeddings" want_model="$MODEL_ID"
    local cur_endpoint cur_model drift=0
    cur_endpoint="$(effective_config "$proj" embeddings.endpoint)"
    cur_model="$(effective_config "$proj" embeddings.model)"
    if [ "$cur_endpoint" != "$want_endpoint" ]; then
        drift=1
        warn "endpoint do kd difere do worker: config='${cur_endpoint:-<default>}' worker='$want_endpoint'"
        next_cmd "kd config set --key embeddings.endpoint --value $want_endpoint"
    fi
    if [ "$cur_model" != "$want_model" ]; then
        drift=1
        warn "modelo do kd difere do worker: config='${cur_model:-<default>}' worker='$want_model'"
        next_cmd "kd config set --key embeddings.model --value $want_model"
    fi
    if [ "$drift" -eq 0 ]; then
        log "config do kd alinhada ao worker (endpoint e modelo)"
        return 0
    fi
    if [ "$apply" = "1" ]; then
        (cd "$proj" && "$KD" config set --key embeddings.endpoint --value "$want_endpoint") >&2 || true
        (cd "$proj" && "$KD" config set --key embeddings.model --value "$want_model") >&2 || true
        log "config reconciliada; rode 'kd drain --force' para reindexar com o modelo novo"
    else
        log "para alinhar agora: kd drain service --install --reconcile"
    fi
    return 0
}

# Probe do endpoint efetivo (D182): `ok`/`fora`/`divergente` (config ≠ worker).
probe_endpoint() {
    local proj=$1 cur base worker
    cur="$(effective_config "$proj" embeddings.endpoint)"
    [ -n "$cur" ] || cur="http://127.0.0.1:8889/v1/embeddings"
    base="${cur%/v1/embeddings}"
    worker="http://127.0.0.1:$PORT/v1/embeddings"
    if [ "$cur" != "$worker" ]; then
        printf 'endpoint: divergente (config=%s, worker=%s)\n' "$cur" "$worker"
        return 0
    fi
    if curl -fsS "$base/health" >/dev/null 2>&1; then
        printf 'endpoint: ok (%s)\n' "$cur"
    else
        printf 'endpoint: fora (%s)\n' "$cur"
    fi
}

# ---------- agendador ----------
detect_scheduler() {
    if command -v systemctl >/dev/null 2>&1 && systemctl --user show-environment >/dev/null 2>&1; then
        SCHEDULER="systemd"
    elif command -v launchctl >/dev/null 2>&1; then
        SCHEDULER="launchd"
    else
        SCHEDULER="none"
    fi
}

# Converte `30m`/`1h`/`1d`/`45s`/`90` em segundos (para o `StartInterval` do launchd).
duration_seconds() {
    local raw=$1 digits unit
    digits="${raw//[!0-9]/}"
    [ -n "$digits" ] || die "duração inválida: $raw (use 30m, 1h, 1d)"
    unit="${raw//[0-9]/}"
    case "$unit" in
        "" | s) printf '%s\n' "$digits" ;;
        m) printf '%s\n' "$((digits * 60))" ;;
        h) printf '%s\n' "$((digits * 3600))" ;;
        d) printf '%s\n' "$((digits * 86400))" ;;
        *) die "duração inválida: $raw (use 30m, 1h, 1d)" ;;
    esac
}

write_systemd_units() {
    mkdir -p "$UNIT_DIR" "$LOG_DIR"
    cat >"$UNIT_DIR/$UNIT.service" <<EOF
[Unit]
Description=knudge: drena a fila de embeddings dos projetos cadastrados
Documentation=file:$CONF

[Service]
Type=oneshot
ExecStart=$BIN_DIR/$UNIT run
Nice=10
EOF
    cat >"$UNIT_DIR/$UNIT.timer" <<EOF
[Unit]
Description=knudge: drain periódico da fila de embeddings

[Timer]
# OnActiveSec arma o timer ao instalar/reiniciar; OnUnitActiveSec reagenda após cada run.
OnActiveSec=5min
OnUnitActiveSec=$EVERY
AccuracySec=1min
Persistent=true
Unit=$UNIT.service

[Install]
WantedBy=timers.target
EOF
    write_systemd_embed
}

write_systemd_embed() {
    cat >"$UNIT_DIR/$EMBED_UNIT.service" <<EOF
[Unit]
Description=knudge: servidor de embeddings local (llama.cpp)
Documentation=file:$CONF
After=network.target

[Service]
Type=simple
ExecStart=$LLAMA serve -m $MODEL --embeddings --pooling mean --host 127.0.0.1 --port $PORT -b 2048 -ub 2048
Restart=on-failure
RestartSec=5
StandardOutput=append:$LOG_DIR/embed.log
StandardError=append:$LOG_DIR/embed.log

[Install]
WantedBy=default.target
EOF
}

unify_legacy() {
    local old
    for old in knudge-drain.timer knudge-drain.service; do
        if [ -e "$UNIT_DIR/$old" ]; then
            systemctl --user disable --now "$old" >/dev/null 2>&1
            trash "$UNIT_DIR/$old"
        fi
    done
}

write_launchd_plists() {
    mkdir -p "$LA_DIR" "$LOG_DIR"
    local interval
    interval="$(duration_seconds "$EVERY")"
    cat >"$LA_IDLE_PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$LA_IDLE_LABEL</string>
    <key>ProgramArguments</key>
    <array>
        <string>$BIN_DIR/$UNIT</string>
        <string>run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>StartInterval</key>
    <integer>$interval</integer>
    <key>StandardOutPath</key>
    <string>$LOG_DIR/idle.log</string>
    <key>StandardErrorPath</key>
    <string>$LOG_DIR/idle.err.log</string>
</dict>
</plist>
EOF
    cat >"$LA_EMBED_PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$LA_EMBED_LABEL</string>
    <key>ProgramArguments</key>
    <array>
        <string>$LLAMA</string>
        <string>serve</string>
        <string>-m</string>
        <string>$MODEL</string>
        <string>--embeddings</string>
        <string>--pooling</string>
        <string>mean</string>
        <string>--host</string>
        <string>127.0.0.1</string>
        <string>--port</string>
        <string>$PORT</string>
        <string>-b</string>
        <string>2048</string>
        <string>-ub</string>
        <string>2048</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$LOG_DIR/embed.log</string>
    <key>StandardErrorPath</key>
    <string>$LOG_DIR/embed.log</string>
</dict>
</plist>
EOF
}

launchd_load() {
    local plist=$1 label=$2
    launchctl bootout "gui/$(id -u)/$label" >/dev/null 2>&1 || true
    if ! launchctl bootstrap "gui/$(id -u)" "$plist" >/dev/null 2>&1; then
        launchctl load -w "$plist" >/dev/null 2>&1 || true
    fi
}

launchd_unload() {
    local plist=$1 label=$2
    launchctl bootout "gui/$(id -u)/$label" >/dev/null 2>&1 ||
        launchctl unload -w "$plist" >/dev/null 2>&1 || true
}

scheduler_install() {
    case "$SCHEDULER" in
        systemd)
            write_systemd_units
            unify_legacy
            systemctl --user daemon-reload
            systemctl --user enable --now "$EMBED_UNIT.service" ||
                die "falha ao habilitar o servidor de embeddings"
            systemctl --user enable --now "$UNIT.timer" || die "falha ao habilitar o timer"
            ;;
        launchd)
            write_launchd_plists
            launchd_load "$LA_EMBED_PLIST" "$LA_EMBED_LABEL"
            launchd_load "$LA_IDLE_PLIST" "$LA_IDLE_LABEL"
            ;;
        *)
            die "sem systemd --user/launchd; use cron: 0 * * * * $BIN_DIR/$UNIT run"
            ;;
    esac
}

scheduler_uninstall() {
    case "$SCHEDULER" in
        systemd)
            systemctl --user disable --now "$UNIT.timer" >/dev/null 2>&1 || true
            systemctl --user disable --now "$EMBED_UNIT.service" >/dev/null 2>&1 || true
            trash "$UNIT_DIR/$UNIT.service"
            trash "$UNIT_DIR/$UNIT.timer"
            trash "$UNIT_DIR/$EMBED_UNIT.service"
            systemctl --user daemon-reload >/dev/null 2>&1 || true
            ;;
        launchd)
            launchd_unload "$LA_IDLE_PLIST" "$LA_IDLE_LABEL"
            launchd_unload "$LA_EMBED_PLIST" "$LA_EMBED_LABEL"
            trash "$LA_IDLE_PLIST"
            trash "$LA_EMBED_PLIST"
            ;;
    esac
}

scheduler_subscribe() {
    case "$SCHEDULER" in
        systemd) systemctl --user enable --now "$UNIT.timer" >/dev/null 2>&1 || true ;;
        launchd) launchd_load "$LA_IDLE_PLIST" "$LA_IDLE_LABEL" ;;
    esac
}

scheduler_unsubscribe() {
    case "$SCHEDULER" in
        systemd) systemctl --user disable --now "$UNIT.timer" >/dev/null 2>&1 || true ;;
        launchd) launchd_unload "$LA_IDLE_PLIST" "$LA_IDLE_LABEL" ;;
    esac
}

scheduler_status() {
    case "$SCHEDULER" in
        systemd)
            printf 'timer: %s\n' "$(systemctl --user is-active "$UNIT.timer" 2>/dev/null || printf inativo)"
            printf 'servidor (unit): %s\n' "$(systemctl --user is-active "$EMBED_UNIT.service" 2>/dev/null || printf inativo)"
            systemctl --user list-timers "$UNIT.timer" --no-pager 2>/dev/null | sed -n '1,2p' | sed 's/^/  /'
            ;;
        launchd)
            printf 'agente idle: %s\n' "$(launchctl print "gui/$(id -u)/$LA_IDLE_LABEL" >/dev/null 2>&1 && printf carregado || printf ausente)"
            printf 'agente servidor: %s\n' "$(launchctl print "gui/$(id -u)/$LA_EMBED_LABEL" >/dev/null 2>&1 && printf carregado || printf ausente)"
            ;;
        *) printf 'agendador: nenhum (cron manual)\n' ;;
    esac
}

# ---------- servidor ----------
# Fallback: se o servidor persistente estiver fora, sobe um efêmero (mean pooling + ubatch 2048).
# O `-ub` default do llama.cpp é 512 e rejeita notas longas (drain falha com `indexed=0`).
ensure_server() {
    local model=$1 port=$2 health="http://127.0.0.1:$port/health"
    mkdir -p "$LOG_DIR"
    if curl -fsS "$health" >/dev/null 2>&1; then
        return 0
    fi
    [ -x "$LLAMA" ] || die_with_guide "llama não encontrado em $LLAMA (defina KNUDGE_LLAMA ou veja o guia abaixo)"
    "$LLAMA" serve -m "$model" --embeddings --pooling mean \
        --host 127.0.0.1 --port "$port" -b 2048 -ub 2048 >>"$LOG" 2>&1 &
    STARTED_PID=$!
    local _i=1
    while [ "$_i" -le 60 ]; do
        curl -fsS "$health" >/dev/null 2>&1 && return 0
        sleep 1
        _i=$((_i + 1))
    done
    die_with_guide "servidor de embeddings não subiu (veja $LOG)"
}

cleanup() {
    [ -n "${STARTED_PID:-}" ] && kill "$STARTED_PID" 2>/dev/null
    return 0
}

# Drena cada projeto: `kd drain --digest` processa todos os lotes numa chamada até pending=0.
drain_all() {
    local rc=0 proj out
    for proj in "$@"; do
        # `kd drain --digest` já processa **todos** os lotes numa chamada (D170).
        out=$(cd "$proj" 2>/dev/null && "$KD" drain --digest 2>&1) || {
            log "falha no drain de $proj: $out"
            rc=1
            continue
        }
        printf '%s: %s\n' "$proj" "$out"
    done
    return $rc
}

# ---------- dependências (llama.cpp + GGUF) ----------
# Resolve o binário do llama.cpp (PATH ou ~/.local/bin) quando $LLAMA não existe.
resolve_llama() {
    [ -x "$LLAMA" ] && return 0
    local found
    found="$(command -v llama 2>/dev/null || true)"
    [ -z "$found" ] && [ -x "$HOME/.local/bin/llama" ] && found="$HOME/.local/bin/llama"
    [ -n "$found" ] && LLAMA="$found"
    return 0
}

# Baixa uma URL para um destino com curl e cai para wget (resiliente a ambientes sem curl).
# `file://` (mirror local/air-gapped) é copiado direto — não passa pelo `--proto '=https'`.
fetch() {
    local url=$1 dest=$2
    case "$url" in
        file://*) cp "${url#file://}" "$dest" && return 0 ;;
    esac
    if command -v curl >/dev/null 2>&1; then
        curl -fL --proto '=https' --tlsv1.2 -o "$dest" "$url" && return 0
    fi
    if command -v wget >/dev/null 2>&1; then
        wget -q -O "$dest" "$url" && return 0
    fi
    return 1
}

# Instala o llama.cpp: binário já presente > instalador oficial (verificado) > gestor.
install_llama() {
    resolve_llama
    [ -x "$LLAMA" ] && return 0
    if command -v curl >/dev/null 2>&1 || command -v wget >/dev/null 2>&1; then
        local tmp="$XDG_CACHE/knudge/llama-install.$$"
        local expected="${KNUDGE_LLAMA_INSTALL_SHA256:-$LLAMA_INSTALL_SHA256}" got=""
        mkdir -p "$(dirname "$tmp")"
        log "baixando instalador oficial do llama.cpp ($LLAMA_INSTALL_URL)..."
        if fetch "$LLAMA_INSTALL_URL" "$tmp"; then
            got="$(sha256_of "$tmp" || true)"
            log "  sha256: ${got:-<indisponível>} (esperado: $expected)"
            if verify_sha256 "$tmp" "$expected"; then
                log "checksum confere; executando o instalador oficial..."
                sh "$tmp" >/dev/null 2>&1 || log "instalador oficial falhou; tentando gestor de pacotes..."
                trash "$tmp"
                hash -r 2>/dev/null || true
                resolve_llama
                [ -x "$LLAMA" ] && return 0
            else
                warn "checksum do instalador não confere; NÃO executando (supply-chain, D183)"
                [ -n "$got" ] && next_cmd "confie explicitamente com: KNUDGE_LLAMA_INSTALL_SHA256=$got kd drain service --install"
                trash "$tmp"
                log "tentando gestor de pacotes..."
            fi
        else
            log "download do instalador falhou; tentando gestor de pacotes..."
        fi
    fi
    local pm
    for pm in brew winget scoop choco apt-get dnf pacman zypper; do
        command -v "$pm" >/dev/null 2>&1 || continue
        log "tentando $pm install llama.cpp..."
        case "$pm" in
            brew) brew install llama.cpp && break ;;
            winget) winget install --id ggml.llamacpp -e --accept-source-agreements --accept-package-agreements && break ;;
            scoop) scoop install llama.cpp && break ;;
            choco) choco install -y llama.cpp && break ;;
            apt-get) sudo apt-get update -qq && sudo apt-get install -y llama.cpp && break ;;
            dnf) sudo dnf install -y llama.cpp && break ;;
            pacman) sudo pacman -S --noconfirm llama.cpp && break ;;
            zypper) sudo zypper install -y llama.cpp && break ;;
        esac
    done
    hash -r 2>/dev/null || true
    resolve_llama
    [ -x "$LLAMA" ]
}

# Baixa o GGUF recomendado para $MODEL e **verifica o SHA-256** (revisão pinada; D183).
install_model() {
    [ -f "$MODEL" ] && return 0
    mkdir -p "$(dirname "$MODEL")"
    local url="${KNUDGE_MODEL_URL:-$MODEL_URL}" expected="${KNUDGE_MODEL_SHA256:-$MODEL_SHA256}"
    log "baixando modelo GGUF (~110 MB) para $MODEL..."
    log "  url: $url"
    log "  revisão: ${KNUDGE_MODEL_REVISION:-$MODEL_REVISION}"
    log "  sha256 esperado: $expected"
    fetch "$url" "$MODEL.tmp" || { trash "$MODEL.tmp"; return 1; }
    if ! verify_sha256 "$MODEL.tmp" "$expected"; then
        warn "checksum do GGUF não confere; abortando (arquivo movido para o lixo, D183)"
        trash "$MODEL.tmp"
        return 1
    fi
    mv "$MODEL.tmp" "$MODEL"
    log "GGUF verificado (sha256 ok)"
    [ -f "$MODEL" ]
}

# Garante llama.cpp + GGUF no install (pulado com --no-deps/KNUDGE_SKIP_DEPS=1).
ensure_deps() {
    if [ "${KNUDGE_SKIP_DEPS:-0}" = "1" ]; then
        [ -x "$LLAMA" ] || command -v llama >/dev/null 2>&1 || die_with_guide "llama.cpp ausente (--no-deps)"
        [ -f "$MODEL" ] || die_with_guide "GGUF ausente: $MODEL (--no-deps)"
        return 0
    fi
    install_llama || die_with_guide "não foi possível instalar llama.cpp; veja o guia abaixo ou defina KNUDGE_LLAMA"
    install_model || die_with_guide "não foi possível baixar o modelo; veja o guia abaixo"
    return 0
}

# ---------- opções ----------
OPT_PROJECT=""
OPT_PORT=""
OPT_MODEL=""
OPT_EVERY=""
parse_common_args() {
    OPT_PROJECT="$PWD"
    OPT_PORT=""
    OPT_MODEL=""
    OPT_EVERY=""
    OPT_NO_DEPS=""
    OPT_DRY_RUN=""
    OPT_KEEP_MODEL=""
    OPT_REMOVE_MODEL=""
    while [ "$#" -gt 0 ]; do
        case "$1" in
            --project)
                OPT_PROJECT="${2:?--project exige um diretório}"
                shift 2
                ;;
            --no-deps)
                OPT_NO_DEPS=1
                shift
                ;;
            --port)
                OPT_PORT="${2:?--port exige um número}"
                shift 2
                ;;
            --model)
                OPT_MODEL="${2:?--model exige um caminho}"
                shift 2
                ;;
            --every)
                OPT_EVERY="${2:?--every exige uma duração (ex.: 1h)}"
                shift 2
                ;;
            --dry-run)
                OPT_DRY_RUN=1
                shift
                ;;
            --keep-model)
                OPT_KEEP_MODEL=1
                shift
                ;;
            --remove-model)
                OPT_REMOVE_MODEL=1
                shift
                ;;
            *) die "opção desconhecida: $1" ;;
        esac
    done
}

# Verificação de pré-instalação: falha loud antes de tocar em qualquer coisa.
preflight() {
    local project=$1
    [ -x "$KD" ] || die "kd não encontrado em $KD (instale o knudge ou defina KNUDGE_KD)"
    [ -d "$project" ] || die "projeto inexistente: $project"
    ensure_deps
}

# ---------- ações ----------
cmd_install() {
    parse_common_args "$@"
    load_conf
    detect_scheduler
    [ -n "$OPT_PORT" ] && PORT="$OPT_PORT"
    [ -n "$OPT_MODEL" ] && MODEL="$OPT_MODEL"
    [ -n "$OPT_EVERY" ] && EVERY="$OPT_EVERY"
    [ -n "$OPT_NO_DEPS" ] && export KNUDGE_SKIP_DEPS=1
    # P8: valida a duração antes de escrever qualquer unit (todos os agendadores).
    duration_seconds "$EVERY" >/dev/null
    if [ -n "$OPT_DRY_RUN" ]; then
        printf 'knudge-idle install (dry-run)\n'
        printf '  projeto: %s\n' "$OPT_PROJECT"
        printf '  porta: %s\n' "$PORT"
        printf '  modelo: %s\n' "$MODEL"
        printf '  timer: %s\n' "$EVERY"
        printf '  agendador: %s\n' "$SCHEDULER"
        printf '  llama.cpp: %s\n' "$LLAMA"
        printf '  gguf url: %s\n' "${KNUDGE_MODEL_URL:-$MODEL_URL}"
        printf '  gguf revisão: %s\n' "${KNUDGE_MODEL_REVISION:-$MODEL_REVISION}"
        printf '  gguf sha256: %s\n' "${KNUDGE_MODEL_SHA256:-$MODEL_SHA256}"
        printf '  llama.cpp instalador sha256: %s\n' "${KNUDGE_LLAMA_INSTALL_SHA256:-$LLAMA_INSTALL_SHA256}"
        return 0
    fi
    preflight "$OPT_PROJECT"
    [ "$SCHEDULER" != "none" ] ||
        die_with_guide "sem systemd --user/launchd; use cron: 0 * * * * $BIN_DIR/$UNIT run"
    has_project "$OPT_PROJECT" || PROJECTS+=("$OPT_PROJECT")
    write_conf
    mkdir -p "$BIN_DIR"
    [ "$SELF" -ef "$BIN_DIR/$UNIT" ] || cp "$SELF" "$BIN_DIR/$UNIT"
    chmod +x "$BIN_DIR/$UNIT"
    scheduler_install
    reconcile_config "$OPT_PROJECT" 0
    log "instalado: ${#PROJECTS[@]} projeto(s), porta=$PORT, timer=$EVERY, agendador=$SCHEDULER"
    case "$SCHEDULER" in
        systemd) log "log: journalctl --user -u $UNIT.service -n 30" ;;
        launchd) log "log: tail -f $LOG_DIR/idle.log" ;;
    esac
}

cmd_subscribe() {
    parse_common_args "$@"
    [ -r "$CONF" ] || die "sistema não instalado; rode --install primeiro"
    load_conf
    detect_scheduler
    if has_project "$OPT_PROJECT"; then
        log "já cadastrado: $OPT_PROJECT"
    else
        PROJECTS+=("$OPT_PROJECT")
        write_conf
        log "cadastrado: $OPT_PROJECT"
    fi
    scheduler_subscribe
}

cmd_unsubscribe() {
    parse_common_args "$@"
    [ -r "$CONF" ] || die "sistema não instalado"
    load_conf
    detect_scheduler
    if ! has_project "$OPT_PROJECT"; then
        log "não cadastrado: $OPT_PROJECT"
        return 0
    fi
    remove_project "$OPT_PROJECT"
    write_conf
    if [ "${#PROJECTS[@]}" -eq 0 ]; then
        scheduler_unsubscribe
        log "descadastrado: $OPT_PROJECT (sem projetos; timer parado, sistema mantido)"
    else
        log "descadastrado: $OPT_PROJECT"
    fi
}

cmd_status() {
    load_conf
    detect_scheduler
    local current="$PWD" p pending out
    printf 'config: %s%s\n' "$CONF" "$([ -r "$CONF" ] || printf ' (ausente)')"
    printf '  porta=%s modelo=%s timer=%s agendador=%s\n' "$PORT" "$MODEL" "$EVERY" "$SCHEDULER"
    printf 'kd: %s\n' "$([ -x "$KD" ] && printf ok || printf AUSENTE)"
    if [ -x "$LLAMA" ]; then printf 'llama.cpp: ok\n'; else printf 'llama.cpp: AUSENTE (instale de %s)\n' "$LLAMA_INSTALL_URL"; fi
    if [ -f "$MODEL" ]; then printf 'modelo: ok\n'; else printf 'modelo: AUSENTE (%s)\n' "$MODEL_URL"; fi
    scheduler_status
    if curl -fsS "http://127.0.0.1:$PORT/health" >/dev/null 2>&1; then
        printf 'servidor: ok\n'
    else
        printf 'servidor: fora\n'
    fi
    probe_endpoint "$current"
    printf 'projeto atual: %s\n' "$(has_project "$current" && printf cadastrado || printf não-cadastrado)"
    printf 'projetos (%s):\n' "${#PROJECTS[@]}"
    for p in "${PROJECTS[@]}"; do
        if [ ! -d "$p" ]; then
            printf '  %s (ausente — projeto removido?)\n' "$p"
            continue
        fi
        out=$(cd "$p" 2>/dev/null && "$KD" drain --status 2>&1)
        pending=$(printf '%s\n' "$out" | sed -n 's/.*pending=\([0-9][0-9]*\).*/\1/p' | tail -1)
        if [ -n "$pending" ]; then
            printf '  %s (pending=%s)\n' "$p" "$pending"
        else
            printf '  %s (pending=? — %s)\n' "$p" "$(printf '%s' "$out" | head -n1)"
        fi
    done
    return 0
}

cmd_uninstall() {
    parse_common_args "$@"
    load_conf
    detect_scheduler
    scheduler_uninstall
    trash "$CONF"
    trash "$BIN_DIR/$UNIT"
    if [ "$OPT_REMOVE_MODEL" = "1" ]; then
        if [ -f "$MODEL" ]; then
            trash "$MODEL"
            log "modelo GGUF movido para o lixo: $MODEL"
        else
            log "modelo GGUF ausente: $MODEL"
        fi
    elif [ -f "$MODEL" ]; then
        log "modelo GGUF preservado: $MODEL (use --remove-model para movê-lo ao lixo)"
    fi
    log "desinstalado (unidades/agentes + config movidos para $TRASH)"
}

cmd_reconcile() {
    parse_common_args "$@"
    load_conf
    [ -n "$OPT_PORT" ] && PORT="$OPT_PORT"
    reconcile_config "$OPT_PROJECT" 1
    if [ -x "$KD" ]; then
        log "reindexando (kd drain --digest)..."
        (cd "$OPT_PROJECT" && "$KD" drain --digest) >&2 || true
    fi
}

cmd_run() {
    load_conf
    local model="${KNUDGE_EMBED_MODEL:-$MODEL}"
    local port="${KNUDGE_EMBED_PORT:-$PORT}"
    if [ "$#" -eq 0 ]; then
        if [ "${#PROJECTS[@]}" -eq 0 ]; then
            log "nenhum projeto cadastrado"
            return 0
        fi
        set -- "${PROJECTS[@]}"
    fi
    trap cleanup EXIT INT TERM
    STARTED_PID=""
    ensure_server "$model" "$port"
    drain_all "$@"
}

usage() {
    cat <<'EOF'
knudge-idle — worker de auto-drain ocioso do knudge (E11-T03/D131/D132/D133)

  knudge-idle install [--project DIR] [--port N] [--model PATH] [--every DUR] [--no-deps] [--dry-run]
  knudge-idle subscribe [--project DIR]     cadastra um projeto (multi-projeto)
  knudge-idle unsubscribe [--project DIR]   descadastra (mantém o sistema instalado)
  knudge-idle status                        saúde: agendador, servidor, fila por projeto
  knudge-idle reconcile [--project DIR]     alinha endpoint/model do kd ao worker instalado
  knudge-idle uninstall [--keep-model|--remove-model]  remove o sistema (preserva o GGUF por padrão)
  knudge-idle run [PROJETO...]              corpo do worker (systemd/launchd chama isto)

Agendador: systemd --user (Linux) ou launchd (macOS). O servidor de embeddings
(knudge-embed) roda persistente; o worker só drena a fila. O install baixa o
llama.cpp (instalador oficial verificado por SHA-256) e o GGUF (revisão pinada +
SHA-256) se ausentes; --no-deps pula. Supply-chain em D183.
EOF
}

cmd="${1:-}"
shift 2>/dev/null || true
case "$cmd" in
    run) cmd_run "$@" ;;
    install) cmd_install "$@" ;;
    subscribe) cmd_subscribe "$@" ;;
    unsubscribe) cmd_unsubscribe "$@" ;;
    status) cmd_status ;;
    uninstall) cmd_uninstall "$@" ;;
    reconcile) cmd_reconcile "$@" ;;
    "" | -h | --help | help) usage ;;
    *) die "subcomando desconhecido: $cmd (use install|subscribe|unsubscribe|status|uninstall|run)" ;;
esac
