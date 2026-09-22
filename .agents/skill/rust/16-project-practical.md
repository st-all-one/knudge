# 16 — Projeto Prático: Guia de Arquitetura

## Decisões Arquiteturais

### Quando usar o quê

| Problema | Solução |
|----------|---------|
| CLI simples | `clap` + `std::io` |
| Web server | `axum` + `tokio` + `sqlx` |
| API client | `reqwest` + `serde` |
| GUI desktop | `egui` / `dioxus` / `slint` |
| GUI web | `leptos` / `yew` / `dioxus` |
| Embedded | `no_std` + `embassy` |
| WebAssembly | `wasm-bindgen` + `wasm-pack` |
| Game | `bevy` |
| Async runtime | `tokio` (default), `async-std`, `smol` |
| Serialization | `serde` (standard) |
| Error handling | `thiserror` (lib) / `anyhow` (app) |
| Logging | `tracing` / `log` + `env_logger` |
| Testing | `criterion` (bench) / `proptest` (property) |
| Database | `sqlx` (async, compile-time checked) / `diesel` |
| ORM | `sea-orm` / `diesel` |
| HTTP client | `reqwest` (high-level) / `hyper` (low-level) |
| HTTP server | `axum` / `actix-web` / `warp` |

### Padrão: Biblioteca + Binário

```rust
// src/lib.rs — toda a lógica
pub mod config;
pub mod models;
pub mod services;
pub mod db;

// src/main.rs — thin wrapper
use my_app::{config, services};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = config::load()?;
    services::run(cfg).await?;
    Ok(())
}
```

### Separando em Crates

```
my_app/
├── Cargo.toml                # workspace root (sem [package])
├── crates/
│   ├── core/                 # tipos, traits, interfaces
│   ├── domain/               # lógica de negócio
│   ├── infra/                # DB, HTTP client, file system
│   ├── api/                  # endpoints HTTP (axum)
│   └── cli/                  # CLI (clap)
```

## Criação de Projeto Passo a Passo

```bash
cargo new my_app
cd my_app
cargo add tokio --features full
cargo add serde --features derive
cargo add anyhow thiserror
cargo add clap --features derive
cargo add tracing tracing-subscriber
```

### Cargo.toml final

```toml
[package]
name = "my_app"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
anyhow = "1"
thiserror = "2"
clap = { version = "4", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
reqwest = { version = "0.12", features = ["json"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres"] }
axum = "0.7"

[dev-dependencies]
criterion = "0.5"
tokio-test = "0.4"

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = "symbols"
```

### Entry Point (main.rs)

```rust
use clap::Parser;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "my_app", about = "My Rust application")]
struct Cli {
    #[arg(short, long)]
    config: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Subcommand)]
enum Command {
    Serve { #[arg(default_value = "3000")] port: u16 },
    Migrate,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Command::Serve { port }) => run_server(port).await?,
        Some(Command::Migrate) => run_migrations().await?,
        None => run_server(3000).await?,
    }

    Ok(())
}
```

## Padrões de Projeto em Rust

### Builder Pattern

```rust
#[derive(Default)]
struct ConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    timeout: Option<Duration>,
}

impl ConfigBuilder {
    fn host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }
    fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
    fn build(self) -> Config {
        Config {
            host: self.host.unwrap_or("localhost".into()),
            port: self.port.unwrap_or(8080),
            timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
        }
    }
}
```

### Newtype Pattern

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    pub fn new(s: &str) -> Result<Self, String> {
        if s.contains('@') {
            Ok(Email(s.to_string()))
        } else {
            Err("invalid email".into())
        }
    }
    pub fn as_str(&self) -> &str { &self.0 }
}

// Serde support via custom impl
impl<'de> Deserialize<'de> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        Email::new(&s).map_err(serde::de::Error::custom)
    }
}
```

### Repository Pattern

```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError>;
    async fn save(&self, user: &User) -> Result<(), AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

// Postgres implementation
pub struct PgUserRepository {
    pool: PgPool,
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as!(
            User,
            "SELECT id, name, email FROM users WHERE id = $1",
            id,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }
    // ...
}
```

### Error Handling Pattern

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

// Axum integration
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

### State Pattern (Tipos de Estado)

```rust
// Em vez de runtime checks, codifique no type system
mod draft { pub struct Draft; }
mod published { pub struct Published; }

struct Post<State> {
    title: String,
    content: String,
    _state: std::marker::PhantomData<State>,
}

impl Post<draft::Draft> {
    pub fn new(title: &str) -> Self {
        Self { title: title.into(), content: String::new(), _state: PhantomData }
    }
    pub fn add_content(&mut self, text: &str) { self.content.push_str(text); }
    pub fn publish(self) -> Post<published::Published> {
        Post { title: self.title, content: self.content, _state: PhantomData }
    }
}

impl Post<published::Published> {
    pub fn content(&self) -> &str { &self.content }
}
```

## Logging e Observabilidade

```rust
use tracing::{info, error, warn, debug, span, Level, instrument};

#[instrument(skip(pool))]
async fn process_user(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    info!("processing user {user_id}");

    let user = find_user(pool, user_id).await?;
    debug!(?user, "found user");

    warn!("deprecated code path for user {user_id}");

    Ok(())
}

// Structured fields
info!(
    user_id = %user.id,
    email = %user.email,
    duration_ms = %elapsed.as_millis(),
    "request completed"
);
```

## Performance

```rust
// Pre-allocate
let mut v = Vec::with_capacity(1000);
let mut s = String::with_capacity(100);

// Avoid clones
fn process(data: &[u8]) { /* ... */ }  // borrow instead of &Vec<u8>

// Small string optimization (smol_str, smartstring)
// interned strings (intern)

// Profile
// cargo build --release && perf record ./target/release/my_app
// cargo flamegraph

// Memory
// jemalloc / mimalloc allocator
use tikv_jemallocator::Jemalloc;
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

// Cache-friendly data layout
// AoS → SoA para acesso sequencial
struct AoS { x: f32, y: f32, z: f32 } // Array of Structs
struct SoA { xs: Vec<f32>, ys: Vec<f32>, zs: Vec<f32> } // Struct of Arrays
```

## CI/CD Completo

```yaml
name: CI
on: [push, pull_request]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: stable
          components: clippy, rustfmt
      - run: cargo fmt -- --check
      - run: cargo clippy -- -D warnings
      - run: cargo check

  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: cargo test
      - run: cargo test --doc

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: cargo install cargo-tarpaulin
      - run: cargo tarpaulin --out Xml
      - uses: codecov/codecov-action@v4

  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: cargo install cargo-audit
      - run: cargo audit

  build:
    name: Build Release
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: cargo build --release
      - uses: actions/upload-artifact@v4
        with:
          name: binary
          path: target/release/my_app
```

## Boas Práticas (Projeto)

1. **Organizar por domínio**, não por camada técnica
2. **Lib + thin binary** — facilita testes e reuso
3. **Separar commands/dao/services** em módulos claros
4. **Usar `anyhow` em app, `thiserror` em lib**
5. **Injeção de dependência via traits** para testabilidade
6. **Testes de integração com `sqlx::test` ou testcontainers**
7. **Documentar decisões** em ADRs (Architecture Decision Records)
8. **`cargo deny`** para auditoria automática
9. **Benchmarks via criterion** para código crítico
10. **Perfil de release otimizado** antes de deploy
