# knudge — alvos de qualidade e build (E01-T03)

CARGO ?= cargo

.PHONY: check fmt clippy test build file-length clean

## Portão completo: formatação, lints, testes e gate de tamanho de arquivo.
check: fmt clippy test file-length

## Verifica formatação sem alterar.
fmt:
	$(CARGO) fmt --all -- --check

## Lints de todos os alvos com warnings como erro (R44).
clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

## Testes do workspace.
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
