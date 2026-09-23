#!/usr/bin/env bash
# knudge-idle — worker de auto-drain ocioso e gerenciador do timer (E11-T03 / D131 / D132).
#
# O knudge não tem daemon: em `embeddings.mode=lazy` (default) o `kd` drena **um lote** no fim
# de cada invocação. Este script instancia um worker contínuo e seguro para quando você não usa
# o `kd` por muito tempo: garante o servidor de embeddings local e drena a fila de **cada projeto
# cadastrado** até `pending = 0`.
#
# Uso:
#   knudge-idle install [--project DIR] [--port N] [--model PATH] [--every DUR]
#   knudge-idle subscribe [--project DIR]      # cadastra um projeto (multi-projeto)
#   knudge-idle unsubscribe [--project DIR]    # descadastra (mantém o sistema instalado)
#   knudge-idle status                         # saúde: timer, servidor, fila por projeto
#   knudge-idle uninstall                      # remove o sistema (unidades + config + binário)
#   knudge-idle run [PROJETO...]               # corpo do worker (o timer/cron chama isto)
#
# Segurança: idempotente; nunca usa `rm` (move para ${XDG_CACHE_HOME:-~/.cache}/knudge/trash);
# só instala em Linux+systemd; fora disso imprime a linha de cron equivalente. O GGUF mora ao
# lado do config.toml global (${XDG_CONFIG_HOME:-~/.config}/local/knudge/).
set -uo pipefail

PROG="knudge-idle"
SELF="${BASH_SOURCE[0]}"
XDG_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}"
XDG_CACHE="${XDG_CACHE_HOME:-$HOME/.cache}"
CONF_DIR="$XDG_CONFIG/local/knudge"
CONF="$CONF_DIR/idle.conf"
UNIT_DIR="$XDG_CONFIG/systemd/user"
UNIT="knudge-idle"
BIN_DIR="${KNUDGE_BIN_DIR:-$HOME/.local/bin}"
TRASH="$XDG_CACHE/knudge/trash"
KD="${KNUDGE_KD:-$BIN_DIR/kd}"
LLAMA="${KNUDGE_LLAMA:-$BIN_DIR/llama}"
DEFAULT_MODEL="$CONF_DIR/granite-97m-r2-Q8_0.gguf"
DEFAULT_PORT=8999
DEFAULT_EVERY=1h
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

# ---------- systemd ----------
write_units() {
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
OnBootSec=5min
OnUnitActiveSec=$EVERY
AccuracySec=1min
Persistent=true
Unit=$UNIT.service

[Install]
WantedBy=timers.target
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

# ---------- servidor ----------
# Sobe o servidor se ainda não houver um no ar (mean pooling + ubatch 2048). O `-ub` default do
# llama.cpp é 512 e rejeita notas longas (drain falha com `indexed=0`).
ensure_server() {
    local model=$1 port=$2 health="http://127.0.0.1:$port/health"
    mkdir -p "$LOG_DIR"
    if curl -fsS "$health" >/dev/null 2>&1; then
        log "servidor já no ar em $health; o worker não o reconfigura (garanta -b 2048 -ub 2048)"
        return 0
    fi
    [ -x "$LLAMA" ] || die "llama não encontrado em $LLAMA (defina KNUDGE_LLAMA)"
    "$LLAMA" serve -m "$model" --embeddings --pooling mean \
        --host 127.0.0.1 --port "$port" -b 2048 -ub 2048 >>"$LOG" 2>&1 &
    STARTED_PID=$!
    local _i
    for _i in $(seq 1 60); do
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
    while [ "$#" -gt 0 ]; do
        case "$1" in
            --project)
                OPT_PROJECT="${2:?--project exige um diretório}"
                shift 2
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
    if ! command -v systemctl >/dev/null 2>&1 || ! systemctl --user show-environment >/dev/null 2>&1; then
        die "systemd --user indisponível; use cron: 0 * * * * $BIN_DIR/$UNIT run"
    fi
    [ -x "$KD" ] || die "kd não encontrado em $KD (instale o knudge ou defina KNUDGE_KD)"
    [ -x "$LLAMA" ] || die "llama.cpp não encontrado em $LLAMA (instale ou defina KNUDGE_LLAMA)"
    [ -f "$MODEL" ] || die "modelo GGUF ausente: $MODEL (coloque-o ao lado do config.toml global)"
    [ -d "$project" ] || die "projeto inexistente: $project"
}

