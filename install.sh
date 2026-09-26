#!/usr/bin/env bash
# knudge installer — estilo `curl | bash` (release) ou `./install.sh --from-source`.
#
# Modo release: baixa os binários pré-compilados do GitHub Release (latest por padrão, ou
# VERSION=vX.Y.Z), verifica o checksum SHA-256 e instala em ~/.local/bin.
#
# Modo source: com `--from-source` (ou rodando `./install.sh` de dentro do repositório), faz
# `cargo build --release` e instala os binários otimizados.
#
# O pacote contém dois binários:
#   kd          → CLI (o comando do dia a dia)
#   knudge-mcp  → servidor MCP (JSON-RPC 2.0 sobre stdio)
#
# Nada é apagado de forma irreversível: artefatos antigos são **movidos** para um lixo
# recuperável em ${XDG_CACHE_HOME:-~/.cache}/knudge/trash (invalida o caminho original).
#
# Uso:
#   curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/st-all-one/knudge/main/install.sh | bash
#   curl ... | VERSION=v0.4.0 bash
#   curl ... | INSTALL_DIR=/usr/local/bin bash
#   ./install.sh --from-source
#   ./install.sh --uninstall
#
# Variáveis de ambiente:
#   VERSION             tag a instalar (default: latest)
#   INSTALL_DIR         diretório de destino (default: ~/.local/bin)
#   KD_TARGET           sobrescreve o target triple detectado
#   KD_REPO             owner/repo do GitHub (default: st-all-one/knudge; útil para testar)
#   KD_BASE_URL         base URL para download (default: https://github.com; útil para testar)
#   KD_CACHE_DIR        raiz de cache/lixo (default: ${XDG_CACHE_HOME:-~/.cache}/knudge)
#   KD_NO_PATH=1        não edita os rc files (PATH)
#   KD_NO_COMPLETIONS=1 não instala completions
set -euo pipefail

if [ -z "${BASH_VERSION:-}" ]; then
    printf '%s\n' "Este instalador usa recursos do bash. Rode: curl ... | bash" >&2
    exit 1
fi

REPO="${KD_REPO:-st-all-one/knudge}"
BASE_URL_ROOT="${KD_BASE_URL:-https://github.com}"
CACHE_ROOT="${KD_CACHE_DIR:-${XDG_CACHE_HOME:-${HOME}/.cache}/knudge}"
BINARIES=(kd knudge-mcp)
INSTALL_DIR="${INSTALL_DIR:-${HOME}/.local/bin}"
VERSION="${VERSION:-latest}"
FROM_SOURCE=0
DO_UNINSTALL=0

# HTTPS obrigatório por padrão (evita downgrade de TLS no `curl | bash`). Um `KD_BASE_URL`
# `http://` explícito (servidor de teste local) libera — nunca é o caminho default.
CURL_PROTO=""
case "$BASE_URL_ROOT" in
    https://*) CURL_PROTO="--proto =https --proto-redir =https --tlsv1.2" ;;
esac

# ── helpers ──────────────────────────────────────────────────────────────────

info() { printf '\033[34m==>\033[0m %s\n' "$*"; }
ok()   { printf '\033[32m  ✓\033[0m %s\n' "$*"; }
warn() { printf '\033[33m  !\033[0m %s\n' "$*" >&2; }
err()  { printf '\033[31m  ✗\033[0m %s\n' "$*" >&2; exit 1; }

require() {
    command -v "$1" >/dev/null 2>&1 || err "Ferramenta necessária não encontrada: $1"
}

# Caminho com $HOME abreviado para ~ (só na mensagem).
tilde() { printf '%s' "${1/#$HOME/\~}"; }

# Move um caminho para o lixo recuperável (nunca apaga; invalida o caminho original).
trash() {
    local src="$1"
    [ -e "$src" ] || [ -L "$src" ] || return 0
    local dest="${CACHE_ROOT}/trash/$(basename "$src").$(date +%Y%m%d%H%M%S).$$"
    mkdir -p "${CACHE_ROOT}/trash"
    mv -- "$src" "$dest"
}

usage() {
    sed -n '2,32p' "$0" 2>/dev/null | sed 's/^# \{0,1\}//' || cat <<'EOF'
knudge installer

Uso: install.sh [--from-source] [--install-dir DIR] [--version TAG] [--uninstall]
EOF
}

