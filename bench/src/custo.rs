//! Custo por resposta fora do runtime (D229).
//!
//! Mede, no `kd` real sobre um projeto temporário, os bytes da resposta padrão do `ask`
//! (baseline D161: 1 corpo cheio + 4 truncados + linhas pipe) e o contra-factual de um leitor
//! que abriria com `cat` cada nota devolvida. Corpus e consultas replicam `quality` (192 notas,
//! 60 consultas, 5 famílias): mesmas consulta/flags/corpus → mesmas linhas. Nenhuma chave de
//! config, verbo ou byte de envelope muda.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use knudge_core::schema::{NoteType, id};

use crate::fixture::Lcg;

/// Semente do corpus (idêntica à de `quality`).
const SEED: u64 = 0x5151_2a2a_0f0f_1234;

/// Notas de tópico por tópico (o restante são distratores).
const PER_TOPIC: usize = 8;

/// Tópico: nome acentuado, forma sem acento e vocabulário.
type Topic = (&'static str, &'static str, &'static [&'static str]);

/// Tópicos PT-BR: (nome acentuado, forma sem acento, vocabulário). Idêntico a `quality`.
const TOPICS: [Topic; 12] = [
    (
        "configuração",
        "configuracao",
        &["ajuste", "parâmetro", "chave", "global"],
    ),
    (
        "índice",
        "indice",
        &["postings", "reconstrução", "derivado", "termo"],
    ),
    (
        "âncora",
        "ancora",
        &["caminho", "arquivo", "working", "set"],
    ),
    (
        "retenção",
        "retencao",
        &["esquecimento", "estabilidade", "revisão", "curva"],
    ),
    (
        "confiança",
        "confianca",
        &["evidência", "confirmação", "posterior", "beta"],
    ),
    (
        "sessão",
        "sessao",
        &["contexto", "handoff", "prime", "rewind"],
    ),
    (
        "análise",
        "analise",
        &["diagnóstico", "integridade", "violação", "relatório"],
    ),
    (
        "árvore",
        "arvore",
        &["grafo", "ciclo", "dependência", "aresta"],
    ),
    (
        "memória",
        "memoria",
        &["nota", "corpo", "frontmatter", "toon"],
    ),
    (
        "depreciação",
        "depreciacao",
        &["aposentadoria", "prune", "shelf", "decay"],
    ),
    (
        "contradição",
        "contradicao",
        &["conflito", "retratação", "defeasible", "tms"],
    ),
    (
        "consulta",
        "consulta",
        &["busca", "recall", "ranking", "fusão"],
    ),
];

/// Vocabulário dos distratores (sem os nomes de tópico). Idêntico a `quality`.
const GLOBAL: [&str; 20] = [
    "projeto",
    "agente",
    "sistema",
    "comando",
    "evento",
    "estado",
    "decisão",
    "tarefa",
    "épico",
    "sequência",
    "escrita",
    "leitura",
    "vetor",
    "fila",
    "histórico",
    "métrica",
    "módulo",
    "função",
    "teste",
    "config",
];

/// Sinônimos por tópico (canal semântico sintético de `quality`).
const SYNONYMS: [&str; 12] = [
    "setup",
    "indexacao",
    "referencia",
    "memorizacao",
    "credibilidade",
    "janela",
    "inspecao",
    "hierarquia",
    "recordacao",
    "obsolescencia",
    "oposicao",
    "pergunta",
];

/// Plurais por tópico (família morfológica de `quality`).
const MORPH: [&str; 12] = [
    "configurações",
    "índices",
    "âncoras",
    "retenções",
    "confianças",
    "sessões",
    "análises",
    "árvores",
    "memórias",
    "depreciações",
    "contradições",
    "consultas",
];

/// Consulta de uma família, com âncora opcional (`working-set`).
struct Query {
    text: String,
    family: &'static str,
    anchor: Option<String>,
}

/// Corpus em JSONL, bytes (statement + corpo) por id e as 60 consultas rotuladas.
struct Dataset {
    jsonl: String,
    contents: BTreeMap<String, usize>,
    queries: Vec<Query>,
}

