//! Codec JSON mínimo e canônico (sem dependência externa).
//!
//! Cobre o subconjunto usado nos arquivos `*.jsonl` do knudge: objetos com chaves string,
//! arrays, strings com escapes, inteiros, floats finitos e booleanos. `null` é **rejeitado**
//! (o projeto não usa `null` — D05). A emissão ordena as chaves para ser determinística.

mod decode;
mod encode;

pub use decode::decode;
pub use encode::encode;