# ── argumentos ───────────────────────────────────────────────────────────────

while [ $# -gt 0 ]; do
    case "$1" in
        --from-source|-s) FROM_SOURCE=1 ;;
        --uninstall)      DO_UNINSTALL=1 ;;
        --install-dir)    [ $# -ge 2 ] || err "--install-dir exige um valor"; INSTALL_DIR="$2"; shift ;;
        --install-dir=*)  INSTALL_DIR="${1#*=}" ;;
        --prefix)         [ $# -ge 2 ] || err "--prefix exige um valor"; INSTALL_DIR="$2/bin"; shift ;;
        --prefix=*)       INSTALL_DIR="${1#*=}/bin" ;;
        --version)        [ $# -ge 2 ] || err "--version exige um valor"; VERSION="$2"; shift ;;
        --version=*)      VERSION="${1#*=}" ;;
        --no-path)        KD_NO_PATH=1 ;;
        --no-completions) KD_NO_COMPLETIONS=1 ;;
        -h|--help)        usage; exit 0 ;;
        *) err "Argumento desconhecido: $1 (use --help)" ;;
    esac
    shift
done

# Diretório do script (quando é um arquivo real, não `curl | bash`).
SCRIPT_DIR="$(pwd)"
if [ -n "${BASH_SOURCE[0]:-}" ] && [ -f "${BASH_SOURCE[0]}" ]; then
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    if [ "$FROM_SOURCE" -eq 0 ] && [ "$DO_UNINSTALL" -eq 0 ] && [ -f "${SCRIPT_DIR}/Cargo.toml" ] && [ -d "${SCRIPT_DIR}/crates/knudge-cli" ]; then
        FROM_SOURCE=1
        info "Repositório detectado; instalando a partir do source."
    fi
fi

# ── detecção de OS/arquitetura → target triple ──────────────────────────────

detect_target() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)  os="linux" ;;
        Darwin) os="darwin" ;;
        MINGW*|MSYS*|CYGWIN*) os="windows" ;;
        *) err "Sistema operacional não suportado: $os" ;;
    esac

    case "$arch" in
        x86_64|amd64)  arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) err "Arquitetura não suportada: $arch" ;;
    esac

    case "$os-$arch" in
        linux-x86_64)   echo "x86_64-unknown-linux-musl" ;;
        linux-aarch64)  echo "aarch64-unknown-linux-musl" ;;
        darwin-aarch64) echo "aarch64-apple-darwin" ;;
        darwin-x86_64)  echo "x86_64-apple-darwin" ;;
        windows-x86_64) echo "x86_64-pc-windows-msvc" ;;
        windows-aarch64) echo "aarch64-pc-windows-msvc" ;;
        *) err "Sem binário publicado para $os/$arch" ;;
    esac
}

# ── resolução da versão ──────────────────────────────────────────────────────

resolve_version() {
    if [ "$VERSION" != "latest" ]; then
        echo "$VERSION"
        return
    fi
    # Segue o redirect de /releases/latest e extrai a tag do caminho final.
    local effective tag
    # `$CURL_PROTO` é intencionalmente não-entre-aspas (word-split das flags).
    # shellcheck disable=SC2086
    effective="$(curl -fsSL $CURL_PROTO -I -o /dev/null -w '%{url_effective}' "${BASE_URL_ROOT}/${REPO}/releases/latest")" \
        || err "Falha ao resolver a versão latest em ${BASE_URL_ROOT}/${REPO}"
    tag="${effective##*/tag/}"
    case "$tag" in
        v[0-9]*|[0-9]*) echo "$tag" ;;
        *) err "Não foi possível resolver a versão latest (release não encontrado em ${BASE_URL_ROOT}/${REPO})" ;;
    esac
}

# ── versão instalada / upgrade ───────────────────────────────────────────────

current_version() {
    local bin="$INSTALL_DIR/kd"
    [ -x "$bin" ] || return 0
    "$bin" self version 2>/dev/null | awk '{print $NF}'
}

