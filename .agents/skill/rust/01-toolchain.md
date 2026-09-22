# 1 — Toolchain, Ferramentas e Setup

## rustup (gerenciador de toolchain)

```bash
# Instalar
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Gerenciar toolchains
rustup show                     # toolchains e targets instalados
rustup update                   # atualizar tudo
rustup default nightly          # mudar default para nightly
rustup toolchain install nightly
rustup target add wasm32-unknown-unknown

# Múltiplas versões
rustup run nightly cargo build  # executar com nightly
cargo +nightly build            # shorthand
```

## rustc (compilador)

```rust
// hello.rs
fn main() {
    println!("Hello, Rust 1.97.0!");
}
```

```bash
rustc hello.rs                  # compilação direta
rustc -O hello.rs               # com otimizações
rustc --edition 2024 hello.rs   # edição específica
```

## Cargo (package manager + build system)

```bash
cargo new my_project              # novo projeto binário
cargo new my_lib --lib            # novo projeto lib
cargo init                        # em diretório existente
cargo build                       # debug build
cargo build --release             # release build (otimizado)
cargo check                       # só verifica tipos (mais rápido que build)
cargo run                         # build + executar
cargo test                        # rodar testes
cargo doc --open                  # documentação
cargo clippy                      # lints extras
cargo fmt                         # formatação automática
```

### Cargo.toml mínimo

```toml
[package]
name = "my_app"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
description = "A brief description"
license = "MIT OR Apache-2.0"

[dependencies]
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
```

### Optimized release profile

```toml
[profile.release]
opt-level = 3          # -O3
lto = "fat"            # link-time optimization
codegen-units = 1      # melhor otimização (compilação mais lenta)
strip = "symbols"      # remover símbolos de debug
debug = false          # sem info de debug em release
panic = "abort"        # binário menor, sem unwinding
```

## rust-analyzer (IDE)

- LSP server oficial para Rust
- Suportado por VS Code, Neovim (via `neovim/nvim-lspconfig`),
  JetBrains (intrinsic), Zed, Helix
- Funcionalidades: autocomplete, go-to-definition, find references,
  inline type hints, code actions, rename, diagnostics

## Clippy (lints)

```bash
cargo clippy                     # todos os lints
cargo clippy -- -W clippy::pedantic   # mais rigoroso
cargo clippy -- -W clippy::nursery    # ainda instáveis
cargo clippy --fix               # auto-fix
```

```toml
# Cargo.toml (opcional - lint config)
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[lints.clippy]
pedantic = "warn"
unwrap_used = "deny"
```

## rustfmt (formatação)

```toml
# rustfmt.toml
max_width = 100
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Max"
```

```bash
cargo fmt                    # formatar
cargo fmt -- --check         # CI: verificar formatação
```

## Ferramentas essenciais do ecossistema

| Ferramenta | Instalação | Para que serve |
|------------|-----------|----------------|
| `cargo-edit` | built-in (`cargo add`) | Gerenciar dependências |
| `cargo-watch` | `cargo install cargo-watch` | Auto-compilar ao salvar |
| `cargo-tarpaulin` | `cargo install cargo-tarpaulin` | Cobertura de código |
| `cargo-audit` | `cargo install cargo-audit` | Auditoria de segurança |
| `cargo-deny` | `cargo install cargo-deny` | Licenças + segurança |
| `cargo-udeps` | `cargo install cargo-udeps` | Dependências não usadas |
| `cargo-expand` | `cargo install cargo-expand` | Expandir macros |
| `criterion` | `cargo add criterion --dev` | Benchmarks |
| `nextest` | `cargo install cargo-nextest` | Test runner mais rápido |
| `sccache` | `cargo install sccache` | Cache de compilação shared |
| `typos` | `cargo install typos-cli` | Spell checker |
| `cargo-machete` | `cargo install cargo-machete` | Deps não usadas |
| `cargo-outdated` | `cargo install cargo-outdated` | Deps desatualizadas |

## CI básico (GitHub Actions)

```yaml
name: CI
on: [push, pull_request]
jobs:
  ci:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: cargo fmt -- --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
      - run: cargo build --release
```

## Project Layout Padrão

```
my_project/
├── Cargo.toml          # manifesto
├── Cargo.lock          # lockfile (commit!)
├── rustfmt.toml        # opcional
├── .cargo/
│   └── config.toml     # opcional
├── src/
│   ├── main.rs         # entry point binário
│   ├── lib.rs          # entry point lib
│   └── bin/            # binários extras
│       └── other.rs
├── tests/              # testes de integração
│   └── integration.rs
├── benches/            # benchmarks
│   └── my_bench.rs
├── examples/           # exemplos
│   └── example.rs
└── build.rs            # build script (opcional)
```