# ---------- ações ----------
cmd_install() {
    parse_common_args "$@"
    load_conf
    [ -n "$OPT_PORT" ] && PORT="$OPT_PORT"
    [ -n "$OPT_MODEL" ] && MODEL="$OPT_MODEL"
    [ -n "$OPT_EVERY" ] && EVERY="$OPT_EVERY"
    preflight "$OPT_PROJECT"
    has_project "$OPT_PROJECT" || PROJECTS+=("$OPT_PROJECT")
    write_conf
    mkdir -p "$BIN_DIR"
    [ "$SELF" -ef "$BIN_DIR/$UNIT" ] || cp "$SELF" "$BIN_DIR/$UNIT"
    chmod +x "$BIN_DIR/$UNIT"
    write_units
    unify_legacy
    systemctl --user daemon-reload
    systemctl --user enable --now "$UNIT.timer" || die "falha ao habilitar o timer"
    log "instalado: ${#PROJECTS[@]} projeto(s), porta=$PORT, timer=$EVERY"
    log "log: journalctl --user -u $UNIT.service -n 30"
}

cmd_subscribe() {
    parse_common_args "$@"
    [ -r "$CONF" ] || die "sistema não instalado; rode --install primeiro"
    load_conf
    if has_project "$OPT_PROJECT"; then
        log "já cadastrado: $OPT_PROJECT"
    else
        PROJECTS+=("$OPT_PROJECT")
        write_conf
        log "cadastrado: $OPT_PROJECT"
    fi
    systemctl --user enable --now "$UNIT.timer" >/dev/null 2>&1 || true
}

cmd_unsubscribe() {
    parse_common_args "$@"
    [ -r "$CONF" ] || die "sistema não instalado"
    load_conf
    if ! has_project "$OPT_PROJECT"; then
        log "não cadastrado: $OPT_PROJECT"
        return 0
    fi
    remove_project "$OPT_PROJECT"
    write_conf
    if [ "${#PROJECTS[@]}" -eq 0 ]; then
        systemctl --user disable --now "$UNIT.timer" >/dev/null 2>&1 || true
        log "descadastrado: $OPT_PROJECT (sem projetos; timer parado, sistema mantido)"
    else
        log "descadastrado: $OPT_PROJECT"
    fi
}

cmd_status() {
    load_conf
    local current="$PWD" p pending
    printf 'config: %s%s\n' "$CONF" "$([ -r "$CONF" ] || printf ' (ausente)')"
    printf '  porta=%s modelo=%s timer=%s\n' "$PORT" "$MODEL" "$EVERY"
    printf 'kd: %s\n' "$([ -x "$KD" ] && printf ok || printf AUSENTE)"
    printf 'llama.cpp: %s\n' "$([ -x "$LLAMA" ] && printf ok || printf AUSENTE)"
    printf 'modelo: %s\n' "$([ -f "$MODEL" ] && printf ok || printf AUSENTE)"
    printf 'unidades: %s\n' "$([ -e "$UNIT_DIR/$UNIT.timer" ] && printf ok || printf AUSENTE)"
    printf 'timer: %s\n' "$(systemctl --user is-active "$UNIT.timer" 2>/dev/null || printf inativo)"
    if curl -fsS "http://127.0.0.1:$PORT/health" >/dev/null 2>&1; then
        printf 'servidor: ok (já no ar; o worker não o reconfigura — garanta -b 2048 -ub 2048)\n'
    else
        printf 'servidor: fora\n'
    fi
    printf 'projeto atual: %s\n' "$(has_project "$current" && printf cadastrado || printf não-cadastrado)"
    printf 'projetos (%s):\n' "${#PROJECTS[@]}"
    for p in "${PROJECTS[@]}"; do
        pending=$(cd "$p" 2>/dev/null && "$KD" maintenance index --status 2>/dev/null | sed -n 's/.*pending: \([0-9][0-9]*\).*/\1/p' | tail -1)
        printf '  %s (pending=%s)\n' "$p" "${pending:-?}"
    done
    systemctl --user list-timers "$UNIT.timer" --no-pager 2>/dev/null | sed -n '1,2p' | sed 's/^/  /'
    return 0
}

cmd_uninstall() {
    systemctl --user disable --now "$UNIT.timer" >/dev/null 2>&1
    trash "$UNIT_DIR/$UNIT.service"
    trash "$UNIT_DIR/$UNIT.timer"
    trash "$CONF"
    trash "$BIN_DIR/$UNIT"
    systemctl --user daemon-reload >/dev/null 2>&1
    log "desinstalado (unidades movidas para $TRASH)"
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
knudge-idle — worker de auto-drain ocioso do knudge (E11-T03/D131/D132)

  knudge-idle install [--project DIR] [--port N] [--model PATH] [--every DUR]
  knudge-idle subscribe [--project DIR]     cadastra um projeto (multi-projeto)
  knudge-idle unsubscribe [--project DIR]   descadastra (mantém o sistema instalado)
  knudge-idle status                        saúde: timer, servidor, fila por projeto
  knudge-idle uninstall                     remove o sistema
  knudge-idle run [PROJETO...]              corpo do worker (timer/cron chama isto)
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
