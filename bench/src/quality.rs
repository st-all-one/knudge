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
use knudge_core::retrieval::{
    DEFAULT_ANCHOR_WEIGHT, DEFAULT_LEXICAL_WEIGHT, DEFAULT_RRF_K, DEFAULT_SEMANTIC_WEIGHT,
    FusionWeights, RecallQuery, Universe, recall,
};
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

/// Sinônimos por tópico (E16/T09): o canal vetorial sintético "sabe" o tópico sem compartilhar
/// nenhum termo lexical com as notas. Alinhado com [`TOPICS`].
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

/// Consulta rotulada: texto + ids relevantes (relevância binária) + família.
struct LabeledQuery {
    query: String,
    relevant: BTreeSet<String>,
    family: &'static str,
    /// Arquivos do working set (canal de âncoras).
    working_paths: Vec<String>,
    /// Ranking do canal vetorial sintético (canal semântico); `None` = desligado.
    vector: Option<Vec<String>>,
}

/// Configuração da fusão a avaliar (E16/T09): `rrf_k` + pesos dos canais.
#[derive(Clone, Copy)]
struct Config {
    rrf_k: u32,
    weights: FusionWeights,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rrf_k: DEFAULT_RRF_K,
            weights: FusionWeights::default(),
        }
    }
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
    let config = Config::default();
    let mut families: BTreeMap<&str, Aggregate> = BTreeMap::new();
    let mut overall = Aggregate::default();
    for labeled in &queries {
        let ranked = rank_ids(&corpus, labeled, &config);
        let eval = evaluate(&ranked, &labeled.relevant);
        families.entry(labeled.family).or_default().add(&eval);
        overall.add(&eval);
    }
    let markdown = render_markdown(corpus.notes.len(), queries.len(), &families, &overall);
    print!("{markdown}");
    let _ = std::fs::write(md_path, &markdown);
    let _ = std::fs::write(json_path, render_json(&families, &overall));
}

/// Ordena os ids para a consulta usando o `recall` com a configuração dada.
fn rank_ids(corpus: &Corpus, labeled: &LabeledQuery, config: &Config) -> Vec<String> {
    let mut query = RecallQuery::new(&labeled.query);
    query.limit = 0;
    query.universe = Universe::All;
    query.rrf_k = config.rrf_k;
    query.weights = config.weights;
    query.working_paths = labeled.working_paths.clone();
    query.vector = labeled.vector.clone();
    match recall(&corpus.index, &corpus.graph, &query) {
        Ok(output) => output.hits.into_iter().map(|hit| hit.id).collect(),
        Err(_) => Vec::new(),
    }
}

/// Agrega as métricas de todas as consultas sob uma configuração (E16/T09).
fn evaluate_all(corpus: &Corpus, queries: &[LabeledQuery], config: &Config) -> Aggregate {
    let mut overall = Aggregate::default();
    for labeled in queries {
        let ranked = rank_ids(corpus, labeled, config);
        overall.add(&evaluate(&ranked, &labeled.relevant));
    }
    overall
}

/// Linha do `sweep` (E16/T09): uma configuração e as métricas agregadas.
struct Row {
    rrf_k: u32,
    anchor: f64,
    semantic: f64,
    ndcg5: f64,
    mrr: f64,
    recall1: f64,
    ndcg1: f64,
}

