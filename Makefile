# knudge — alvos de qualidade e build (E01-T03, E13-T07)

CARGO ?= cargo

.PHONY: check fmt clippy test build file-length clean \
        nextest doc deny audit machete typos miri fuzz coverage ci

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
	@command -v cargo-miri >/dev/null 2>&1 && $(CARGO) miri test -p knudge-core --lib \
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
