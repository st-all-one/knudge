//! Implementações reais (`std`) das portas.
//!
//! **O domínio nunca importa este módulo** — só `knudge-cli`/`knudge-mcp` montam os adaptadores.
//! É aqui que métodos "impuros" (`SystemTime::now`, `std::env::var`, …) são permitidos com
//! `#[allow(..., reason = ...)]`; em todo o resto eles são proibidos pelo `clippy.toml` (R01/R03).

pub mod clock;
pub mod env;
pub mod fs;
pub mod git;
pub mod rng;

pub use clock::SystemClock;
pub use env::StdEnv;
pub use fs::StdFs;
pub use git::StdGit;
pub use rng::ThreadRng;
