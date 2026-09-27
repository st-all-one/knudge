//! Avaliação de **qualidade** de retrieval (E16/T01): Recall@k, MRR e nDCG@k.
//!
//! Corpus PT-BR **sintético e rotulado** (tópicos × consultas), gerado de forma determinística.
//! Mede o **resultado** do `recall` (tokenização + BM25 + âncoras + fusão RRF) com os defaults de
//! produção, não a latência (isso é `micro`/`e2e`). Baseline em `bench/qualidade.md` e
//! `bench/qualidade.json`. `criterion` é não-objetivo (E13-T09): a bancada **observa**, não é gate.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use knudge_core::corpus::Corpus;
use knudge_core::retrieval::{RecallQuery, Universe, recall};
use knudge_core::schema::{NoteType, id};
use knudge_core::store::Note;
use knudge_core::write::Draft;

use crate::fixture::Lcg;

/// `k` das métricas `@k`.
const KS: [usize; 4] = [1, 3, 5, 10];

/// Notas por tópico (as demais notas são distratores).
const PER_TOPIC: usize = 8;

/// Semente fixa do gerador (determinismo do baseline).
const SEED: u64 = 0x5151_2a2a_0f0f_1234;

/// Tópicos PT-BR: (nome acentuado, forma dobrada, vocabulário do tópico).
///
/// A consulta **sem acento** é o caso que D172 (fold) precisa resolver: hoje `configuracao` não
/// casa `configuração` (o tokenizador ASCII D36 parte a palavra em `configura` + `o`); depois do
/// fold, casa. O dataset serve de régua para T03/T04/T09.
const TOPICS: [(&str, &str, &[&str]); 12] = [
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

/// Vocabulário global dos distratores (sem os nomes de tópico, para o sinal do tópico ser limpo).
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

/// Consulta rotulada: texto + ids relevantes (relevância binária) + família.
struct LabeledQuery {
    query: String,
    relevant: BTreeSet<String>,
    family: &'static str,
}

/// Resultado de uma consulta.
#[derive(Default)]
struct Eval {
    recall: BTreeMap<usize, f64>,
    mrr: f64,
    ndcg: BTreeMap<usize, f64>,
}

/// Soma agregada de uma família (ou do dataset inteiro).
#[derive(Default)]
struct Aggregate {
    n: usize,
    recall: BTreeMap<usize, f64>,
    mrr: f64,
    ndcg: BTreeMap<usize, f64>,
}

impl Aggregate {
    fn add(&mut self, eval: &Eval) {
        self.n += 1;
        for (&k, &value) in &eval.recall {
            *self.recall.entry(k).or_insert(0.0) += value;
        }
        for (&k, &value) in &eval.ndcg {
            *self.ndcg.entry(k).or_insert(0.0) += value;
        }
        self.mrr += eval.mrr;
    }

    fn recall_at(&self, k: usize) -> f64 {
        self.recall.get(&k).copied().unwrap_or(0.0) / self.n.max(1) as f64
    }

    fn ndcg_at(&self, k: usize) -> f64 {
        self.ndcg.get(&k).copied().unwrap_or(0.0) / self.n.max(1) as f64
    }

    fn mean_mrr(&self) -> f64 {
        self.mrr / self.n.max(1) as f64
    }
}

/// Executa a avaliação e grava os relatórios em `md_path`/`json_path`.
pub fn run(md_path: &Path, json_path: &Path) {
    let (notes, queries) = dataset();
    let corpus = Corpus::from_notes(notes).expect("corpus sintético de qualidade é válido");
    let mut families: BTreeMap<&str, Aggregate> = BTreeMap::new();
    let mut overall = Aggregate::default();
    for labeled in &queries {
        let ranked = rank_ids(&corpus, &labeled.query);
        let eval = evaluate(&ranked, &labeled.relevant);
        families.entry(labeled.family).or_default().add(&eval);
        overall.add(&eval);
    }
    let markdown = render_markdown(corpus.notes.len(), queries.len(), &families, &overall);
    print!("{markdown}");
    let _ = std::fs::write(md_path, &markdown);
    let _ = std::fs::write(json_path, render_json(&families, &overall));
}

/// Ordena os ids para a consulta usando o `recall` com os defaults de produção.
fn rank_ids(corpus: &Corpus, text: &str) -> Vec<String> {
    let mut query = RecallQuery::new(text);
    query.limit = 0;
    query.universe = Universe::All;
    match recall(&corpus.index, &corpus.graph, &query) {
        Ok(output) => output.hits.into_iter().map(|hit| hit.id).collect(),
        Err(_) => Vec::new(),
    }
}

/// Calcula Recall@k, MRR e nDCG@k (ganho binário) para uma lista ordenada.
fn evaluate(ranked: &[String], relevant: &BTreeSet<String>) -> Eval {
    let mut eval = Eval::default();
    let total = relevant.len().max(1) as f64;
    for &k in &KS {
        let mut found = 0_usize;
        let mut dcg = 0.0_f64;
        for (index, id) in ranked.iter().take(k).enumerate() {
            if relevant.contains(id) {
                found += 1;
                dcg += 1.0 / ((index as f64) + 2.0).log2();
            }
        }
        let ideal: f64 = (0..relevant.len().min(k))
            .map(|index| 1.0 / ((index as f64) + 2.0).log2())
            .sum();
        let _ = eval.recall.insert(k, found as f64 / total);
        let _ = eval
            .ndcg
            .insert(k, if ideal > 0.0 { dcg / ideal } else { 0.0 });
    }
    eval.mrr = ranked
        .iter()
        .position(|id| relevant.contains(id))
        .map_or(0.0, |index| 1.0 / ((index as f64) + 1.0));
    eval
}

/// Gera o corpus rotulado: notas de tópico (relevantes) + distratores.
fn dataset() -> (Vec<Note>, Vec<LabeledQuery>) {
    let now_ms = 1_700_000_000_000_i64;
    let mut rng = Lcg::new(SEED);
    let mut notes = Vec::new();
    let mut queries = Vec::new();

    for (topic, folded, words) in TOPICS {
        let mut relevant = BTreeSet::new();
        for slot in 0..PER_TOPIC {
            let first = words[rng.pick(words.len())];
            let second = words[rng.pick(words.len())];
            let statement = format!("{topic} {first} {second} variante{slot}");
            let note_id = id::note_id(NoteType::Fact, &statement);
            let _ = relevant.insert(note_id);
            let mut draft = Draft::new(NoteType::Fact, statement);
            draft.body = topic_body(&mut rng, words);
            draft.anchors = vec![format!("src/{folded}/modulo_{slot}.rs")];
            if let Ok(note) = draft.to_note(now_ms) {
                notes.push(note);
            }
        }
        queries.push(LabeledQuery {
            query: topic.to_string(),
            relevant: relevant.clone(),
            family: "com-acento",
        });
        queries.push(LabeledQuery {
            query: folded.to_string(),
            relevant,
            family: "sem-acento",
        });
    }

    // Distratores: notas sem nome de tópico (só vocabulário global), para o lexical não ser trivial.
    for index in 0..(TOPICS.len() * PER_TOPIC) {
        let first = GLOBAL[rng.pick(GLOBAL.len())];
        let second = GLOBAL[rng.pick(GLOBAL.len())];
        let statement = format!("{first} {second} ruido{index}");
        let mut draft = Draft::new(NoteType::Fact, statement);
        draft.body = global_body(&mut rng);
        if let Ok(note) = draft.to_note(now_ms) {
            notes.push(note);
        }
    }

    (notes, queries)
}

/// Corpo de uma nota de tópico: vocabulário global + duas palavras do tópico.
fn topic_body(rng: &mut Lcg, words: &[&str]) -> String {
    let mut out = String::new();
    for index in 0..12 {
        if index > 0 {
            out.push(' ');
        }
        out.push_str(GLOBAL[rng.pick(GLOBAL.len())]);
    }
    out.push(' ');
    out.push_str(words[rng.pick(words.len())]);
    out.push(' ');
    out.push_str(words[rng.pick(words.len())]);
    out
}

/// Corpo de um distrator: só vocabulário global.
fn global_body(rng: &mut Lcg) -> String {
    let mut out = String::new();
    for index in 0..14 {
        if index > 0 {
            out.push(' ');
        }
        out.push_str(GLOBAL[rng.pick(GLOBAL.len())]);
    }
    out
}

/// Formata uma fração `[0,1]` como percentual com uma casa.
fn pct(value: f64) -> String {
    format!("{:.1}%", value * 100.0)
}

/// Renderiza o relatório legível (Markdown).
fn render_markdown(
    notes: usize,
    queries: usize,
    families: &BTreeMap<&str, Aggregate>,
    overall: &Aggregate,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# Qualidade de retrieval — baseline (E16/T01)\n");
    let _ = writeln!(
        out,
        "> Corpus PT-BR sintético e rotulado (**{notes} notas**, **{queries} consultas**) gerado\n\
         > deterministicamente. Métricas sobre o `recall` com os defaults de produção\n\
         > (RRF `k=60`, pesos lexical/âncora/semântico `1/1/30`, sem canal vetorial).\n\
         > `criterion` é não-objetivo (E13-T09): a bancada **observa**, não é gate.\n"
    );
    let _ = writeln!(
        out,
        "| família | n | R@1 | R@3 | R@5 | R@10 | MRR | nDCG@1 | nDCG@3 | nDCG@5 | nDCG@10 |"
    );
    let _ = writeln!(
        out,
        "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"
    );
    for (family, agg) in families {
        let _ = writeln!(out, "{}", row(family, agg));
    }
    let _ = writeln!(out, "{}", row("**geral**", overall));
    out
}

/// Uma linha da tabela de métricas.
fn row(family: &str, agg: &Aggregate) -> String {
    format!(
        "| {family} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
        agg.n,
        pct(agg.recall_at(1)),
        pct(agg.recall_at(3)),
        pct(agg.recall_at(5)),
        pct(agg.recall_at(10)),
        pct(agg.mean_mrr()),
        pct(agg.ndcg_at(1)),
        pct(agg.ndcg_at(3)),
        pct(agg.ndcg_at(5)),
        pct(agg.ndcg_at(10)),
    )
}

/// Renderiza o contrato de máquina (JSON).
fn render_json(families: &BTreeMap<&str, Aggregate>, overall: &Aggregate) -> String {
    let mut entries: Vec<(&str, &Aggregate)> = families.iter().map(|(k, v)| (*k, v)).collect();
    entries.push(("geral", overall));
    let mut out = String::from("[\n");
    for (index, (family, agg)) in entries.iter().enumerate() {
        let comma = if index + 1 == entries.len() { "" } else { "," };
        let _ = writeln!(
            out,
            "  {{\"family\":\"{family}\",\"n\":{},\"recall\":{},\"mrr\":{:.4},\"ndcg\":{}}}{comma}",
            agg.n,
            metrics_json(agg, false),
            agg.mean_mrr(),
            metrics_json(agg, true),
        );
    }
    out.push_str("]\n");
    out
}

/// Objeto JSON `{"1":…,"3":…,"5":…,"10":…}` de recall ou de nDCG.
fn metrics_json(agg: &Aggregate, ndcg: bool) -> String {
    let values: [f64; 4] = if ndcg {
        [
            agg.ndcg_at(1),
            agg.ndcg_at(3),
            agg.ndcg_at(5),
            agg.ndcg_at(10),
        ]
    } else {
        [
            agg.recall_at(1),
            agg.recall_at(3),
            agg.recall_at(5),
            agg.recall_at(10),
        ]
    };
    format!(
        "{{\"1\":{:.4},\"3\":{:.4},\"5\":{:.4},\"10\":{:.4}}}",
        values[0], values[1], values[2], values[3]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(|item| (*item).to_string()).collect()
    }

    fn relevant(list: &[&str]) -> BTreeSet<String> {
        list.iter().map(|item| (*item).to_string()).collect()
    }

    #[test]
    fn perfect_ranking_has_full_metrics() {
        let eval = evaluate(&ids(&["a", "b", "c"]), &relevant(&["a", "b"]));
        assert_eq!(eval.recall.get(&1), Some(&0.5));
        assert_eq!(eval.recall.get(&3), Some(&1.0));
        assert!((eval.mrr - 1.0).abs() < 1e-9);
        assert!((eval.ndcg.get(&3).copied().unwrap_or(0.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn no_relevant_hit_is_zero() {
        let eval = evaluate(&ids(&["x", "y"]), &relevant(&["a"]));
        assert_eq!(eval.recall.get(&1), Some(&0.0));
        assert_eq!(eval.mrr, 0.0);
    }

    #[test]
    fn dataset_is_deterministic_and_labeled() {
        let (first, queries) = dataset();
        let (second, _) = dataset();
        assert_eq!(first.len(), TOPICS.len() * PER_TOPIC * 2);
        assert_eq!(queries.len(), TOPICS.len() * 2);
        assert_eq!(
            first.first().map(|note| note.frontmatter.id().ok()),
            second.first().map(|note| note.frontmatter.id().ok())
        );
    }
}