/// Gera o corpus e as consultas de forma determinística (espelho de `quality::dataset_with`).
fn dataset() -> Dataset {
    let mut rng = Lcg::new(SEED);
    let mut probe = Lcg::new(SEED ^ 0x00dd_1790);
    let mut jsonl = String::new();
    let mut contents = BTreeMap::new();
    let mut queries: Vec<Query> = Vec::new();
    for (topic_index, entry) in TOPICS.iter().enumerate() {
        let (topic, folded, words) = *entry;
        for slot in 0..PER_TOPIC {
            let first = pick_word(&mut rng, words);
            let second = pick_word(&mut rng, words);
            let statement = format!("{topic} {first} {second} variante{slot}");
            let body = topic_body(&mut rng, words);
            push_note(
                &mut jsonl,
                &mut contents,
                statement,
                body,
                &[format!("src/{folded}/modulo_{slot}.rs")],
            );
        }
        let plural = MORPH.get(topic_index).copied().unwrap_or(topic);
        let synonym = SYNONYMS.get(topic_index).copied().unwrap_or(topic);
        for (text, family) in [
            (topic, "com-acento"),
            (folded, "sem-acento"),
            (plural, "morfologia"),
        ] {
            queries.push(Query {
                text: text.to_string(),
                family,
                anchor: None,
            });
        }
        queries.push(Query {
            text: pick_word(&mut probe, &GLOBAL).to_string(),
            family: "working-set",
            anchor: Some(format!("src/{folded}/modulo_0.rs")),
        });
        queries.push(Query {
            text: format!("{} {}", synonym, pick_word(&mut probe, &GLOBAL)),
            family: "sinonimo",
            anchor: None,
        });
    }
    for index in 0..(TOPICS.len() * PER_TOPIC) {
        let first = pick_word(&mut rng, &GLOBAL);
        let second = pick_word(&mut rng, &GLOBAL);
        let statement = format!("{first} {second} ruido{index}");
        push_note(
            &mut jsonl,
            &mut contents,
            statement,
            global_body(&mut rng),
            &[],
        );
    }
    Dataset {
        jsonl,
        contents,
        queries,
    }
}

/// Acrescenta uma nota ao JSONL e registra os bytes de statement + corpo pelo id.
fn push_note(
    jsonl: &mut String,
    contents: &mut BTreeMap<String, usize>,
    statement: String,
    body: String,
    anchors: &[String],
) {
    let note_id = id::note_id(NoteType::Fact, &statement);
    let _ = contents.insert(note_id, statement.len().saturating_add(body.len()));
    let listed = anchors
        .iter()
        .map(|anchor| format!("\"{}\"", json_escape(anchor)))
        .collect::<Vec<_>>()
        .join(",");
    jsonl.push_str(&format!(
        "{{\"type\":\"fact\",\"statement\":\"{}\",\"body\":\"{}\",\"anchors\":[{}]}}\n",
        json_escape(&statement),
        json_escape(&body),
        listed,
    ));
}

/// Escolhe uma palavra sem indexação direta.
fn pick_word<'a>(rng: &mut Lcg, words: &[&'a str]) -> &'a str {
    words
        .get(rng.pick(words.len()))
        .copied()
        .unwrap_or_default()
}

/// Corpo de uma nota de tópico (vocabulário global + duas palavras do tópico).
fn topic_body(rng: &mut Lcg, words: &[&str]) -> String {
    let mut parts: Vec<&str> = (0..12).map(|_| pick_word(rng, &GLOBAL)).collect();
    parts.push(pick_word(rng, words));
    parts.push(pick_word(rng, words));
    parts.join(" ")
}

/// Corpo de um distrator (só vocabulário global).
fn global_body(rng: &mut Lcg) -> String {
    (0..14)
        .map(|_| pick_word(rng, &GLOBAL))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Escapa uma string para JSON (o corpus só tem `[a-z0-9 _-]` e acentos).
fn json_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            other => out.push(other),
        }
    }
    out
}

/// Projeto temporário removido ao sair (a menos que `KNUDGE_BENCH_KEEP=1`).
struct TempProject {
    root: PathBuf,
}

