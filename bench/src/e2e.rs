//! Benchmark ponta-a-ponta: invoca o binário `kd` real sobre um projeto temporário.
//!
//! Mede a latência de parede de cada ação (processo + resolução de projeto + I/O +
//! domínio), em corpora de tamanhos diferentes. Inclui um piso (`self version`) para
//! separar o custo de *startup* do custo do comando.

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::fixture;
use crate::harness::Harness;

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Projeto temporário removido ao sair (a menos que `KNUDGE_BENCH_KEEP=1`).
pub struct TempProject {
    /// Raiz do projeto (onde vive `.knudge/`).
    pub root: PathBuf,
}

impl Drop for TempProject {
    fn drop(&mut self) {
        if std::env::var("KNUDGE_BENCH_KEEP").as_deref() != Ok("1") {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

/// Identificadores úteis extraídos do corpus.
struct Ids {
    note: String,
    task: String,
}

/// Runner de comandos contra um projeto.
struct Runner<'a> {
    kd: &'a Path,
    root: &'a Path,
    no_idle: bool,
}

impl Runner<'_> {
    fn raw(&self, args: &[String]) -> Output {
        let mut command = Command::new(self.kd);
        command
            .args(args)
            .current_dir(self.root)
            .env("HOME", self.root)
            .env("XDG_CONFIG_HOME", self.root.join(".config"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("NO_COLOR", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if self.no_idle {
            command.env("KNUDGE_NO_IDLE", "1");
        }
        command.output().expect("kd deve ser executável")
    }

    fn timed(&self, args: &[String]) -> (Duration, Output) {
        let start = Instant::now();
        let output = self.raw(args);
        (start.elapsed(), output)
    }

    fn ok(&self, args: &[String]) -> String {
        let output = self.raw(args);
        assert!(
            output.status.success(),
            "comando falhou: kd {} \nstderr: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).to_string()
    }
}

/// Executa a bancada ponta-a-ponta.
pub fn run(harness: &mut Harness, kd: &Path, sizes: &[usize], samples: usize, no_idle: bool) {
    assert!(kd.exists(), "binário kd não encontrado em {}", kd.display());
    for &n in sizes {
        let project = build_project(kd, n);
        let ids = extract_ids(&project.root);
        let runner = Runner {
            kd,
            root: &project.root,
            no_idle,
        };
        let suffix = if no_idle { " [no-idle]" } else { "" };
        let group = format!("e2e/N={n}{suffix}");
        measure_floor(harness, &group, &runner, samples);
        measure_reads(harness, &group, &runner, &ids, samples);
        measure_mutations(harness, &group, &runner, &ids, samples);
        // `sync` é medido por último (deixa o worktree limpo no fim).
        let _ = runner.timed(&args(&["sync", "--message", "bancada"]));
    }
}

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|item| (*item).to_string()).collect()
}

/// Cria o projeto, inicializa o git e popula o corpus.
fn build_project(kd: &Path, knowledge: usize) -> TempProject {
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "knudge-bench-{}-{}-{}",
        std::process::id(),
        counter,
        knowledge
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("cria projeto");
    let _ = Command::new("git")
        .args(["init", "-q"])
        .current_dir(&root)
        .status();
    let _ = Command::new("git")
        .args(["config", "user.email", "bench@example.invalid"])
        .current_dir(&root)
        .status();
    let _ = Command::new("git")
        .args(["config", "user.name", "bench"])
        .current_dir(&root)
        .status();

    let runner = Runner {
        kd,
        root: &root,
        no_idle: false,
    };
    runner.ok(&args(&["init", "--no-prompt"]));

    let gen_dir = root.join("gen");
    fs::create_dir_all(&gen_dir).expect("cria gen");
    // Lotes de 100 (teto default de `write.batch_max`).
    let mut start = 0;
    while start < knowledge {
        let count = (knowledge - start).min(100);
        let path = gen_dir.join(format!("notes_{start}.jsonl"));
        write_file(&path, &fixture::knowledge_jsonl(start, count));
        runner.ok(&args(&[
            "write",
            "--batch",
            path.to_str().expect("path utf8"),
        ]));
        start += count;
    }
    let epics = (knowledge / 20).max(1);
    let tasks = (knowledge / 8).max(1);
    let task_lines = fixture::task_jsonl(epics, tasks);
    // `task.batch_max` default = 100: divide o lote em arquivos de 100 linhas.
    for (chunk, lines) in task_lines.lines().collect::<Vec<_>>().chunks(100).enumerate() {
        let path = gen_dir.join(format!("tasks_{chunk}.jsonl"));
        let mut text = String::new();
        for line in lines {
            text.push_str(line);
            text.push('\n');
        }
        write_file(&path, &text);
        runner.ok(&args(&[
            "task",
            "new",
            "--batch",
            path.to_str().expect("path utf8"),
        ]));
    }

    let actual = count_notes(&root);
    eprintln!("  corpus pedido N={knowledge}: {actual} notas gravadas");

    TempProject { root }
}

fn count_notes(root: &Path) -> usize {
    let notas = root.join(".knudge").join("notas");
    walk(&notas)
        .iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
        .count()
}

fn write_file(path: &Path, contents: &str) {
    let mut file = fs::File::create(path).expect("cria arquivo de lote");
    file.write_all(contents.as_bytes()).expect("escreve lote");
}

/// Extrai um id de nota e um de tarefa varrendo `notas/`.
fn extract_ids(root: &Path) -> Ids {
    let mut note = String::new();
    let mut task = String::new();
    let notas = root.join(".knudge").join("notas");
    for entry in walk(&notas) {
        let is_md = entry.extension().and_then(|ext| ext.to_str()) == Some("md");
        if !is_md {
            continue;
        }
        let Some(stem) = entry.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if task.is_empty() && stem.starts_with("task_") {
            task = stem.to_string();
        }
        if note.is_empty() && stem.starts_with("fact_") {
            note = stem.to_string();
        }
    }
    assert!(!note.is_empty(), "nenhuma nota fact encontrada");
    assert!(!task.is_empty(), "nenhuma nota task encontrada");
    Ids { note, task }
}

fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else {
            out.push(path);
        }
    }
    out
}

fn measure_floor(harness: &mut Harness, group: &str, runner: &Runner<'_>, samples: usize) {
    // Piso: custo de processo + parsing do clap + resolução de projeto.
    harness.measure_cmd(group, "self version (piso de startup)", samples, || {
        runner.timed(&args(&["self", "version"])).0
    });
    harness.measure_cmd(group, "self version --json", samples, || {
        runner.timed(&args(&["self", "version", "--json"])).0
    });
    harness.measure_cmd(group, "--help", samples, || {
        runner.timed(&args(&["--help"])).0
    });
}

fn measure_reads(
    harness: &mut Harness,
    group: &str,
    runner: &Runner<'_>,
    ids: &Ids,
    samples: usize,
) {
    harness.measure_cmd(group, "prime", samples, || {
        runner.timed(&args(&["prime"])).0
    });
    harness.measure_cmd(group, "prime --long", samples, || {
        runner.timed(&args(&["prime", "--long"])).0
    });
    harness.measure_cmd(group, "ask (query comum)", samples, || {
        runner.timed(&args(&["ask", "retrieval indice grafo"])).0
    });
    harness.measure_cmd(group, "ask (query rara)", samples, || {
        runner.timed(&args(&["ask", "shelf life decay ancora"])).0
    });
    harness.measure_cmd(group, "ask --json", samples, || {
        runner.timed(&args(&["ask", "cache embedding vetor", "--json"])).0
    });
    harness.measure_cmd(group, "ask --limit 50", samples, || {
        runner.timed(&args(&["ask", "retrieval indice grafo", "--limit", "50"])).0
    });
    harness.measure_cmd(group, "ask --brief", samples, || {
        runner.timed(&args(&["ask", "retrieval indice grafo", "--brief"])).0
    });
    harness.measure_cmd(group, "ask --type fact --anchor src/**", samples, || {
        runner.timed(&args(&[
            "ask",
            "retrieval indice grafo",
            "--type",
            "fact",
            "--anchor",
            "src/**",
        ]))
        .0
    });
    harness.measure_cmd(group, "ask --around <nota>", samples, || {
        runner
            .timed(&args(&["ask", "--around", &ids.note, "--depth", "2"]))
            .0
    });
    harness.measure_cmd(group, "ask --id <nota>", samples, || {
        runner.timed(&args(&["ask", "--id", &ids.note])).0
    });
    harness.measure_cmd(group, "rewind", samples, || {
        runner.timed(&args(&["rewind"])).0
    });
    harness.measure_cmd(group, "rewind --json", samples, || {
        runner.timed(&args(&["rewind", "--json"])).0
    });
    harness.measure_cmd(group, "rewind --files src/core/**", samples, || {
        runner
            .timed(&args(&["rewind", "--files", "src/core/modulo_0/arquivo_0.rs"]))
            .0
    });
    harness.measure_cmd(group, "task list", samples, || {
        runner.timed(&args(&["task", "list"])).0
    });
    harness.measure_cmd(group, "task list --ready", samples, || {
        runner.timed(&args(&["task", "list", "--ready"])).0
    });
    harness.measure_cmd(group, "task list --sort impact", samples, || {
        runner
            .timed(&args(&["task", "list", "--sort", "impact"]))
            .0
    });
    harness.measure_cmd(group, "task list --full-content", samples, || {
        runner
            .timed(&args(&["task", "list", "--full-content"]))
            .0
    });
    harness.measure_cmd(group, "task show --id <tarefa>", samples, || {
        runner.timed(&args(&["task", "show", "--id", &ids.task])).0
    });
    harness.measure_cmd(group, "task graph", samples, || {
        runner.timed(&args(&["task", "graph"])).0
    });
    harness.measure_cmd(group, "knowledge map --universe", samples, || {
        runner
            .timed(&args(&["knowledge", "map", "--universe"]))
            .0
    });
    harness.measure_cmd(group, "knowledge rank --universe", samples, || {
        runner
            .timed(&args(&["knowledge", "rank", "--universe"]))
            .0
    });
    harness.measure_cmd(group, "knowledge tags", samples, || {
        runner.timed(&args(&["knowledge", "tags"])).0
    });
    harness.measure_cmd(group, "drain --status", samples, || {
        runner.timed(&args(&["drain", "--status"])).0
    });
    harness.measure_cmd(group, "doctor", samples, || {
        runner.timed(&args(&["doctor"])).0
    });
    harness.measure_cmd(group, "doctor --explain", samples, || {
        runner.timed(&args(&["doctor", "--explain"])).0
    });
    harness.measure_cmd(group, "maintenance learn --universe", samples, || {
        runner
            .timed(&args(&["maintenance", "learn", "--universe"]))
            .0
    });
    harness.measure_cmd(group, "maintenance compact --universe", samples, || {
        runner
            .timed(&args(&["maintenance", "compact", "--universe"]))
            .0
    });
    harness.measure_cmd(group, "maintenance prune --universe", samples, || {
        runner
            .timed(&args(&["maintenance", "prune", "--universe"]))
            .0
    });
    harness.measure_cmd(group, "config list", samples, || {
        runner.timed(&args(&["config", "list"])).0
    });
    harness.measure_cmd(group, "config get recall.default_limit", samples, || {
        runner
            .timed(&args(&["config", "get", "recall.default_limit"]))
            .0
    });
}

fn measure_mutations(
    harness: &mut Harness,
    group: &str,
    runner: &Runner<'_>,
    ids: &Ids,
    samples: usize,
) {
    // Escrita nova: afirmações únicas a cada chamada (o corpus cresce).
    let mut serial = 0_u64;
    let mut write_once = || {
        serial += 1;
        let statement = format!("medicao de escrita numero {serial} com termo unico{serial}");
        let duration = runner
            .timed(&args(&["write", "--summary", &statement]))
            .0;
        duration
    };
    harness.measure_cmd(group, "write (nova)", samples, || write_once());
    // Escrita idempotente: mesma afirmação → `unchanged`.
    harness.measure_cmd(group, "write (idempotente)", samples, || {
        runner
            .timed(&args(&["write", "--summary", "afirmacao idempotente da bancada"]))
            .0
    });
    harness.measure_cmd(group, "config set (projeto)", samples, || {
        runner
            .timed(&args(&["config", "set", "recall.default_limit", "5"]))
            .0
    });
    harness.measure_cmd(group, "forget (soft)", samples, || {
        runner.timed(&args(&["forget", "--id", &ids.note])).0
    });
    harness.measure_cmd(group, "forget --restore", samples, || {
        runner
            .timed(&args(&["forget", "--id", &ids.note, "--restore"]))
            .0
    });
    harness.measure_cmd(group, "sync", samples, || {
        runner.timed(&args(&["sync", "--message", "bancada"])).0
    });
}
