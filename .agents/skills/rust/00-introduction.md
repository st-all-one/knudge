# Rust 1.100.0 — Guia Completo e Otimizado

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

## Destaques da 1.100.0

| Recurso | O que muda |
|---|---|
| Never type (`!`) | Estável e nomeável em qualquer posição; `Infallible` virou alias de `!`; fallback agora é `!` em **todas** as editions |
| `Allocator` API | Alocadores customizados para `Vec` e `Box` (`new_in`, `with_capacity_in`, `allocator`, `_with_allocator`) |
| `DropGuard` | Roda uma closure em qualquer saída de escopo — `return`, `?` ou panic |
| `conjure_zst` | Cria ZSTs *inhabited* de forma explícita (unsafe), no lugar de `transmute(())` |
| Cargo lints | `[lints.cargo]` no `Cargo.toml` + `cargo::unused_dependencies` |
| `min-publish-age` | Cooldown de versões recém-publicadas (proteção de supply chain) |

Detalhes e APIs completas em `CHANGELOG.md`; uso prático nos arquivos de
cada tópico (ver `SKILL.md` → File Map).

## Edition 2024

Rust 1.100.0 usa a **Rust Edition 2024** como default para novos projetos.

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
- [ ] `rust-version = "1.85"` no `Cargo.toml` (mínimo da edition; ajuste à sua MSRV)
- [ ] Clippy ativado (`cargo clippy`)
- [ ] `[lints.cargo]` com `unused_dependencies` e `[lints.rust]` no `Cargo.toml`
- [ ] `rustfmt` configurado (`rustfmt.toml`)
- [ ] CI com `cargo test`, `cargo clippy`, `cargo fmt --check`
- [ ] `rust-analyzer` para IDE support
- [ ] `cargo deny` / `cargo audit` para segurança
- [ ] `min-publish-age` em `.cargo/config.toml` para cooldown de dependências
