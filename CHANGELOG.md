# Changelog

Todas as mudanças relevantes do knudge. Formato baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/).

## [Não publicado]

### Adicionado
- **AGENTS.md** — guia de contribuição do repositório: padrões de desenvolvimento, erros,
  logs, testes, contrato de bytes e checklist de conclusão.
- **E03 — Store, notas e eventos** (Fase 0, concluído):
  - `jsonl`: codec JSON próprio (`encode`/`decode` canônicos, chaves ordenadas e sem
    dependência externa) e leitor de linhas tolerante a CRLF/linhas em branco.
  - `store`: `Note` (frontmatter + corpo) com render/parse byte-exato, `Store` (write atômico,
    list, read, `update` com `revision`, `remove`) e ordem de commit **nota → evento** (D21).
  - `store/events`: `Event` com `id` derivado do conteúdo (`evt_<base36(8)>`), `EventLog`
    append-only com **dedup on-read** (D26/D28), tolerância a linha malformada, rotação por
    tamanho (`events-NNNN.jsonl`), checkpoint derivado (`.idx/events.checkpoint`) e `history`.
  - `store/lock`: lock advisory por arquivo-alvo (`create_exclusive`), stale 30 s, reclaim por
    rename sidecar e liberação RAII (D23–D25/R05).
  - `store/rebuild`: `Staging` double-buffer (`.idx.new/` + rename atômico — D27).
  - `store/purge`: `purge_derived` remove o id de `*.jsonl` e `index.json` em toda remoção
    (D84), usado por `Store::remove`.
  - `store/sweep`: varredura de resíduos `*.tmp`/`*.lock`/`*.stale` por idade, com `warn`
    (R10), **sem** remover lock fresco de processo vivo.
  - Porta `Fs` estendida com `append`, `create_exclusive`, `rename`, `sync`, `modified_ms`,
    `is_dir` e `remove_dir_all`; `MemFs` virou um FS fiel (tmp+rename, diretórios implícitos) e
    `FaultyFs` injeta falhas para testes de crash.
  - `ARCHITECTURE.md` §5 documenta a persistência; decisão **D96** registrada.
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
- **D95**: hash curto `SHA-256 → u32`; chave/derivação de `id` e gramática TOON v1 (`TOON.md`).
- **D96**: registro de evento (`id` derivado + dedup on-read), rotação por tamanho e
  semântica de `revision`.

### Testes
- 82 testes de unidade no `knudge-core` (erro, tempo/proptest, redação, fakes, symlink,
  schema/hash/ID, TOON, JSONL/JSON, store: commit/lock/events/rebuild/purge/sweep).
- 8 testes de integração do binário (`--help`, `kd == kd prime`, `--json`, exit codes,
  EPIPE, comando desconhecido).
