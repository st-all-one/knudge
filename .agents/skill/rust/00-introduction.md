# Rust 1.97.0 — Guia Completo e Otimizado

## Filosofia Rust

- **Performance**: zero-cost abstractions, sem GC, controle fino de
  alocação e layout de memória
- **Confiabilidade**: o sistema de tipos elimina classes inteiras de bugs
  (data races, dangling pointers, null pointer exceptions)
- **Produtividade**: documentação excelente, compiler messages amigáveis,
  ferramental de classe mundial (rustfmt, clippy, rust-analyzer)

## Quem usa Rust

- Infraestrutura crítica (Firefox, Dropbox, Cloudflare, AWS, Meta)
- Sistemas embarcados, WebAssembly, CLI tools, kernels, bancos de dados
- Onde C/C++ era a única opção, Rust agora compete e vence

## Edition 2024

Rust 1.97.0 usa a **Rust Edition 2024** como default para novos projetos.

| Edition | Ano | Mudanças principais |
|---------|-----|---------------------|
| 2015 | 2015 | Estilo inicial, macros `try!`, `#[macro_use]` |
| 2018 | 2018 | NLL borrow checker, `impl Trait`, `dyn Trait`, módulos |
| 2021 | 2021 | `Cargo.toml` resolver v2, `IntoIterator` for arrays, closures |
| 2024 | 2024 | `unsafe` precisions, `impl Trait` everywhere, `gen` blocks |

```toml
# Cargo.toml
[package]
name = "meu-projeto"
version = "0.1.0"
edition = "2024"  # default a partir de rustc 1.85+
```

## Checklist para Projetos Reais

- [ ] `edition = "2024"` no `Cargo.toml`
- [ ] `rust-version = "1.85"` no `Cargo.toml`
- [ ] Clippy ativado (`cargo clippy`)
- [ ] `rustfmt` configurado (`rustfmt.toml`)
- [ ] CI com `cargo test`, `cargo clippy`, `cargo fmt --check`
- [ ] `rust-analyzer` para IDE support
- [ ] `cargo deny` / `cargo audit` para segurança