/// Varre `rrf_k` × pesos na bancada de qualidade (E16/T09/D179).
///
/// Imprime a linha de base (defaults de produção) e as melhores combinações por
/// `nDCG@5`/`MRR`. **Observação, não gate** (E13-T09): justifica (ou rejeita) um default novo.
pub fn sweep() {
    let (notes, queries) = dataset();
    let corpus = Corpus::from_notes(notes).expect("corpus sintético de qualidade é válido");
    let baseline = evaluate_all(&corpus, &queries, &Config::default());
    let mut rows: Vec<Row> = Vec::new();
    for rrf_k in [1_u32, 5, 10, 20, 40, 60, 100] {
        for anchor in [1.0_f64, 2.0, 4.0, 8.0] {
            for semantic in [1.0_f64, 10.0, 30.0, 60.0] {
                let config = Config {
                    rrf_k,
                    weights: FusionWeights {
                        lexical: DEFAULT_LEXICAL_WEIGHT,
                        anchor,
                        semantic,
                        ppr: 0.0,
                    },
                };
                let agg = evaluate_all(&corpus, &queries, &config);
                rows.push(Row {
                    rrf_k,
                    anchor,
                    semantic,
                    ndcg5: agg.ndcg_at(5),
                    mrr: agg.mean_mrr(),
                    recall1: agg.recall_at(1),
                    ndcg1: agg.ndcg_at(1),
                });
            }
        }
    }
    rows.sort_by(|a, b| {
        b.ndcg5
            .total_cmp(&a.ndcg5)
            .then_with(|| b.mrr.total_cmp(&a.mrr))
            .then_with(|| a.rrf_k.cmp(&b.rrf_k))
            .then_with(|| a.anchor.total_cmp(&b.anchor))
            .then_with(|| a.semantic.total_cmp(&b.semantic))
    });
    println!("# Sweep da fusão (E16/T09/D179)\n");
    println!(
        "Baseline (defaults): rrf_k={DEFAULT_RRF_K} anchor_w={DEFAULT_ANCHOR_WEIGHT:.0} \
         semantic_w={DEFAULT_SEMANTIC_WEIGHT:.0} → nDCG@5={} MRR={} R@1={}\n",
        pct(baseline.ndcg_at(5)),
        pct(baseline.mean_mrr()),
        pct(baseline.recall_at(1)),
    );
    println!("| rrf_k | anchor_w | semantic_w | nDCG@5 | MRR | R@1 | nDCG@1 |");
    println!("|---:|---:|---:|---:|---:|---:|---:|");
    for row in rows.iter().take(20) {
        println!(
            "| {} | {:.0} | {:.0} | {} | {} | {} | {} |",
            row.rrf_k,
            row.anchor,
            row.semantic,
            pct(row.ndcg5),
            pct(row.mrr),
            pct(row.recall1),
            pct(row.ndcg1),
        );
    }
    let best = &rows[0];
    let best_config = Config {
        rrf_k: best.rrf_k,
        weights: FusionWeights {
            lexical: DEFAULT_LEXICAL_WEIGHT,
            anchor: best.anchor,
            semantic: best.semantic,
            ppr: 0.0,
        },
    };
    println!("\n## Candidatos (pesos default salvo indicado)\n");
    println!("| configuração | nDCG@5 | MRR | R@1 |");
    println!("|---|---:|---:|---:|");
    for (name, config) in candidates() {
        let agg = evaluate_all(&corpus, &queries, &config);
        println!(
            "| {name} | {} | {} | {} |",
            pct(agg.ndcg_at(5)),
            pct(agg.mean_mrr()),
            pct(agg.recall_at(1)),
        );
    }
    println!("\n## Por família — baseline (defaults)\n");
    print_families(&corpus, &queries, &Config::default());
    println!(
        "\n## Por família — melhor (rrf_k={} anchor_w={:.0} semantic_w={:.0})\n",
        best.rrf_k, best.anchor, best.semantic
    );
    print_families(&corpus, &queries, &best_config);
}

/// Configurações nomeadas do `sweep` para comparar um lever por vez.
fn candidates() -> Vec<(String, Config)> {
    let mut out = Vec::new();
    for rrf_k in [10_u32, 20, 30, 60] {
        out.push((
            format!("rrf_k={rrf_k} (defaults)"),
            Config {
                rrf_k,
                weights: FusionWeights::default(),
            },
        ));
    }
    for anchor in [2.0_f64, 4.0] {
        out.push((
            format!("anchor_w={anchor:.0} (rrf_k=60)"),
            Config {
                rrf_k: DEFAULT_RRF_K,
                weights: FusionWeights {
                    anchor,
                    ..FusionWeights::default()
                },
            },
        ));
    }
    out
}

