#!/usr/bin/env bash
# bump-version — atualiza a versão do knudge em todos os pontos do repositório.
#
# Uso:
#   ./scripts/bump-version.sh v0.2.1        # aceita vX.Y.Z ou X.Y.Z
#   make update-version VERSION=v0.2.1
#
# Pontos atualizados (o portão da release exige tag == Cargo.toml):
#   - Cargo.toml  ([workspace.package] version)
#   - Cargo.lock  (crates knudge-core / knudge-cli / knudge-mcp)
#   - crates/knudge-cli/tests/golden/json_prime.json e json_version.json
#   - install.sh e README.md  (exemplos `VERSION=vX.Y.Z`)
#   - CHANGELOG.md  (finaliza `[Não publicado]` → `[X.Y.Z] - <data>`)
#
# Nunca usa `rm` (reusa com `mv`); recusa versão igual ou inválida.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

raw="${1:-}"
if [ -z "$raw" ]; then
    echo "uso: $0 vX.Y.Z" >&2
    exit 2
fi
new="${raw#v}"
if ! printf '%s' "$new" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "versão inválida: $raw (use X.Y.Z ou vX.Y.Z)" >&2
    exit 2
fi

old="$(grep -m1 -E '^version = "[0-9]' Cargo.toml | cut -d'"' -f2)"
if [ "$old" = "$new" ]; then
    echo "já está na versão $new; nada a fazer"
    exit 0
fi
echo "atualizando versão: $old -> $new"

# Substitui `from` por `to` no arquivo (via tmp + mv; sem `rm`).
replace() {
    local file=$1 from=$2 to=$3
    [ -f "$file" ] || return 0
    sed "s|$from|$to|g" "$file" >"$file.bump.tmp"
    mv "$file.bump.tmp" "$file"
}

# 1. Cargo.toml — só a linha de versão do workspace.
sed -E "s|^version = \"$old\"$|version = \"$new\"|" Cargo.toml >Cargo.toml.bump.tmp
mv Cargo.toml.bump.tmp Cargo.toml

# 2. Cargo.lock — os três crates do workspace.
awk -v new="$new" '
    /^name = "knudge-core"$/ || /^name = "knudge-cli"$/ || /^name = "knudge-mcp"$/ {
        print
        if ((getline line) > 0) {
            if (line ~ /^version = /) print "version = \"" new "\""
            else print line
        }
        next
    }
    { print }
' Cargo.lock >Cargo.lock.bump.tmp
mv Cargo.lock.bump.tmp Cargo.lock

# 3. Goldens do prime/version (contrato de bytes).
replace crates/knudge-cli/tests/golden/json_prime.json "\"version\":\"$old\"" "\"version\":\"$new\""
replace crates/knudge-cli/tests/golden/json_version.json "\"version\":\"$old\"" "\"version\":\"$new\""

# 4. Exemplos de instalação.
replace install.sh "VERSION=v$old" "VERSION=v$new"
replace README.md "VERSION=v$old" "VERSION=v$new"

# 5. CHANGELOG — finaliza a seção `[Não publicado]`, mantendo uma nova no topo.
if grep -q '^## \[Não publicado\]$' CHANGELOG.md; then
    today="$(date +%Y-%m-%d)"
    awk -v new="$new" -v date="$today" '
        /^## \[Não publicado\]$/ && !done {
            print "## [Não publicado]"
            print ""
            print "## [" new "] - " date
            done=1
            next
        }
        { print }
    ' CHANGELOG.md >CHANGELOG.md.bump.tmp
    mv CHANGELOG.md.bump.tmp CHANGELOG.md
fi

echo "pronto. Próximos passos:"
echo "  git add -A && git commit -S -m \"chore(release): $new\""
echo "  git tag -a v$new -m \"knudge v$new\""
echo "  git push origin main && git push origin v$new"