impl Drop for TempProject {
    fn drop(&mut self) {
        if std::env::var("KNUDGE_BENCH_KEEP").as_deref() != Ok("1") {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

/// Runner de comandos contra o projeto (mesmo isolamento do `e2e`).
struct Runner<'a> {
    kd: &'a Path,
    root: &'a Path,
}

impl Runner<'_> {
    fn ok(&self, list: &[String]) -> Option<Vec<u8>> {
        let output = Command::new(self.kd)
            .args(list)
            .current_dir(self.root)
            .env("HOME", self.root)
            .env("XDG_CONFIG_HOME", self.root.join(".config"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("NO_COLOR", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .ok()?;
        if output.status.success() {
            Some(output.stdout)
        } else {
            eprintln!(
                "kd {} falhou: {}",
                list.join(" "),
                String::from_utf8_lossy(&output.stderr)
            );
            None
        }
    }
}

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|item| (*item).to_string()).collect()
}

/// Cria o projeto e grava o corpus em lotes de 100 (`write.batch_max`).
fn build_project(kd: &Path, dataset: &Dataset) -> Option<TempProject> {
    let root = std::env::temp_dir().join(format!("knudge-bench-custo-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).ok()?;
    for cmd in [
        args(&["init", "-q"]),
        args(&["config", "user.email", "bench@example.invalid"]),
        args(&["config", "user.name", "bench"]),
    ] {
        let _ = Command::new("git").args(cmd).current_dir(&root).status();
    }
    let runner = Runner { kd, root: &root };
    runner.ok(&args(&["init", "--no-prompt"]))?;
    let lines: Vec<&str> = dataset.jsonl.lines().collect();
    for (chunk, batch) in lines.chunks(100).enumerate() {
        let path = root.join(format!("custo_{chunk}.jsonl"));
        fs::write(&path, format!("{}\n", batch.join("\n"))).ok()?;
        runner.ok(&args(&["write", "--batch", path.to_str()?]))?;
    }
    Some(TempProject { root })
}

/// Extrai os ids das linhas `id|statement` do `--brief`.
fn brief_ids(stdout: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(stdout)
        .lines()
        .filter_map(|line| line.split('|').next())
        .filter(|id| id.starts_with("fact_"))
        .map(str::to_string)
        .collect()
}

/// Argumentos do `ask` para uma consulta (default ou `--brief`).
fn ask_args(query: &Query, brief: bool) -> Vec<String> {
    let mut list = vec!["ask".to_string()];
    if brief {
        list.push("--brief".to_string());
    }
    if let Some(anchor) = &query.anchor {
        list.push("--anchor".to_string());
        list.push(anchor.clone());
    }
    list.push(query.text.clone());
    list
}

/// Soma os bytes de statement + corpo das notas devolvidas.
fn counterfactual(stdout: &[u8], contents: &BTreeMap<String, usize>) -> usize {
    brief_ids(stdout)
        .iter()
        .filter_map(|note_id| contents.get(note_id))
        .fold(0_usize, |total, bytes| total.saturating_add(*bytes))
}

/// Mede cada família e imprime o TSV `familia\tbytes_out\tbytes_counterfactual\trazao`.
pub fn run(kd: &Path) {
    if !kd.is_file() {
        eprintln!(
            "[PENDENTE] binário kd não encontrado em {} — use --kd PATH",
            kd.display()
        );
        return;
    }
    let dataset = dataset();
    let Some(project) = build_project(kd, &dataset) else {
        eprintln!("[PENDENTE] falha ao preparar projeto temporário");
        return;
    };
    let runner = Runner {
        kd,
        root: &project.root,
    };
    let mut rows: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for query in &dataset.queries {
        let Some(out) = runner.ok(&ask_args(query, false)) else {
            continue;
        };
        let Some(short) = runner.ok(&ask_args(query, true)) else {
            continue;
        };
        let bytes = counterfactual(&short, &dataset.contents);
        let entry = rows.entry(query.family).or_insert((0, 0));
        entry.0 = entry.0.saturating_add(out.len());
        entry.1 = entry.1.saturating_add(bytes);
    }
    println!("familia\tbytes_out\tbytes_counterfactual\trazao");
    for (family, (out, counterfactual)) in rows {
        let razao = if out == 0 {
            0.0
        } else {
            counterfactual as f64 / out as f64
        };
        println!("{family}\t{out}\t{counterfactual}\t{razao:.3}");
    }
}
