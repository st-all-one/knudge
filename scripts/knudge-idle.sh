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
#   knudge-idle install [--project DIR] [--port N] [--model PATH] [--every DUR] [--no-deps]
#   knudge-idle subscribe [--project DIR]      # cadastra um projeto (multi-projeto)
#   knudge-idle unsubscribe [--project DIR]    # descadastra (mantém o sistema instalado)
#   knudge-idle status                         # saúde: agendador, servidor, fila por projeto
#   knudge-idle uninstall                      # remove o sistema (unidades/agentes + config + binário)
#   knudge-idle run [PROJETO...]               # corpo do worker (o agendador chama isto)
#
# O install baixa o llama.cpp (script oficial llama.app; fallback para brew/winget/scoop/choco/
# apt/dnf/pacman/zypper) e o GGUF recomendado se ausentes; --no-deps pula.
#
# Segurança: idempotente; nunca usa `rm` (move para ${XDG_CACHE_HOME:-~/.cache}/knudge/trash).
# O GGUF mora ao lado do config.toml global (${XDG_CONFIG_HOME:-~/.config}/local/knudge/).
set -uo pipefail

PROG="knudge-idle"
SELF="${BASH_SOURCE[0]}"
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
DEFAULT_PORT=8999
DEFAULT_EVERY=1h
LOG_DIR="$XDG_CACHE/knudge"
LOG="$LOG_DIR/idle.log"
SCHEDULER="" # systemd | launchd | none
LLAMA_INSTALL_URL="https://llama.app/install.sh"
MODEL_URL="https://huggingface.co/mykor/granite-embedding-97m-multilingual-r2-GGUF/resolve/main/granite-embedding-97M-multilingual-r2-Q8_0.gguf"

log() { printf '%s: %s\n' "$PROG" "$*" >&2; }
die() {
    log "$*"
    exit 1
}

trash() {
    [ -e "$1" ] || return 0
    mkdir -p "$TRASH"
    mv "$1" "$TRASH/$(basename "$1").$(date +%Y%m%d%H%M%S).$$" 2>/dev/null || true
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

2) Baixe o modelo GGUF (~100 MB) para $MODEL:
   curl -fL -o "$MODEL" \\
     https://huggingface.co/mykor/granite-embedding-97m-multilingual-r2-GGUF/resolve/main/granite-embedding-97M-multilingual-r2-Q8_0.gguf
   (sem curl: wget -O "$MODEL" <url>)

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
     (modelo em docs/06-embeddings.md) e carregue: launchctl bootstrap gui/\$(id -u) <plist>
   Windows: atalho/.bat no Startup do usuario ou tarefa no Agendador de Tarefas:
     llama.exe serve -m "%USERPROFILE%\\granite-97m-r2-Q8_0.gguf" --embeddings --pooling mean -b 2048 -ub 2048 --port $PORT
   Sem agendador (qualquer SO):
     nohup llama serve -m "$MODEL" --embeddings --pooling mean -b 2048 -ub 2048 --host 127.0.0.1 --port $PORT >> "$LOG_DIR/embed.log" 2>&1 &

5) Aponte o kd e valide:
   kd config set embeddings.provider http
   kd config set embeddings.endpoint http://127.0.0.1:$PORT/v1/embeddings
   kd config set embeddings.model ibm-granite/granite-embedding-97m-multilingual-r2
   kd config set embeddings.dimensions 384
   kd maintenance index --drain
   kd maintenance watch-service --status

Guia completo por SO: docs/06-embeddings.md
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

