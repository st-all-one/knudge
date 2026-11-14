# 15 — Cargo em Profundidade

## Cargo.toml Completo

```toml
[package]
name = "my_app"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
authors = ["Alice <alice@example.com>"]
description = "A brief description"
license = "MIT OR Apache-2.0"
license-file = "LICENSE"
readme = "README.md"
homepage = "https://example.com"
repository = "https://github.com/user/repo"
documentation = "https://docs.rs/my_app"
keywords = ["cli", "tool"]
categories = ["command-line-utilities"]
publish = false   # impede publish acidental
resolver = "2"
default-run = "my_app"  # qual binário rodar com cargo run
build = "build.rs"
links = "my_native_lib"
exclude = ["ci/", "*.png"]
include = ["src/**/*", "Cargo.toml"]
```

### Targets

```toml
[lib]
name = "my_lib"
path = "src/lib.rs"
crate-type = ["lib", "cdylib"]  # lib + shared library
doc = true
doctest = true
bench = true
test = true

[[bin]]
name = "cli"
path = "src/bin/cli.rs"
required-features = ["cli"]

[[example]]
name = "example1"
path = "examples/example1.rs"

[[test]]
name = "integration"
path = "tests/integration.rs"

[[bench]]
name = "benchmark"
harness = false  # desativa harness padrão
```

## Dependencies

```toml
[dependencies]
# Crates.io (semver caret por default)
serde = "1"                         # ^1.0.0
serde = "1.2"                       # ^1.2.0
serde = "=1.2.3"                    # exato
serde = "~1.2"                      # >=1.2.0, <1.3.0
serde = ">=1.0, <2.0"              # range

# Git
tokio = { git = "https://github.com/tokio-rs/tokio", branch = "master" }
my_crate = { git = "https://github.com/user/repo", rev = "abc123" }

# Path
local_dep = { path = "../local_dep" }

# Multiple sources (dev usa path, publish usa version)
my_crate = { path = "../my_crate", version = "0.1" }

# Platform-specific
[target.'cfg(windows)'.dependencies]
winapi = "0.3"

[target.'cfg(target_arch = "x86_64")'.dependencies]
special = "0.1"

# Renaming (package name different from dependency key)
serde_json = { package = "serde_json_alt", version = "1" }

# Optional
tokio = { version = "1", optional = true }

# Dev-only
[dev-dependencies]
criterion = "0.5"

# Build-only
[build-dependencies]
cc = "1"
```

### Resolver

```toml
[package]
resolver = "2"  # Edition 2021+, recomendado
```

| Resolver | Comportamento |
|----------|---------------|
| 1 (2015) | Unifica features entre targets |
| 2 (2021) | Features separados por target, build-deps não unificam |
| 3 (nightly) | MSRV-aware |

## Profiles

```toml
[profile.dev]
opt-level = 0           # sem otimização
debug = true            # símbolos completos
debug-assertions = true
overflow-checks = true
incremental = true      # compilação incremental
codegen-units = 256     # mais unidades = compilação mais rápida
lto = false
panic = "unwind"

# Novo em 1.99: profile "debug" built-in
# Preparação para transição do profile dev
# Atualmente idêntico ao dev
[profile.debug]
inherits = "dev"

[profile.release]
opt-level = 3           # -O3
debug = false
debug-assertions = false
overflow-checks = false
lto = "fat"             # link-time optimization
codegen-units = 1       # melhor otimização
strip = "symbols"       # remove símbolos
panic = "abort"         # binário menor (sem unwinding)
incremental = false

[profile.bench]
inherits = "release"    # herda de release
debug = 1               # line tables only

# Override por pacote
[profile.dev.package.'*']
opt-level = 2           # otimiza todas deps em dev

[profile.dev.package.serde]
opt-level = 3

# Build override (para build scripts)
[profile.dev.build-override]
opt-level = 3
```

> **Rust 1.99+**: Em CI (detectado pela env var `CI`), a compilação
> incremental é desabilitada por default para builds mais confiáveis.

## Build-dir e cache (1.100+)

Cargo separa **artefatos finais** (`target-dir`: bins, libs, docs) de
**intermediários** (`build-dir`: incremental, saídas de build scripts).
Por default ambos apontam para `target/`; para separar:

```toml
[build]
build-dir = "/tmp/build-cache"   # ou CARGO_BUILD_BUILD_DIR
target-dir = "target"
```

Na 1.100 o `build-dir` passou a aplicar o hash de configuração
(`--features`, profile, rustflags) no **diretório pai**, organizando por
etapa de build em vez de tipo de arquivo. Efeitos:

- builds com features diferentes não colidem mais no `build-dir`;
- compartilhar `build-dir` entre worktrees deixa de causar rebuilds
  espúrios;