/// Tabela por família de uma configuração (auxiliar do `sweep`).
fn print_families(corpus: &Corpus, queries: &[LabeledQuery], config: &Config) {
    let mut families: BTreeMap<&str, Aggregate> = BTreeMap::new();
    let mut overall = Aggregate::default();
    for labeled in queries {
        let ranked = rank_ids(corpus, labeled, config);
        let eval = evaluate(&ranked, &labeled.relevant);
        families.entry(labeled.family).or_default().add(&eval);
        overall.add(&eval);
    }
    println!("| família | n | R@1 | R@5 | MRR | nDCG@5 |");
    println!("|---|---:|---:|---:|---:|---:|");
    for (family, agg) in &families {
        println!(
            "| {family} | {} | {} | {} | {} | {} |",
            agg.n,
            pct(agg.recall_at(1)),
            pct(agg.recall_at(5)),
            pct(agg.mean_mrr()),
            pct(agg.ndcg_at(5)),
        );
    }
    println!(
        "| **geral** | {} | {} | {} | {} | {} |",
        overall.n,
        pct(overall.recall_at(1)),
        pct(overall.recall_at(5)),
        pct(overall.mean_mrr()),
        pct(overall.ndcg_at(5)),
    );
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
    let mut probe = Lcg::new(SEED ^ 0x00dd_1790);
    let mut notes = Vec::new();
    let mut queries = Vec::new();

    for (topic_index, entry) in TOPICS.iter().enumerate() {
        let (topic, folded, words) = *entry;
        let mut relevant = BTreeSet::new();
        let mut topic_ids: Vec<String> = Vec::new();
        for slot in 0..PER_TOPIC {
            let first = words[rng.pick(words.len())];
            let second = words[rng.pick(words.len())];
            let statement = format!("{topic} {first} {second} variante{slot}");
            let note_id = id::note_id(NoteType::Fact, &statement);
            let _ = relevant.insert(note_id.clone());
            let mut draft = Draft::new(NoteType::Fact, statement);
            draft.body = topic_body(&mut rng, words);
            draft.anchors = vec![format!("src/{folded}/modulo_{slot}.rs")];
            if let Ok(note) = draft.to_note(now_ms) {
                topic_ids.push(note_id);
                notes.push(note);
            }
        }
        queries.push(LabeledQuery {
            query: topic.to_string(),
            relevant: relevant.clone(),
            family: "com-acento",
            working_paths: Vec::new(),
            vector: None,
        });
        queries.push(LabeledQuery {
            query: folded.to_string(),
            relevant,
            family: "sem-acento",
            working_paths: Vec::new(),
            vector: None,
        });
        // Famílias multi-canal (E16/T09): exercitam a fusão lexical+âncora+semântico.
        if let Some(first_id) = topic_ids.first() {
            let noise = GLOBAL[probe.pick(GLOBAL.len())];
            queries.push(LabeledQuery {
                query: noise.to_string(),
                relevant: BTreeSet::from([first_id.clone()]),
                family: "working-set",
                working_paths: vec![format!("src/{folded}/modulo_0.rs")],
                vector: None,
            });
        }
        let synonym = SYNONYMS.get(topic_index).copied().unwrap_or(topic);
        let noise = GLOBAL[probe.pick(GLOBAL.len())];
        queries.push(LabeledQuery {
            query: format!("{synonym} {noise}"),
            relevant: topic_ids.iter().cloned().collect(),
            family: "sinonimo",
            working_paths: Vec::new(),
            vector: Some(topic_ids.clone()),
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
         > (RRF `k={DEFAULT_RRF_K}`, pesos lexical/âncora/semântico\n\
         > `{DEFAULT_LEXICAL_WEIGHT}/{DEFAULT_ANCHOR_WEIGHT}/{DEFAULT_SEMANTIC_WEIGHT}`). As famílias\n\
         > `working-set` e `sinonimo` exercitam a fusão com o canal de âncoras (working set) e um\n\
         > canal vetorial sintético.\n\
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
        assert_eq!(queries.len(), TOPICS.len() * 4);
        assert_eq!(
            first.first().map(|note| note.frontmatter.id().ok()),
            second.first().map(|note| note.frontmatter.id().ok())
        );
    }
}
