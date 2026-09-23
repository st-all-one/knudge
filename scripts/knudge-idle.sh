#!/usr/bin/env bash
# knudge-idle — worker de auto-drain ocioso e instalador do timer (E11-T03 / D131).
#
# O knudge não tem daemon: em `embeddings.mode=lazy` (default) o `kd` drena **um lote** no fim
# de cada invocação. Este script instancia um worker contínuo e seguro para quando você não usa
# o `kd` por muito tempo: garante o servidor de embeddings local e drena a fila de cada projeto
# até `pending = 0`.
#
# Uso:
#   knudge-idle run [PROJETO...]     corpo do worker (o timer/cron chama isto)
#   knudge-idle install [--project DIR] [--port N] [--model PATH] [--every DUR]
#   knudge-idle uninstall            desliga e move as unidades para o lixo
#   knudge-idle status               mostra timer + config instalada
#
# Segurança: idempotente; nunca usa `rm` (move para ${XDG_CACHE_HOME:-~/.cache}/knudge/trash);
# só instala em Linux+systemd; fora disso imprime a linha de cron equivalente.
set -uo pipefail

PROG="knudge-idle"
SELF="${BASH_SOURCE[0]}"
XDG_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}"
XDG_CACHE="${XDG_CACHE_HOME:-$HOME/.cache}"
CONF_DIR="$XDG_CONFIG/local/knudge"
CONF="$CONF_DIR/idle.conf"
UNIT_DIR="$XDG_CONFIG/systemd/user"
BIN_DIR="${KNUDGE_BIN_DIR:-$HOME/.local/bin}"
TRASH="$XDG_CACHE/knudge/trash"
KD="${KNUDGE_KD:-$BIN_DIR/kd}"
LLAMA="${KNUDGE_LLAMA:-$BIN_DIR/llama}"
DEFAULT_MODEL="$XDG_CONFIG/local/knudge/granite-97m-r2-Q8_0.gguf"
LOG_DIR="$XDG_CACHE/knudge"
LOG="$LOG_DIR/idle.log"

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

# Sobe o servidor de embeddings se ainda não houver um no ar (mean pooling + ubatch 2048).
# O `-ub` default do llama.cpp é 512 e rejeita notas longas (drain falha com `indexed=0`).
ensure_server() {
    local model=$1 port=$2 health="http://127.0.0.1:$port/health"
    mkdir -p "$LOG_DIR"
    curl -fsS "$health" >/dev/null 2>&1 && return 0
    [ -x "$LLAMA" ] || die "llama não encontrado em $LLAMA (defina KNUDGE_LLAMA)"
    "$LLAMA" serve -m "$model" --embeddings --pooling mean \
        --host 127.0.0.1 --port "$port" -b 2048 -ub 2048 >>"$LOG" 2>&1 &
    STARTED_PID=$!
    for _ in $(seq 1 60); do
        curl -fsS "$health" >/dev/null 2>&1 && return 0
        sleep 1
    done
    die "servidor de embeddings não subiu (veja $LOG)"
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

cmd_run() {
    local model="${KNUDGE_EMBED_MODEL:-$DEFAULT_MODEL}"
    local port="${KNUDGE_EMBED_PORT:-8999}"
    [ "$#" -eq 0 ] && set -- "$PWD"
    trap cleanup EXIT INT TERM
    STARTED_PID=""
    ensure_server "$model" "$port"
    drain_all "$@"
}

cmd_install() {
    local project="$PWD" port="8999" model="$DEFAULT_MODEL" every="1h"
    while [ "$#" -gt 0 ]; do
        case "$1" in
            --project)
                project="${2:?--project exige um diretório}"
                shift 2
                ;;
            --port)
                port="${2:?--port exige um número}"
                shift 2
                ;;
            --model)
                model="${2:?--model exige um caminho}"
                shift 2
                ;;
            --every)
                every="${2:?--every exige uma duração (ex.: 1h)}"
                shift 2
                ;;
            *) die "opção desconhecida: $1" ;;
        esac
    done
    [ -d "$project" ] || die "projeto inexistente: $project"
    [ -x "$KD" ] || die "kd não encontrado em $KD (defina KNUDGE_KD)"
    if ! command -v systemctl >/dev/null 2>&1 || ! systemctl --user show-environment >/dev/null 2>&1; then
        die "systemd --user indisponível; use cron: 0 * * * * $BIN_DIR/knudge-idle run $project"
    fi

    mkdir -p "$CONF_DIR" "$BIN_DIR" "$UNIT_DIR" "$LOG_DIR"
    cp "$SELF" "$BIN_DIR/knudge-idle"
    chmod +x "$BIN_DIR/knudge-idle"
    {
        printf '# gerado por knudge-idle install (%s)\n' "$(date +%Y-%m-%d)"
        printf 'PROJECT=%s\nPORT=%s\nMODEL=%s\nEVERY=%s\n' "$project" "$port" "$model" "$every"
    } >"$CONF"

    cat >"$UNIT_DIR/knudge-idle.service" <<EOF