# Comparação semver simples (X.Y.Z); retorna 0 quando $1 > $2.
version_gt() {
    case "$1" in
        [0-9]*.[0-9]*.[0-9]*) : ;;
        *) return 1 ;;
    esac
    case "$2" in
        [0-9]*.[0-9]*.[0-9]*) : ;;
        *) return 1 ;;
    esac
    [ "$1" = "$2" ] && return 1
    local IFS=. i
    # shellcheck disable=SC2206
    local a=($1) b=($2)
    for i in 0 1 2; do
        [ "${a[$i]:-0}" -gt "${b[$i]:-0}" ] && return 0
        [ "${a[$i]:-0}" -lt "${b[$i]:-0}" ] && return 1
    done
    return 1
}

check_upgrade() {
    local installed
    installed="$(current_version)"
    [ -n "$installed" ] || return 0

    if [ "$installed" = "${VERSION#v}" ]; then
        ok "knudge ${VERSION} já instalado em $(tilde "$INSTALL_DIR") — nada a fazer."
        exit 0
    fi
    if version_gt "${VERSION#v}" "$installed"; then
        info "Atualizando knudge v${installed} → v${VERSION#v}"
    else
        warn "Versão mais nova instalada (v${installed}); instalando v${VERSION#v} (downgrade)."
    fi
}

# ── build a partir do source ─────────────────────────────────────────────────

build_from_source() {
    require cargo
    [ -f "${SCRIPT_DIR}/Cargo.toml" ] || err "--from-source exige rodar de dentro do repositório (Cargo.toml não encontrado em ${SCRIPT_DIR})"
    info "Compilando release a partir do source..."
    ( cd "$SCRIPT_DIR" && cargo build --release --locked -p knudge-cli -p knudge-mcp )
    SRC_DIR="${SCRIPT_DIR}/target/release"
}

# ── download do release ──────────────────────────────────────────────────────

download_release() {
    local target="$1"
    local ext="tar.gz"
    if [ "$target" = *-pc-windows-* ]; then
        ext="zip"
        require unzip
    fi

    local version_no_v="${VERSION#v}"
    local asset="knudge-${version_no_v}-${target}.${ext}"
    local base_url="${BASE_URL_ROOT}/${REPO}/releases/download/${VERSION}"

    # Staging fixo em cache (sobrevive entre execuções; nada é apagado).
    local staging="${CACHE_ROOT}/staging"
    mkdir -p "$staging"
    # Invalida o staging anterior movendo-o para o lixo.
    trash "${staging}/out"
    mkdir -p "${staging}/out"

    info "Baixando ${asset}..."
    # shellcheck disable=SC2086
    curl -fsSL $CURL_PROTO -o "${staging}/${asset}" "${base_url}/${asset}" \
        || err "Falha no download: ${base_url}/${asset}"

    info "Verificando checksum SHA-256..."
    if ! command -v sha256sum >/dev/null 2>&1 && ! command -v shasum >/dev/null 2>&1; then
        err "Nenhum verificador SHA-256 encontrado (instale coreutils ou perl-Digest-SHA)"
    fi
    # shellcheck disable=SC2086
    curl -fsSL $CURL_PROTO -o "${staging}/sha256sums.txt" "${base_url}/sha256sums.txt" \
        || err "Falha no download do checksum: ${base_url}/sha256sums.txt"
    (
        cd "$staging"
        grep -F "  ${asset}" sha256sums.txt > "checksum.one" \
            || err "Checksum para ${asset} não encontrado em sha256sums.txt"
        sha256sum -c "checksum.one" >/dev/null 2>&1 \
            || shasum -a 256 -c "checksum.one" >/dev/null 2>&1 \
            || err "Verificação de checksum falhou. Abortando por segurança."
    )
    ok "Checksum OK"

    info "Extraindo binários..."
    if [ "$ext" = "zip" ]; then
        unzip -o -q "${staging}/${asset}" -d "${staging}/out"
    else
        tar --no-same-owner -xzf "${staging}/${asset}" -C "${staging}/out"
    fi
    SRC_DIR="${staging}/out"
}

# ── instalação ───────────────────────────────────────────────────────────────