- o layout é **interno e instável**: ferramentas devem localizar
  artefatos via `cargo build --message-format=json`, não assumindo
  `target/debug/deps/...`;
- scripts/CI que dependiam de workarounds antigos podem quebrar — ver
  `cargo#17287`.

Isso desbloqueia GC de cache, deduplicação/caching cross-workspace e
locking mais fino.

## Workspaces

```toml
# Cargo.toml (root)
[workspace]
members = [
    "crates/*",
    "tools/cli",
]
exclude = ["legacy"]
resolver = "2"
default-members = ["crates/app", "tools/cli"]

# Shared metadata
[workspace.package]
version = "0.1.0"
edition = "2024"
authors = ["Alice <alice@example.com>"]

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
thiserror = "2"

[workspace.lints]
rust.unsafe_code = "forbid"
```

```toml
# crates/app/Cargo.toml
[package]
name = "app"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
tokio.workspace = true
my_utils = { path = "../utils" }

# Rust 1.99+: Workspace members em edition 2024+ podem override
# default-features de dependência herdada (RFC 3945)
serde = { workspace = true, default-features = false }
```

## Features

```toml
[features]
default = ["std"]
std = []
serde = ["dep:serde"]
derive = []
full = ["std", "serde", "derive", "async"]

# Conditional dependencies
[dependencies]
serde = { version = "1", optional = true, features = ["derive"] }
tokio = { version = "1", optional = true, default-features = false }

# Weak dependencies (Edition 2021+)
# "rgb?/serde" = depende de rgb, mas não ativa rgb
```

```rust
// Feature gates
#[cfg(feature = "serde")]
impl Serialize for MyType {}

#[cfg(not(feature = "std"))]
extern crate alloc;
```

## Build Scripts

```rust
// build.rs
fn main() {
    // Gera cfg condicional
    println!("cargo::rustc-cfg=has_foo");

    // Adiciona link search path
    println!("cargo::rustc-link-search=/path/to/libs");

    // Link library
    println!("cargo::rustc-link-lib=static=foo");
    println!("cargo::rustc-link-arg=-Wl,-rpath,/path/to");

    // Re-run conditions
    println!("cargo::rerun-if-changed=src/ffi/wrapper.h");
    println!("cargo::rerun-if-env-changed=LLVM_CONFIG_PATH");

    // Link args
    println!("cargo::rustc-link-arg-bins=-static");
    println!("cargo::rustc-link-arg-tests=-lm");

    // Erros (build falha)
    println!("cargo::error=missing dependency: foo");

    // Generate code
    let out_dir = std::env::var("OUT_DIR").unwrap();
    std::fs::write(format!("{out_dir}/generated.rs"), "pub fn gen() {}").unwrap();
}

// cc crate para compilar C
fn main() {
    cc::Build::new()
        .file("src/c_code.c")
        .compile("my_c_code");
}
```

## Publishing

```bash
cargo login                              # autentica
cargo publish --dry-run                  # simula
cargo package --list                     # lista arquivos do .crate
cargo publish                            # publica
cargo yank --version 1.0.3               # remove da lista
cargo yank --version 1.0.3 --undo        # desfaz yank
cargo owner --add github:myorg:myrepo    # add owner
```

> **Rust 1.100**: `cargo install` usa por default o `Cargo.lock`
> embutido no `.crate`; `--locked` virou no-op.

## Dependency cooldowns (`min-publish-age`, 1.100+)

Versões maliciosas costumam ser detectadas em horas/dias. O resolver
pode ignorar versões novas por um período:

```toml
# .cargo/config.toml
[registry]
global-min-publish-age = "3 days"
```

```console
$ cargo update
     Locking 2 packages to highest compatible version as of min-publish-age
      Adding cc v1.11.0 (available: v1.18.0, published 3 hours ago)
```

- Só afeta versões **novas**: o que já está no `Cargo.lock` é mantido.
- `resolver.incompatible-publish-age` controla o comportamento
  (`deny` default; `allow` ignora o cooldown).
- Override pontual:
  `CARGO_RESOLVER_INCOMPATIBLE_PUBLISH_AGE=allow cargo update clap --precise 4.6.9`
  (a versão fica gravada no lock).
- Por registry: `registries.<name>.min-publish-age`; aceita `"0"` e
  `"N seconds|minutes|hours|days|weeks|months"`.
- `cargo install` **ignora** o cooldown (crate e dependências); pine
  `cargo install foo@1.2.3` se precisar.
- Registries sem `pubtime` no índice nunca são seguradas.
- É redução de janela, não defesa completa: continue revisando updates
  e usando `cargo audit`/`cargo deny`.

## Config (.cargo/config.toml)

