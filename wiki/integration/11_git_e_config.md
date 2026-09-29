# 11 — Git, persistência e configuração

O knudge se ancora no **worktree principal** e versiona a memória via `notas/`+`eventos/`,
excluindo o derivado (D29–D34). A config é em dois níveis com segredos só no global (D61/D91).
Especificação: [`../specs/git-e-worktree.md`](../specs/git-e-worktree.md) e
[`../specs/configuracao.md`](../specs/configuracao.md).

## `Project` e layout (D29/D214)

```rust
use knudge_core::git::Project;

// resolve pelo Git (worktree principal; submódulo no próprio top-level)
let project = Project::resolve(&git, &env)?;

// layout alternativo (default `.knudge`; aceita `.a/b`)
let project = Project::resolve_with(&git, &env, ".a/b")?;

// raiz explícita, sem Git
let project = Project::at("/srv/app", "var/mem")?;

project.root();           // &Path (worktree principal)
project.layout();         // &Path (relativo)
project.layout_str();     // String com `/`
project.knowledge_dir();  // <root>/<layout>
project.config_path();    // <root>/<layout>/config.toml
project.ignored_dirs();   // topo do layout + .git/target/node_modules/.hg
```

## `onboard` (fundar/atualizar)

```rust
use knudge_core::git::{onboard, onboard_with_layout, OnboardOptions, Persistence};

let report = onboard(kd.fs_dyn(), kd.git(), kd.env(), OnboardOptions {
    force: false,
    persistence: Some(Persistence::Versioned), // Some força o modo (D213/D34)
})?;

// layout alternativo:
let report = onboard_with_layout(fs, git, env, options, ".a/b")?;
```

Campos úteis de `OnboardReport`: `root`, `knowledge_dir`, `name`, `in_repo`, `config_written`,
`exclude_changed`, `attributes_changed`, `agents_changed`, `skill_changed`.

O `onboard` é **idempotente**: clona a config global, aplica exclusões, cria `notas/`/`eventos/`,
escreve o bloco do `AGENTS.md` e a skill (D60/D162).

## Exclusão e `.gitattributes` (D30/D31/D34)

```rust
use knudge_core::git::{exclude, attributes, exclude_path};

let mudou = exclude::apply_with_layout(
    kd.fs_dyn(),
    project.common_dir(),
    Persistence::Versioned, // LocalOnly exclui o diretório inteiro
    &project.layout_str(),
)?;
let attrs = attributes::apply_with_layout(
    kd.fs_dyn(),
    project.root(),
    Persistence::Versioned,
    &project.layout_str(),
)?;
```

- Nunca usamos `.gitignore` (D30); a exclusão vive em `.git/info/exclude`.
- `Versioned` exclui só `.idx/`, `cache/`, `.locks/`; `LocalOnly` exclui o diretório inteiro.
- Alternar modos **reverte** as linhas sem duplicar.
- `attributes` cobre `notas/**`, config, `eventos/events*.jsonl` (`merge=union`) e o derivado.

## `sync` (commit da memória)

```rust
use knudge_core::git::sync;

let report = sync(
    kd.fs_dyn(),
    kd.git(),
    project,
    Persistence::Versioned,
    Some("mensagem opcional"),   // None = gera do último evento
)?;
println!("commitado={} arquivos={}", report.committed, report.files.len());
```

`sync` usa `git -C <raiz>` (worktree certo) e commita `<layout>/notas` + `<layout>/eventos`.
Fora de repositório ou com `LocalOnly`, é no-op.

## Configuração em dois níveis (D61)

```rust
use knudge_core::config::{Config, global_config_path, KEYS};

// efetiva = defaults + global + projeto (projeto vence)
let global = Config::load(kd.fs_dyn(), &global_config_path(kd.env())?)?;
let local  = Config::load(kd.fs_dyn(), &project.config_path())?;
let config = Config::effective(global.as_ref(), local.as_ref());

let strict = config.get_bool("behavior.strict").unwrap_or(false);
let limit  = config.get_int("recall.default_limit").unwrap_or(5);
let texto  = config.get_str("embeddings.model");

// mutação (grava TOML canônico; preserva ordem na leitura)
let mut c = Config::load(kd.fs_dyn(), &project.config_path())?.unwrap_or_default();
c.set_str("recall.default_limit", "8")?;
c.unset("proposals.gate")?;
c.save(kd.fs_dyn(), &project.config_path())?;
```

- `KEYS`/`schema::spec` enumeram as chaves canônicas; valor inválido é rejeitado com sugestão
  (D212).
- `Config::validate()` valida a tabela; `defaults()` gera a base.

### Segredos (D91)

```rust
use knudge_core::config::schema::SECRETS_PREFIX;
use knudge_core::logging::Redactor;

let mut projeto = local.clone().unwrap_or_default();
projeto.strip_secrets();                    // remove `secrets.*` do projeto

let redator = Redactor::new([token.clone()]);
let log = redator.redact(&format!("token={token}")); // "[REDACTED:<tipo>]"
```

Segredos vivem **só no global**; o projeto é sanitizado em `Config::effective`.

## Recomendações

- **Um layout só.** Se você embute e também usa a CLI, configure o mesmo `knowledge_dir`; senão o
  exclude/sync/AGENTS apontarão para diretórios diferentes.
- **`LocalOnly` para dados sensíveis.** Se a memória não deve ir ao Git, use
  `persistence = LocalOnly`.
- **`sync` no fim do ciclo,** não a cada escrita.
- **Nunca logue segredos.** Redija na borda com `Redactor` (doc 12).
- **Preserve a ordem.** `Config::load` mantém a ordem do arquivo para diffs mínimos; só a escrita
  é canônica.
- **`force` recopia o global** — use só quando quiser sobrescrever o projeto.