install_binaries() {
    mkdir -p "$INSTALL_DIR" 2>/dev/null || err "Não foi possível criar $(tilde "$INSTALL_DIR") (use sudo ou outro --install-dir)"
    [ -w "$INSTALL_DIR" ] || err "$(tilde "$INSTALL_DIR") não é gravável (use sudo ou outro --install-dir)"

    for bin in "${BINARIES[@]}"; do
        local src="${SRC_DIR}/${bin}"
        case "$(uname -s)" in
            MINGW*|MSYS*|CYGWIN*) src="${SRC_DIR}/${bin}.exe" ;;
        esac
        [ -f "$src" ] || err "Binário não encontrado no pacote: ${bin}"
        # Preserva o binário anterior no lixo antes de sobrescrever.
        trash "${INSTALL_DIR}/${bin}"
        install -m 0755 "$src" "${INSTALL_DIR}/${bin}"
        ok "Instalado: $(tilde "${INSTALL_DIR}/${bin}")"
    done
}

# Confirma que o binário instalado **executa** (pega linker/arquitetura errados antes do PATH).
verify_binaries() {
    local kd="${INSTALL_DIR}/kd" version
    [ -x "$kd" ] || err "Binário não instalado: $(tilde "$kd")"
    version="$("$kd" self version 2>/dev/null | head -n1)" \
        || err "O binário instalado não executa ($(tilde "$kd")); verifique dependências do sistema"
    ok "Binário responde: ${version}"
}

# Cria a pasta de configuração global e semeia os defaults (D61).
setup_global_config() {
    info "Configurando a config global..."
    local global_dir="${XDG_CONFIG_HOME:-$HOME/.config}/local/knudge"
    local global_file="${global_dir}/config.toml"

    if [ -f "$global_file" ]; then
        ok "config global já existe em $(tilde "$global_file")"
        return 0
    fi
    if "${INSTALL_DIR}/kd" config set --key mcp.hints_cap --value 3 --global >/dev/null 2>&1; then
        ok "config global criada em $(tilde "$global_file")"
    else
        mkdir -p "$global_dir" 2>/dev/null || true
        warn "Não foi possível semear $(tilde "$global_file"); defaults serão usados."
    fi
}

# Instala completions de bash/zsh/fish a partir de `kd self completions`.
setup_completions() {
    [ "${KD_NO_COMPLETIONS:-0}" = "1" ] && return 0
    local kd="${INSTALL_DIR}/kd"
    [ -x "$kd" ] || return 0

    info "Instalando completions..."
    local data="${XDG_DATA_HOME:-$HOME/.local/share}"
    local config="${XDG_CONFIG_HOME:-$HOME/.config}"

    # bash
    local bash_dir="${data}/bash-completion/completions"
    mkdir -p "$bash_dir" 2>/dev/null || true
    if "$kd" self completions bash > "${bash_dir}/kd" 2>/dev/null; then
        ok "completion bash: $(tilde "${bash_dir}/kd")"
    fi

    # zsh
    local zsh_dir="${data}/zsh/site-functions"
    mkdir -p "$zsh_dir" 2>/dev/null || true
    if "$kd" self completions zsh > "${zsh_dir}/_kd" 2>/dev/null; then
        ok "completion zsh: $(tilde "${zsh_dir}/_kd")"
        add_zsh_fpath "$zsh_dir"
    fi

    # fish (só quando o fish parece instalado)
    if command -v fish >/dev/null 2>&1 || [ -d "${config}/fish" ]; then
        local fish_dir="${config}/fish/completions"
        mkdir -p "$fish_dir" 2>/dev/null || true
        if "$kd" self completions fish > "${fish_dir}/kd.fish" 2>/dev/null; then
            ok "completion fish: $(tilde "${fish_dir}/kd.fish")"
        fi
    fi
}

add_zsh_fpath() {
    local dir="$1"
    local rc="${ZDOTDIR:-$HOME}/.zshrc"
    [ -f "$rc" ] || return 0
    grep -qF "$dir" "$rc" 2>/dev/null && return 0

    {
        echo ""
        echo "# --- knudge completions ---"
        echo "fpath=(\"$dir\" \$fpath)"
    } >> "$rc"
    ok "fpath adicionado a $(tilde "$rc")"
}

# ── PATH ─────────────────────────────────────────────────────────────────────

