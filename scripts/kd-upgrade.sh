#!/usr/bin/env bash
# kd-upgrade — atualiza o binário `kd` chamando o instalador oficial (release + checksum; E18/D187).
#
# Não reimplementa o install: baixa o `install.sh` oficial (ou usa um local, se houver),
# opcionalmente verifica um SHA-256 pinado do próprio instalador e o executa com `VERSION`.
# Verboso: cada passo vai para stderr. Cross-platform via Git Bash/WSL no Windows (o wrapper
# embutido é Unix — D185); no Windows nativo, baixe o release manualmente (wiki/usage/16_self.md).
#
# Uso:
#   kd-upgrade [upgrade] [--version TAG] [--dry-run]
#
# Variáveis:
#   KNUDGE_INSTALL_URL     URL do install.sh (default: raw.githubusercontent do repositório)
#   KNUDGE_INSTALL_SHA256  SHA-256 esperado do install.sh (opcional; vazio ⇒ não verifica)
set -uo pipefail

PROG="kd-upgrade"
INSTALL_URL="${KNUDGE_INSTALL_URL:-https://raw.githubusercontent.com/st-all-one/knudge/main/install.sh}"
INSTALL_SHA256="${KNUDGE_INSTALL_SHA256:-}"
VERSION=""
DRY_RUN=""
XDG_CACHE="${XDG_CACHE_HOME:-$HOME/.cache}"
TRASH="$XDG_CACHE/knudge/trash"

log() { printf '%s: %s\n' "$PROG" "$*" >&2; }
die() {
    log "$*"
    exit 1
}

# Move para o lixo recuperável (nunca `rm`).
trash() {
    [ -e "$1" ] || return 0
    mkdir -p "$TRASH"
    mv "$1" "$TRASH/$(basename "$1").$(date +%Y%m%d%H%M%S).$$" 2>/dev/null || true
}

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

usage() {
    cat <<'EOF'
kd-upgrade — atualiza o `kd` pelo instalador oficial (release + checksum; D187).

  kd-upgrade [upgrade] [--version TAG] [--dry-run]

  --version TAG   versão/tag alvo (default: latest)
  --dry-run       mostra o plano sem baixar nem executar
EOF
}

case "${1:-}" in
    upgrade) shift ;;
    -h | --help | help)
        usage
        exit 0
        ;;
    "") : ;;      # sem args ⇒ upgrade
    -*) : ;;       # flag direta (sem subcomando)
    *) die "subcomando desconhecido: $1 (use upgrade)" ;;
esac

while [ "$#" -gt 0 ]; do
    case "$1" in
        --version)
            VERSION="${2:?--version exige uma tag}"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=1
            shift
            ;;
        -h | --help)
            usage
            exit 0
            ;;
        *) die "opção desconhecida: $1" ;;
    esac
done

if [ -n "$DRY_RUN" ]; then
    printf 'kd-upgrade (dry-run)\n'
    printf '  instalador: %s\n' "$INSTALL_URL"
    printf '  versão: %s\n' "${VERSION:-latest}"
    [ -n "$INSTALL_SHA256" ] && printf '  sha256 do instalador: %s\n' "$INSTALL_SHA256"
    printf '  comando: VERSION=%s bash <instalador>\n' "${VERSION:-latest}"
    exit 0
fi

tmp="$XDG_CACHE/knudge/kd-upgrade.$$"
mkdir -p "$(dirname "$tmp")"
log "baixando instalador oficial ($INSTALL_URL)..."
fetch "$INSTALL_URL" "$tmp" || die "falha ao baixar o instalador"
if [ -n "$INSTALL_SHA256" ]; then
    got="$(sha256_of "$tmp")" || die "nenhum verificador SHA-256 disponível"
    if [ "$got" != "$INSTALL_SHA256" ]; then
        trash "$tmp"
        die "checksum do instalador não confere: esperado $INSTALL_SHA256, obtido $got"
    fi
    log "checksum do instalador confere"
fi
log "executando instalador (versão ${VERSION:-latest})..."
VERSION="${VERSION:-latest}" bash "$tmp"
rc=$?
trash "$tmp"
exit "$rc"