# Drena cada projeto em lotes (`--drain` processa `embeddings.batch` por chamada) até pending=0.
drain_all() {
    local rc=0 proj n out pending indexed last
    for proj in "$@"; do
        n=0
        last=""
        while :; do
            out=$(cd "$proj" 2>/dev/null && "$KD" maintenance index --drain 2>&1) || {
                log "falha no drain de $proj: $out"
                rc=1
                break
            }
            printf '%s: %s\n' "$proj" "$out"
            pending=$(printf '%s\n' "$out" | sed -n 's/.*pending=\([0-9][0-9]*\).*/\1/p' | tail -1)
            indexed=$(printf '%s\n' "$out" | sed -n 's/.*indexed=\([0-9][0-9]*\).*/\1/p' | tail -1)
            [ -z "$pending" ] && break
            [ "$pending" -eq 0 ] && break
            if [ "${indexed:-0}" -eq 0 ] && [ "$pending" = "$last" ]; then
                log "sem progresso em $proj (pending=$pending) — provedor ok?"
                rc=1
                break
            fi
            last="$pending"
            n=$((n + 1))
            if [ "$n" -ge "${KNUDGE_DRAIN_MAX:-10000}" ]; then
                log "backstop de lotes atingido em $proj"
                rc=1
                break
            fi
        done
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
fetch() {
    local url=$1 dest=$2
    if command -v curl >/dev/null 2>&1; then
        curl -fL --proto '=https' --tlsv1.2 -o "$dest" "$url" && return 0
    fi
    if command -v wget >/dev/null 2>&1; then
        wget -q -O "$dest" "$url" && return 0
    fi
    return 1
}

# Instala o llama.cpp: binário já presente > script oficial (llama.app) > gestor de pacotes.
install_llama() {
    resolve_llama
    [ -x "$LLAMA" ] && return 0
    if command -v curl >/dev/null 2>&1; then
        log "instalando llama.cpp (script oficial $LLAMA_INSTALL_URL)..."
        if curl -LsSf "$LLAMA_INSTALL_URL" | sh >/dev/null 2>&1; then
            hash -r 2>/dev/null || true
            resolve_llama
            [ -x "$LLAMA" ] && return 0
        fi
        log "script oficial falhou; tentando gestor de pacotes..."
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

# Baixa o GGUF recomendado para $MODEL (default: ao lado do config.toml global).
install_model() {
    [ -f "$MODEL" ] && return 0
    mkdir -p "$(dirname "$MODEL")"
    log "baixando modelo GGUF (~100 MB) para $MODEL..."
    fetch "$MODEL_URL" "$MODEL.tmp" || { trash "$MODEL.tmp"; return 1; }
    mv "$MODEL.tmp" "$MODEL"
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
    preflight "$OPT_PROJECT"
    [ "$SCHEDULER" != "none" ] ||
        die_with_guide "sem systemd --user/launchd; use cron: 0 * * * * $BIN_DIR/$UNIT run"
    has_project "$OPT_PROJECT" || PROJECTS+=("$OPT_PROJECT")
    write_conf
    mkdir -p "$BIN_DIR"
    [ "$SELF" -ef "$BIN_DIR/$UNIT" ] || cp "$SELF" "$BIN_DIR/$UNIT"
    chmod +x "$BIN_DIR/$UNIT"
    scheduler_install
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
    local current="$PWD" p pending
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
    printf 'projeto atual: %s\n' "$(has_project "$current" && printf cadastrado || printf não-cadastrado)"
    printf 'projetos (%s):\n' "${#PROJECTS[@]}"
    for p in "${PROJECTS[@]}"; do
        pending=$(cd "$p" 2>/dev/null && "$KD" maintenance index --status 2>/dev/null | sed -n 's/.*pending: \([0-9][0-9]*\).*/\1/p' | tail -1)
        printf '  %s (pending=%s)\n' "$p" "${pending:-?}"
    done
    return 0
}

cmd_uninstall() {
    detect_scheduler
    scheduler_uninstall
    trash "$CONF"
    trash "$BIN_DIR/$UNIT"
    log "desinstalado (unidades/agentes movidos para $TRASH)"
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

  knudge-idle install [--project DIR] [--port N] [--model PATH] [--every DUR] [--no-deps]
  knudge-idle subscribe [--project DIR]     cadastra um projeto (multi-projeto)
  knudge-idle unsubscribe [--project DIR]   descadastra (mantém o sistema instalado)
  knudge-idle status                        saúde: agendador, servidor, fila por projeto
  knudge-idle uninstall                     remove o sistema (unidades/agentes + config)
  knudge-idle run [PROJETO...]              corpo do worker (systemd/launchd chama isto)

Agendador: systemd --user (Linux) ou launchd (macOS). O servidor de embeddings
(knudge-embed) roda persistente; o worker só drena a fila. O install baixa o
llama.cpp (llama.app) e o GGUF se ausentes; --no-deps pula.
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
    uninstall) cmd_uninstall ;;
    "" | -h | --help | help) usage ;;
    *) die "subcomando desconhecido: $cmd (use install|subscribe|unsubscribe|status|uninstall|run)" ;;
esac
