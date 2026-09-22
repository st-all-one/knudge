# Changelog

Todas as mudanças relevantes do knudge. Formato baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/).

## [Não publicado]

### Adicionado
- **AGENTS.md** — guia de contribuição do repositório: padrões de desenvolvimento, erros,
  logs, testes, contrato de bytes e checklist de conclusão.
- **E02 — Contrato de bytes: TOON, schema e IDs** (Fase 0, concluído):
  - `schema/types`: enums fechados `NoteType` (11), `Scope`, `Classification`, `Status`.
  - `schema/hash`: hash curto `SHA-256 → u32`, `hex8` e `base36(8)` (D95).
  - `schema/body`: `normalize` (NFC + trim + colapso) e `body_hash` (D06).
  - `schema/id`: `id` endereçado por conteúdo (`type + U+001F + statement`) e validador (D01–D03).
  - `schema/text`: contagem de `statement` em escalares Unicode, limite 120 (D08).
  - `schema/frontmatter`: ordem canônica das 19 chaves, omissão de opcionais (D05), leitura
    tolerante com warning (D16) e validação (T01/T07).
  - `toon`: parser/emissor próprios (`lex`/`flow`/`parse`/`emit`), round-trip byte-exato,
    comentários, escapes, zero bytes, `1.0 → 1` e detecção de `schema_version` (D74/D75).
  - `TOON.md` publica a gramática e as regras de bytes.
- **E01 — Fundação** (Fase 0, concluído):
  - Workspace Cargo (`knudge-core`, `knudge-cli`, `knudge-mcp`) com `edition = "2024"`,
    MSRV `1.97`, `[workspace.lints]` (R44), perfil de release (R41) e `make check` (E01-T01/T03).
  - Portas determinísticas (`Clock`, `Rng`, `Env`, `Fs`, `Git`, `HookRunner`, `Logger`),
    adaptadores `std` e fakes (`FixedClock`, `SeqRng`, `FakeEnv`, `FakeGit`, `MemFs`,
    `RecordingLogger`) (E01-T02).
  - Modelo de erro `Error`/`ErrorKind` com mapa código→exit, `thiserror`, poison via
    `into_inner` e contexto de I/O com path (E01-T06; R30–R35).
  - `Timestamp` UTC com milissegundos, formatação/parsing próprios e round-trip por proptest
    (E01-T02; D07).
  - Redação de segredos no layer de log (allowlist + chaves sensíveis) (E01-T07; R22).
  - Binário `kd` com a **superfície v2** (E01-T04; `16_cli_surface.md`): `kd` = `prime`,
    `--json` (`{success, command, data?, error?, warnings?}`), exit codes estáveis e
    `EPIPE → exit 0`.
  - Política de memória: `#![forbid(unsafe_code)]`, rejeição de symlink em `.knudge/` (E01-T08).
  - `ARCHITECTURE.md`, `MODULE.md` por crate e módulos temáticos de `knudge-core` (E01-T05).

### Decidido (superfície CLI v2)
- **D57** reescrita: `prime` é protocolo estático byte-idêntico; estado/handoff → `rewind`.
- **D88** ajustada: `rewind` emite `context_id`; `kd rewind --resume <id>`.
- **D93**: `task` com `scope` fechado (`plan|epic|issue|task`), hierarquia máx. 4.
- **D94**: `strict` é config de projeto (`[behavior] strict`), sem flag.

### Testes
- 56 testes de unidade no `knudge-core` (erro, tempo/proptest, redação, fakes, symlink,
  schema/hash/ID, TOON com proptest de round-trip).
- 8 testes de integração do binário (`--help`, `kd == kd prime`, `--json`, exit codes,
  EPIPE, comando desconhecido).
