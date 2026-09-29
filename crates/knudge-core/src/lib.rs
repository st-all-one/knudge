//! # knudge-core
//!
//! Núcleo puro do knudge (D65). Sem terminal, `argv`, relógio global ou RNG global: todo acesso
//! ao mundo externo passa por uma **porta** em [`ports`]. As implementações reais ficam em
//! [`adapters`] e **não** são usadas pelo domínio.
//!
//! ## Uso como biblioteca
//!
//! Para incorporar o knudge num projeto — a mesma montagem feita por `knudge-cli`/`knudge-mcp` —
//! use a fachada [`Knudge`]:
//!
//! ```no_run
//! use knudge_core::Knudge;
//!
//! # fn main() -> knudge_core::Result<()> {
//! let kd = Knudge::builder().knowledge_dir(".a/b").open()?;
//! let index = kd.index()?;
//! # let _ = index;
//! # Ok(())
//! # }
//! ```
//!
//! O diretório de conhecimento é configurável (default `.knudge`); ver [`git::Project`].
//!
//! ## Invariantes (D92 / R01–R05)
//! - `#![forbid(unsafe_code)]`.
//! - Sem `Rc`/`RefCell`; estado compartilhado com `Arc<Mutex<_>>`.
//! - Sem `unwrap`/`expect`/`panic` no código de produção.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod adapters;
pub mod config;
pub mod corpus;
pub mod embeddings;
pub mod error;
pub mod git;
pub mod graph;
pub mod handoff;
pub mod health;
pub mod jsonl;
pub mod knowledge;
pub mod knudge;
pub mod lifecycle;
pub mod logging;
pub mod maintenance;
pub mod ports;
pub mod retrieval;
pub mod schema;
pub mod store;
pub mod task;
pub mod time;
pub mod toon;
pub mod write;

pub use config::{Config, ConfigValue};
pub use error::{Error, ErrorKind, Result, lock_or_recover};
pub use git::Project;
pub use graph::Graph;
pub use knudge::{Knudge, KnudgeBuilder};
pub use store::{Event, EventLog, Note, Store};
pub use time::Timestamp;
pub use write::{Draft, Patch, WriteAction, WriteOutcome};