```toml
# ~/.cargo/config.toml ou .cargo/config.toml no projeto

[alias]
c = "check"
b = "build"
r = "run"
t = "test"
cl = "clippy"

[build]
jobs = 4
rustc-wrapper = "sccache"
target-dir = "/tmp/cargo-targets"
incremental = true

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "target-cpu=native"]

[env]
RUST_LOG = "debug"

[http]
proxy = "http://proxy:8080"
timeout = 30

[net]
git-fetch-with-cli = true
offline = true

[term]
quiet = false
verbose = true
color = "auto"

[registries.my-registry]
index = "https://my-registry.com/index"
token = "..."

[registry]
default = "my-registry"

[source.crates-io]
replace-with = "mirror"

[source.mirror]
registry = "https://mirror.example.com/cargo"
```

> **Rust 1.100**: chaves `[target]` com `.` no nome (ex.:
> `thumbv8m.main-none-eabihf`) precisam de aspas:
> `[target."thumbv8m.main-none-eabihf"]`. A forma sem aspas ainda
> funciona para os targets atuais, mas emite future-incompatibility
> warning.

### Config via CLI

```bash
cargo build --config 'build.jobs=8'
cargo run --config 'registries.my-registry.index="https://..."'
cargo test --config /path/to/config.toml
```

## Environment Variables

```bash
# Lidas pelo Cargo
CARGO_HOME=~/.cargo
CARGO_TARGET_DIR=/tmp/target
RUSTFLAGS="-C target-cpu=native"
RUSTC_WRAPPER=sccache
CARGO_INCREMENTAL=1

# Setadas para o build
CARGO_MANIFEST_DIR=/project
CARGO_PKG_VERSION=0.1.0
CARGO_PKG_NAME=my_app
CARGO_CRATE_NAME=my_app
CARGO_BIN_NAME=cli
OUT_DIR=/project/target/debug/build/my_app-abc/out
TARGET=x86_64-unknown-linux-gnu
HOST=x86_64-unknown-linux-gnu
NUM_JOBS=8
OPT_LEVEL=0
DEBUG=true
PROFILE=debug
```

## Lints

```toml
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"
unused_imports = "deny"
clippy::pedantic = "warn"
clippy::unwrap_used = "deny"
clippy::expect_used = "deny"
clippy::panic = "deny"

# Workspace-level
[workspace.lints]
rust.unsafe_code = "forbid"

# Lints do próprio Cargo (1.100+)
[lints.cargo]
unused_dependencies = "deny"
```

### Cargo lints (1.100+)

Além de `rustc`/Clippy, o Cargo tem lints próprios, controlados por
`[lints.cargo]` (mesmas prioridades de `[lints]`):

| Lint | Grupo | Default | O que faz |
|---|---|---|---|
| `unused_dependencies` | style | warn | dependência que nenhum target usa |
| `unused_workspace_dependencies` | suspicious | warn | entrada de `[workspace.dependencies]` não usada |
| `unused_workspace_package_fields` | suspicious | warn | campo de `[workspace.package]` não herdado |
| `missing_lints_inheritance` | suspicious | warn | workspace tem `workspace.lints` mas o crate não usa `[lints] workspace = true` |
| `blanket_hint_mostly_unused` | suspicious | warn | `hint-mostly-unused` aplicado a `package."*"` |
| `manual_readme` | pedantic | allow | `package.readme` redundante |

Limitações de `unused_dependencies` (evita caçar falso positivo):

- só roda quando todos os targets que podem usar aquela tabela são
  compilados;
- `cargo check` cobre `[dependencies]` e `[build-dependencies]`;
- `cargo check --bin foo` **não** checa `[dependencies]` (outro bin pode
  usá-las); `[dev-dependencies]` nunca é checada;
- `path` dependencies só são checadas quando são o pacote selecionado;
- depender de uma transitiva só para ativar feature gera falso positivo
  — mova para `[target."cfg(false)".dependencies]`.

Herança em workspace:

```toml
# raiz
[workspace.lints.cargo]
unused_dependencies = "warn"

# membro
[lints]
workspace = true
```

## Boas Práticas (Cargo)

1. **Commit `Cargo.lock`** para aplicações (não para bibliotecas)
2. **`rust-version`** no Cargo.toml para MSRV claro
3. **`publish = false`** para apps internos
4. **Workspaces** para monorepos — dependências compartilhadas
5. **`resolver = "2"`** sempre (Edition 2021+)
6. **Features `default` mínimas** — não force dependências extras
7. **`dep:` prefix** para features que expõem dependências opcionais
8. **`[lints]` no workspace** para consistência
9. **`cargo deny`** para auditoria de licenças e segurança
10. **`[profile.release]` otimizado** — LTO, strip, codegen-units=1
11. **`[lints.cargo]`** para pegar dependências não usadas sem `cargo-udeps`
12. **`min-publish-age`** para cooldown de supply chain