[Unit]
Description=knudge: drena a fila de embeddings do projeto
Documentation=file:$project/.knudge

[Service]
Type=oneshot
Environment=KNUDGE_EMBED_PORT=$port
Environment=KNUDGE_EMBED_MODEL=$model
ExecStart=$BIN_DIR/knudge-idle run $project
Nice=10
EOF

    cat >"$UNIT_DIR/knudge-idle.timer" <<EOF
[Unit]
Description=knudge: drain periódico da fila de embeddings

[Timer]
OnBootSec=5min
OnUnitActiveSec=$every
AccuracySec=1min
Persistent=true
Unit=knudge-idle.service

[Install]
WantedBy=timers.target
EOF

    # Unifica instalações anteriores do mesmo worker (`knudge-drain`).
    for old in knudge-drain.timer knudge-drain.service; do
        if [ -e "$UNIT_DIR/$old" ]; then
            systemctl --user disable --now "$old" >/dev/null 2>&1
            trash "$UNIT_DIR/$old"
        fi
    done

    systemctl --user daemon-reload
    systemctl --user enable --now knudge-idle.timer || die "falha ao habilitar o timer"
    log "instalado: projeto=$project porta=$port timer=$every"
    log "teste imediato: systemctl --user start knudge-idle.service"
    log "log: journalctl --user -u knudge-idle.service -n 30"
}

cmd_uninstall() {
    systemctl --user disable --now knudge-idle.timer >/dev/null 2>&1
    trash "$UNIT_DIR/knudge-idle.service"
    trash "$UNIT_DIR/knudge-idle.timer"
    trash "$CONF"
    trash "$BIN_DIR/knudge-idle"
    systemctl --user daemon-reload >/dev/null 2>&1
    log "desinstalado (unidades movidas para $TRASH)"
}

cmd_status() {
    if [ -r "$CONF" ]; then
        printf '%s\n' "--- $CONF ---"
        cat "$CONF"
    else
        printf '%s\n' "sem config instalada ($CONF)"
    fi
    printf '%s\n' "--- timer ---"
    systemctl --user list-timers knudge-idle.timer --no-pager 2>/dev/null || true
}

usage() {
    cat <<'EOF'
knudge-idle — worker de auto-drain ocioso do knudge (E11-T03/D131)

  knudge-idle run [PROJETO...]     corpo do worker (o timer/cron chama isto)
  knudge-idle install [--project DIR] [--port N] [--model PATH] [--every DUR]
  knudge-idle uninstall
  knudge-idle status
EOF
}

cmd="${1:-}"
shift 2>/dev/null || true
case "$cmd" in
    run) cmd_run "$@" ;;
    install) cmd_install "$@" ;;
    uninstall) cmd_uninstall ;;
    status) cmd_status ;;
    "" | -h | --help | help) usage ;;
    *) die "subcomando desconhecido: $cmd (use run|install|uninstall|status)" ;;
esac
