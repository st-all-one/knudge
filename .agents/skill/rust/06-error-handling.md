# 6 — Error Handling

## Estratégia: Errors são valores

Rust não tem exceções. Erros são valores que devem ser tratados:

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

## panic! (unrecoverable)

```rust
panic!("crash and burn");      // mensagem + backtrace
unreachable!();                // lógica impossível
unimplemented!();              // placeholder (compila, mas panica)
todo!();                       // igual unimplemented!()
```

### Quando usar panic!

- Exemplos e protótipos
- Testes (casos de erro devem panic)
- Invariantes definitivamente violados (índice known-safe)

```bash
RUST_BACKTRACE=1 cargo run    # backtrace completo
RUST_BACKTRACE=full cargo run # backtrace máximo
```

## Result (recoverable)

```rust
use std::fs::File;
use std::io::{self, Read};

fn read_username(path: &str) -> Result<String, io::Error> {
    let mut file = File::open(path)?;    // ? propaga erro
    let mut username = String::new();
    file.read_to_string(&mut username)?;
    Ok(username)
}
```

### O operador `?`

```rust
// Conciso
let file = File::open(path)?;

// Equivalente a:
let file = match File::open(path) {
    Ok(f) => f,
    Err(e) => return Err(e.into()),  // conversão automática via From
};
```

`?` funciona com `Result` e `Option`:

```rust
// Option
fn last_char(text: &str) -> Option<char> {
    text.lines().next()?.chars().last()
}

// Result
fn parse_int(s: &str) -> Result<i32, Box<dyn std::error::Error>> {
    let n: i32 = s.parse()?;   // ParseIntError → Box<dyn Error>
    Ok(n)
}
```

### Combinators

```rust
let len = result.map(|s| s.len());               // Result<usize, E>
let val = result.map_err(|e| format!("{e}"));    // Result<T, String>
let val = result.and_then(|s| s.parse::<i32>()); // Result<i32, E>
let val = result.or_else(|_| Ok(42));            // fallback
let val = result.unwrap_or("default");
let val = result.unwrap_or_else(|_| compute_default());
let ok = result.is_ok();
let err = result.is_err();
```

## Error Types

### Box<dyn Error> (simples)

```rust
use std::error::Error;
use std::fs;

fn read_config() -> Result<String, Box<dyn Error>> {
    let content = fs::read_to_string("config.toml")?; // io::Error
    let parsed = content.parse::<i32>()?;              // ParseIntError
    Ok(content)
}
```

### Custom Error Enum (recomendado)

```rust
#[derive(Debug)]
enum AppError {
    Io(io::Error),
    Parse(String),
    NotFound { path: String },
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO error: {e}"),
            AppError::Parse(s) => write!(f, "parse error: {s}"),
            AppError::NotFound { path } => write!(f, "not found: {path}"),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for AppError {
    fn from(e: io::Error) -> Self { AppError::Io(e) }
}
```

### thiserror (recomendado — crate)

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("parse error for {input}: {source}")]
    Parse {
        input: String,
        #[source]
        source: ParseIntError,
    },

    #[error("not found: {path}")]
    NotFound { path: String },
}
```

- `#[from]` gera `From<io::Error>`
- `#[error("...")]` gera `Display`
- `#[source]` gera `source()`

### anyhow (recomendado — crate)

Para código de aplicação (não bibliotecas):

```rust
use anyhow::{Context, Result, bail};

fn read_config(path: &str) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {path}"))?;
    let config: Config = toml::from_str(&content)
        .with_context(|| "invalid config format")?;

    if !config.valid {
        bail!("config validation failed");
    }
    Ok(config)
}

// Custom error check
if value < 0 {
    anyhow::bail!("negative value: {value}");
}
```

## Error Handling em main

```rust
// main pode retornar Result
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = read_config("config.toml")?;
    run(config)?;
    Ok(())
}

// Ou usar anyhow
fn main() -> anyhow::Result<()> {
    let config = read_config("config.toml")?;
    run(config)?;
    Ok(())
}
```

## Padrão: Tipos de Validação

```rust
// Em vez de validar runtime, codifique no tipo:
pub struct Guess {
    value: i32,
}

impl Guess {
    pub fn new(value: i32) -> Result<Guess, String> {
        if value < 1 || value > 100 {
            return Err("must be 1-100".into());
        }
        Ok(Guess { value })
    }

    pub fn value(&self) -> i32 { self.value }
}

// Uso → o type system garante que Guess sempre é válido
fn play(guess: Guess) {
    // guess.value está sempre entre 1-100
}
```

## Boas Práticas

1. **Use `?`** em vez de `unwrap()` — propaga erro para o caller
2. **Use `anyhow`** para aplicações, **`thiserror`** para bibliotecas
3. **Crie error types customizados** para erros que o caller precisa
   diferenciar
4. **Use `Box<dyn Error>`** só para protótipos ou quando os erros
   não importam
5. **`unwrap()`/`expect()`** só quando o erro é impossível ou em
   testes
6. **Context é seu amigo** — `with_context()` dá significado ao erro
7. **Prefira `bail!()`** a `return Err(...)` com anyhow
8. **Não abuse de panic** — Result é mais composto e testável
9. **Error types devem ser `Send + Sync`** para async e threading
10. **Implemente `source()`** para erros que encadeiam
