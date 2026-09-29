# 01 — Instalação e fachada (`Knudge`)

A fachada `Knudge` reproduz a montagem que a CLI/MCP fazem: `StdFs`/`StdGit`/`StdEnv` +
resolução do `Project` + config efetiva. É a porta de entrada para a maioria das integrações.

## Dependência

```toml
[dependencies]
knudge-core = "0.5"           # publicado
# knudge-core = { path = "../knudge/crates/knudge-core" }   # local
```

O crate é `no_std`? **Não** — a fachada e os adapters usam `std`. O domínio em si é independente
de SO (portas), mas o crate compila com `std`.

## Construção

```rust
use knudge_core::{Knudge, KnudgeBuilder, Result};

// 1) cwd + worktree principal + layout default `.knudge`
let kd = Knudge::open()?;

// 2) layout alternativo (aninhado é permitido)
let kd = Knudge::builder()
    .knowledge_dir(".a/b")
    .open()?;

// 3) raiz explícita, sem consultar Git (in_repo() == false)
let kd = Knudge::builder()
    .root("/srv/app")
    .knowledge_dir("var/mem")
    .open()?;
```

| Método | Efeito |
|---|---|
| `Knudge::open()` | cwd/worktree + `.knudge` |
| `KnudgeBuilder::root(p)` | usa `p` como raiz; **não** consulta Git |
| `KnudgeBuilder::knowledge_dir(l)` | layout relativo (`.knudge`, `.a/b`); validado |
| `KnudgeBuilder::open()` | resolve tudo e carrega a config |

`knowledge_dir` rejeita absolutos, `..`, `.` e vazio (erro `Config`, exit 7).

## O que a sessão expõe

```rust
let kd = Knudge::open()?;

kd.project();          // &Project (raiz, layout, in_repo, common_dir)
kd.config();           // &Config (efetiva: defaults + global + projeto)
kd.project_root();     // &Path
kd.knowledge_dir();    // PathBuf (<raiz>/<layout>)
kd.now_ms();           // i64 (instante capturado na abertura)

kd.fs();  kd.fs_dyn(); // StdFs / &dyn Fs
kd.git();              // &StdGit
kd.env();              // &StdEnv

kd.store();            // Store<'_>
kd.events();           // EventLog<'_>
kd.index()?;           // Index
kd.corpus()?;          // Corpus { notes, index, graph }
kd.notes()?;           // Vec<Note>
kd.graph()?;           // Graph
kd.thresholds()?;      // DedupThresholds
kd.write_context()?;   // WriteContext<'_>
kd.sweep_residues(&logger) -> Vec<String>;
```

### Reaproveite — não reabra por operação

`Knudge` mantém `fs`/`git`/`env`/config em memória. Construa **uma vez** por processo (ou por
requisição, se o cwd muda) e reutilize. Reabrir por nota relê a config e re-resolve o Git.

```rust
struct App { kd: Knudge }
```

> `Knudge` não é `Sync`/`Clone` por design (segura o estado da sessão). Para concorrência,
> compartilhe **dados** (índice/grafo imutáveis) e reabra `Store`/`WriteContext` por thread, ou
> proteja com `Mutex` (stateful).

## Config efetiva e segredos

`Knudge` carrega a config em dois níveis (D61): defaults → global
(`$XDG_CONFIG_HOME/local/knudge/config.toml` ou `~/.config/...`) → projeto
(`<raiz>/<layout>/config.toml`). **Segredos só entram pelo global** e são removidos do projeto.

```rust
let strict = kd.config().get_bool("behavior.strict").unwrap_or(false);
let limit  = kd.config().get_int("recall.default_limit").unwrap_or(5);
```

Para usar `Clock`/`Fs`/`Git`/`Embedder` próprios (ex.: determinismo em teste), não use a fachada:
monte `Project`, `Store`, `Index` e chame o domínio diretamente com suas portas. Veja
[12 — Determinismo](12_erros_logs_e_determinismo.md).

## Abrir sem a fachada

```rust
use knudge_core::adapters::{StdEnv, StdFs, StdGit};
use knudge_core::config::{Config, global_config_path};
use knudge_core::git::Project;
use knudge_core::store::Store;

let fs = StdFs::new();
let git = StdGit::new();
let env = StdEnv::new();
let project = Project::resolve_with(&git, &env, ".a/b")?;

let global = Config::load(&fs, &global_config_path(&env)?)?;
let local = Config::load(&fs, &project.config_path())?;
let config = Config::effective(global.as_ref(), local.as_ref());

let store = Store::new(&fs, project.knowledge_dir());
```

## Recomendações

- **Um `Knudge` por processo.** Se o diretório de trabalho muda (ex.: serviço multi-projeto),
  prefira `KnudgeBuilder::root(projeto)` por requisição em vez de `set_current_dir`.
- **Trate `open()` como falível.** Num serviço, degrade com defaults se a config estiver
  corrompida (`Config::load` já devolve `None` para ausente).
- **Nunca edite `notas/` à mão.** A nota é a verdade, mas a escrita passa por `write`/`Store` para
  garantir atomicidade e evento (D20/D21).
- **Log nunca para stdout.** Use a porta `Logger` (doc 12). O core não imprime.