add_path_line() {
    local file="$1"
    local line="export PATH=\"${INSTALL_DIR}:\${PATH}\""
    local marker="# --- knudge path ---"

    grep -qxF "$line" "$file" 2>/dev/null && return 0
    grep -qxF "$marker" "$file" 2>/dev/null && return 0

    if [ -s "$file" ] && [ "$(tail -c1 "$file" | wc -l)" -eq 0 ]; then
        echo "" >> "$file"
    fi
    {
        echo "$marker"
        echo "$line"
    } >> "$file"
    ok "PATH adicionado a $(tilde "$file")"
}

setup_path() {
    [ "${KD_NO_PATH:-0}" = "1" ] && return 0
    info "Verificando PATH..."
    if printf '%s' "$PATH" | tr ':' '\n' | grep -qxF "$INSTALL_DIR"; then
        ok "$(tilde "$INSTALL_DIR") já está no PATH"
        return 0
    fi

    local touched=0 file
    for file in \
        "${HOME}/.profile" \
        "${HOME}/.bashrc" \
        "${HOME}/.bash_profile" \
        "${HOME}/.zshrc" \
        "${ZDOTDIR:-${HOME}}/.zshrc"; do
        [ -f "$file" ] || continue
        touched=1
        add_path_line "$file"
    done

    # Nenhum rc existente: cria ~/.profile (lido por login shells) para garantir o PATH.
    if [ "$touched" -eq 0 ]; then
        : > "${HOME}/.profile"
        add_path_line "${HOME}/.profile"
    fi

    warn "Reinicie o shell ou rode: export PATH=\"${INSTALL_DIR}:\$PATH\""
}

# ── desinstalação ────────────────────────────────────────────────────────────

uninstall() {
    info "Invalidando binários de $(tilde "$INSTALL_DIR") (movidos para o lixo)..."
    for bin in "${BINARIES[@]}"; do
        trash "${INSTALL_DIR}/${bin}"
        trash "${INSTALL_DIR}/${bin}.exe"
        ok "Removido do PATH: $(tilde "${INSTALL_DIR}/${bin}")"
    done
    local data="${XDG_DATA_HOME:-$HOME/.local/share}"
    local config="${XDG_CONFIG_HOME:-$HOME/.config}"
    trash "${data}/bash-completion/completions/kd"
    trash "${data}/zsh/site-functions/_kd"
    trash "${config}/fish/completions/kd.fish"
    ok "Completions invalidadas (config global e notas preservadas)."
    ok "Lixo recuperável em $(tilde "${CACHE_ROOT}/trash")"
}

# ── main ─────────────────────────────────────────────────────────────────────

if [ "$DO_UNINSTALL" -eq 1 ]; then
    uninstall
    exit 0
fi

TARGET="${KD_TARGET:-$(detect_target)}"
if [ "$FROM_SOURCE" -eq 0 ]; then
    require curl
    info "Detectado: ${TARGET}"
    VERSION="$(resolve_version)"
    info "Instalando knudge ${VERSION} (${TARGET}) em $(tilde "$INSTALL_DIR")..."
    check_upgrade
    download_release "$TARGET"
else
    info "Instalando knudge a partir do source em $(tilde "$INSTALL_DIR")..."
    build_from_source
fi

install_binaries
verify_binaries
setup_global_config
setup_completions
setup_path

echo ""
info "Pronto! Dois binários instalados:"
echo "  kd          → CLI (kd init, kd prime, kd ask, kd write, ...)"
echo "  knudge-mcp  → servidor MCP (JSON-RPC 2.0 sobre stdio)"
echo ""
info "Próximos passos:"
echo "  cd ~/meu-projeto"
echo "  kd init                     # funda .knudge/ e o AGENTS.md"
echo "  kd write --summary \"...\" --type fact --anchor src/x.rs   # statement em --summary; corpo é posicional"
echo "  kd ask \"...\"                # busca no corpus"
echo ""
echo "  # Config global (template copiado pelo \`kd init\`):"
echo "  #   $(tilde "${XDG_CONFIG_HOME:-$HOME/.config}/local/knudge/config.toml")"
echo "  # Config do projeto: .knudge/config.toml"
echo "  kd config set --key mcp.hints_cap --value 3 --global   # exemplo"
echo ""
echo "  # Busca semântica (opcional): baixa llama.cpp + modelo e sobe o servidor persistente"
echo "  kd maintenance watch-service --install"
echo ""
info "Docs: https://github.com/${REPO}"
