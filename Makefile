# knudge — alvos de qualidade e build (E01-T03, E13-T07)

CARGO ?= cargo
PREFIX ?= $(HOME)/.local
BINDIR ?= $(PREFIX)/bin

.PHONY: check fmt clippy test build file-length clean install uninstall \
        update-version nextest doc deny audit machete typos miri fuzz coverage ci dist bench bench-quick

## Portão completo local: formatação, lints, testes e gate de tamanho de arquivo.
check: fmt clippy test file-length

## Verifica formatação sem alterar.
fmt:
	$(CARGO) fmt --all -- --check

## Lints de todos os alvos com warnings como erro (R44).
clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

## Testes do workspace (inclui doc-tests).
test:
	$(CARGO) test --workspace

## Build de debug do workspace.
build:
	$(CARGO) build --workspace

## Nenhum arquivo de produção passa de 300 linhas (D92).
file-length:
	./scripts/check_file_length.sh

clean:
	$(CARGO) clean

# --- Instalação local (source) ---

## Compila em release e instala `kd` + `knudge-mcp`, a config global e as completions.
## Use PREFIX=... / BINDIR=... para mudar o destino (default: ~/.local/bin).
install:
	INSTALL_DIR="$(DESTDIR)$(BINDIR)" ./install.sh --from-source

## Remove os binários e as completions (preserva a config global e as notas).
uninstall:
	INSTALL_DIR="$(DESTDIR)$(BINDIR)" ./install.sh --uninstall

# --- Versão (tag == Cargo.toml, exigido pelo portão da release) ---

## Atualiza a versão em Cargo.toml/Cargo.lock/goldens/install.sh/README/CHANGELOG.
## Uso: make update-version VERSION=v0.2.1   (ou: make update-version v0.2.1)
update-version:
	./scripts/bump-version.sh "$(or $(VERSION),$(filter v%,$(MAKECMDGOALS)))"

# Absorve o argumento posicional de `make update-version vX.Y.Z` (sem regra real).
v%:
	@:

# --- Empacotamento local (mesmo padrão do GitHub Release) ---

## Compila release e empacota a plataforma atual em `dist/` (`knudge-<versão>-<target>.*`).
dist:
	./scripts/package.sh

# --- Alvos extras (CI / verificação dinâmica). Pulam se a ferramenta não estiver instalada. ---

## Runner paralelo (E13-T07).
nextest:
	@command -v cargo-nextest >/dev/null 2>&1 && $(CARGO) nextest run --workspace \
		|| echo "nextest ausente; pule (instale com: cargo install cargo-nextest --locked)"

## Supply chain: licenças/advisories/fontes (E13-T09).
deny:
	@command -v cargo-deny >/dev/null 2>&1 && $(CARGO) deny check \
		|| echo "cargo-deny ausente; pule (cargo install cargo-deny --locked)"

audit:
	@command -v cargo-audit >/dev/null 2>&1 && $(CARGO) audit \
		|| echo "cargo-audit ausente; pule (cargo install cargo-audit --locked)"

## Dependências não usadas (orçamento R43).
machete:
	@command -v cargo-machete >/dev/null 2>&1 && $(CARGO) machete \
		|| echo "cargo-machete ausente; pule (cargo install cargo-machete --locked)"

## Verificação ortográfica (E13-T09).
typos:
	@command -v typos >/dev/null 2>&1 && typos \
		|| echo "typos ausente; pule (https://github.com/crate-ci/typos)"

## Verificação dinâmica de UB nos crates puros (E13-T08).
miri:
	@command -v cargo-miri >/dev/null 2>&1 && PROPTEST_DISABLE_FAILURE_PERSISTENCE=1 $(CARGO) miri test -p knudge-core --lib -- --skip adapters:: \
		|| echo "miri ausente; pule (rustup +nightly component add miri)"

## Fuzz smoke dos parsers (E13-T08).
fuzz:
	@command -v cargo-fuzz >/dev/null 2>&1 \
		&& $(CARGO) fuzz build \
		|| echo "cargo-fuzz ausente; pule (cargo install cargo-fuzz --locked)"

## Cobertura de linhas (E13-T09, observação).
coverage:
	@command -v cargo-llvm-cov >/dev/null 2>&1 \
		&& $(CARGO) llvm-cov --workspace --summary-only \
		|| echo "cargo-llvm-cov ausente; pule (cargo install cargo-llvm-cov --locked)"

## Portão do CI: check + doc-tests + extras disponíveis.
ci: check nextest deny audit machete typos

# --- Bancada de benchmark (fora do workspace, observação — E13-T09) ---

## Compara componentes (micromb) e ações (ponta-a-ponta) em corpora 200 e 1000.
## Resultado em tabela Markdown; `--json`/`--out` guardam artefatos.
bench:
	$(CARGO) build --release -p knudge-cli
	$(CARGO) run --release --manifest-path bench/Cargo.toml -- all \
		--kd target/release/kd --sizes 200,1000 --samples 8 --out bench/ULTIMO.md

## Versão rápida (1 corpus, 5 amostras).
bench-quick:
	$(CARGO) build --release -p knudge-cli
	$(CARGO) run --release --manifest-path bench/Cargo.toml -- all \
		--kd target/release/kd --quick
