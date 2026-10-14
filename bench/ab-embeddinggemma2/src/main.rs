//! Harness A/B de qualidade de embeddings sobre o corpus real `_REF/.knudge`.
//!
//! Uso:
//!   ab-harness --scenario <nome> --endpoint <url> --model <id> --dims <n> \
//!              --truncate <0|n> --prefix <none|gemma> --out <arquivo.json>
//!
//! Mede recuperação **semântica pura** (cosseno sobre vetores normalizados) em 5 famílias
//! de consulta construídas a partir das próprias notas (known-item + relacionadas).

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use knudge_core::adapters::{HttpEmbedder, StdEnv};
use knudge_core::config::Config;
use knudge_core::embeddings::embedding_text;
use knudge_core::embeddings::vector::{dot, l2_norm};
use knudge_core::ports::Embedder;
use knudge_core::retrieval::{RecallQuery, Universe, recall};
use knudge_core::{Knudge, Result};

const KS: [usize; 4] = [1, 3, 5, 10];

#[derive(Default, Clone)]
struct Agg {
    n: usize,
    recall: BTreeMap<usize, f64>,
    mrr: f64,
    ndcg10: f64,
}

impl Agg {
    fn add(&mut self, ranked: &[String], relevant: &BTreeSet<String>) {
        self.n += 1;
        let total = relevant.len().max(1) as f64;
        for &k in &KS {
            let found = ranked.iter().take(k).filter(|id| relevant.contains(*id)).count() as f64;
            *self.recall.entry(k).or_insert(0.0) += found / total;
        }
        let rank = ranked.iter().position(|id| relevant.contains(id));
        self.mrr += rank.map_or(0.0, |i| 1.0 / (i as f64 + 1.0));
        // nDCG@10, ganho binário.
        let mut dcg = 0.0;
        for (i, id) in ranked.iter().take(10).enumerate() {
            if relevant.contains(id) {
                dcg += 1.0 / ((i as f64) + 2.0).log2();
            }
        }
        let ideal: f64 = (0..relevant.len().min(10))
            .map(|i| 1.0 / ((i as f64) + 2.0).log2())
            .sum();
        self.ndcg10 += if ideal > 0.0 { dcg / ideal } else { 0.0 };
    }
    fn merge(&mut self, other: &Agg) {
        self.n += other.n;
        for (k, v) in &other.recall {
            *self.recall.entry(*k).or_insert(0.0) += v;
        }
        self.mrr += other.mrr;
        self.ndcg10 += other.ndcg10;
    }
    fn r(&self, k: usize) -> f64 {
        if self.n == 0 { 0.0 } else { self.recall.get(&k).copied().unwrap_or(0.0) / self.n as f64 }
    }
    fn m(&self) -> f64 {
        if self.n == 0 { 0.0 } else { self.mrr / self.n as f64 }
    }
    fn ndcg(&self) -> f64 {
        if self.n == 0 { 0.0 } else { self.ndcg10 / self.n as f64 }
    }
}

struct Args {
    scenario: String,
    root: String,
    endpoint: String,
    model: String,
    dims: usize,
    truncate: usize,
    prefix: String,
    out: PathBuf,
    dump: PathBuf,
    hybrid: bool,
    limit: usize,
    families: Vec<String>,
}

fn parse_args() -> Args {
    let mut a = Args {
        scenario: "x".into(),
        root: "/home/one/projects/one/knudge/_REF".into(),
        endpoint: "http://127.0.0.1:8890/v1/embeddings".into(),
        model: "m".into(),
        dims: 384,
        truncate: 0,
        prefix: "none".into(),
        out: PathBuf::from("/tmp/ab/out.json"),
        dump: PathBuf::from("/tmp/ab/ranks.tsv"),
        hybrid: false,
        limit: usize::MAX,
        families: vec!["self", "kw", "body", "anchor", "tag"].into_iter().map(String::from).collect(),
    };
    let mut it = std::env::args().skip(1);
    while let Some(k) = it.next() {
        let v = it.next().unwrap_or_default();
        match k.as_str() {
            "--scenario" => a.scenario = v,
            "--root" => a.root = v,
            "--endpoint" => a.endpoint = v,
            "--model" => a.model = v,
            "--dims" => a.dims = v.parse().unwrap(),
            "--truncate" => a.truncate = v.parse().unwrap(),
            "--prefix" => a.prefix = v,
            "--out" => a.out = PathBuf::from(v),
            "--dump" => a.dump = PathBuf::from(v),
            "--hybrid" => a.hybrid = v == "1" || v == "true",
            "--limit" => a.limit = v.parse().unwrap(),
            "--families" => a.families = v.split(',').map(String::from).collect(),
            other => panic!("arg desconhecido: {other}"),
        }
    }
    a
}

fn doc_text(prefix: &str, statement: &str, body: &str) -> String {
    let raw = embedding_text(statement, body);
    let b = body.trim();
    match prefix {
        "gemma" => {
            if b.is_empty() {
                format!("title: none | text: {statement}")
            } else {
                format!("title: none | text: {raw}")
            }
        }
        "gemmatitle" => {
            if b.is_empty() {
                format!("title: {statement} | text: {statement}")
            } else {
                format!("title: {statement} | text: {b}")
            }
        }
        _ => raw,
    }
}

fn query_text(prefix: &str, q: &str) -> String {
    match prefix {
        "gemma" | "gemmatitle" => format!("task: search result | query: {q}"),
        _ => q.to_string(),
    }
}

/// Palavras de parada PT/EN para a família `kw`.
const STOP: &[&str] = &[
    "a", "o", "as", "os", "de", "da", "do", "das", "dos", "em", "no", "na", "nos", "nas",
    "um", "uma", "uns", "umas", "para", "por", "com", "sem", "que", "se", "e", "ou", "ao",
    "aos", "à", "às", "the", "of", "to", "in", "and", "is", "are", "for", "on", "with",
    "não", "nao", "é", "eh", "ser", "são", "sao", "como", "mais", "ja", "já",
];

fn keywords(statement: &str) -> String {
    let folded: String = statement
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'á'|'à'|'â'|'ã'|'ä' => 'a', 'é'|'è'|'ê'|'ë' => 'e', 'í'|'ì'|'î'|'ï' => 'i',
            'ó'|'ò'|'ô'|'õ'|'ö' => 'o', 'ú'|'ù'|'û'|'ü' => 'u', 'ç' => 'c', c => c,
        })
        .collect();
    let toks: Vec<&str> = folded
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 3 && !STOP.contains(t))
        .collect();
    toks.iter().take(10).copied().collect::<Vec<_>>().join(" ")
}

fn first_sentence(body: &str, max: usize) -> String {
    let end = body.find(". ").map_or(body.len(), |i| i + 1);
    let s: String = body.chars().take(max.min(end)).collect();
    s.trim().to_string()
}

fn main() {
    if let Err(e) = run() {
        eprintln!("erro: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = parse_args();
    let kd = Knudge::builder().root(args.root.clone()).open()?;
    let notes = kd.notes()?;

    // Extrai campos.
    struct Rec {
        id: String,
        statement: String,
        body: String,
        tags: Vec<String>,
        anchors: Vec<String>,
    }
    let mut recs = Vec::new();
    for n in &notes {
        let Ok(id) = n.frontmatter.id() else { continue };
        let Ok(statement) = n.frontmatter.statement() else { continue };
        let tags = n.frontmatter.string_list("tags").unwrap_or_default().iter().map(|s| s.to_string()).collect();
        let anchors = n.frontmatter.string_list("anchors").unwrap_or_default().iter().map(|s| s.to_string()).collect();
        recs.push(Rec {
            id: id.to_string(),
            statement: statement.to_string(),
            body: n.body.clone(),
            tags,
            anchors,
        });
    }
    eprintln!("notas: {}", recs.len());

    // Config para o embedder.
    // http://... endpoint e model vêm do CLI; dimensions = args.dims (meta declarada).
    let toml = format!(
        "[embeddings]\nprovider = \"http\"\nendpoint = \"{}\"\nmodel = \"{}\"\nrevision = \"ab\"\nsimilarity = \"cosine\"\ndimensions = {}\n",
        args.endpoint, args.model, args.dims
    );
    let cfg = Config::parse(&toml)?;
    let env = StdEnv::new();
    let embedder = HttpEmbedder::new(&cfg, &env)?;
    eprintln!("embedder meta: {:?} dims={}", embedder.meta().model, embedder.meta().dimensions);

    let batch = 32;
    let embed_all = |texts: &[String]| -> Vec<Vec<f32>> {
        let mut out = Vec::with_capacity(texts.len());
        for chunk in texts.chunks(batch) {
            let vs = embedder.embed(chunk).unwrap_or_else(|e| panic!("embed falhou: {e}"));
            for mut v in vs {
                if args.truncate > 0 && v.len() > args.truncate {
                    v.truncate(args.truncate);
                    let norm: f32 = l2_norm(&v);
                    if norm > 0.0 {
                        for x in &mut v {
                            *x /= norm;
                        }
                    }
                }
                out.push(v);
            }
            eprint!(".");
        }
        eprintln!();
        out
    };

    // Textos de documento + de consulta (dedup de consultas).
    let docs: Vec<String> = recs
        .iter()
        .map(|r| doc_text(&args.prefix, &r.statement, &r.body))
        .collect();

    let mut queries: Vec<(String, String)> = Vec::new();
    let mut families: BTreeMap<String, Vec<(usize, BTreeSet<String>)>> = BTreeMap::new();
    // ids por tag/âncora
    let mut by_tag: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
    let mut by_anchor: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
    for (i, r) in recs.iter().enumerate() {
        for t in &r.tags {
            by_tag.entry(t.clone()).or_default().insert(i);
        }
        for a in &r.anchors {
            by_anchor.entry(a.clone()).or_default().insert(i);
        }
    }

    let want = |name: &str| args.families.iter().any(|f| f == name);
    let mut generated = 0_usize;
    for (i, r) in recs.iter().enumerate() {
        if generated >= args.limit {
            break;
        }
        generated += 1;
        let raw_stmt = r.statement.clone();
        let raw_kw = keywords(&r.statement);
        let raw_body = first_sentence(&r.body, 400);
        let q_stmt = (raw_stmt, query_text(&args.prefix, &r.statement));
        let q_kw = (raw_kw, query_text(&args.prefix, &keywords(&r.statement)));
        let q_body = (raw_body, query_text(&args.prefix, &first_sentence(&r.body, 400)));

        if want("self") {
            let qi = queries.len();
            queries.push(q_stmt.clone());
            families.entry("self".into()).or_default().push((qi, BTreeSet::from([r.id.clone()])));
        }
        if want("kw") {
            let qi = queries.len();
            queries.push(q_kw);
            families.entry("kw".into()).or_default().push((qi, BTreeSet::from([r.id.clone()])));
        }
        if want("body") {
            if first_sentence(&r.body, 400).chars().count() >= 20 {
                let qi = queries.len();
                queries.push(q_body);
                families.entry("body".into()).or_default().push((qi, BTreeSet::from([r.id.clone()])));
            }
        }
        // Para anchor/tag a consulta é o statement; reaproveita o índice de q_stmt.
        if want("anchor") && !r.anchors.is_empty() {
            let mut rel = BTreeSet::new();
            for a in &r.anchors {
                if let Some(s) = by_anchor.get(a) {
                    for &j in s {
                        if j != i {
                            rel.insert(recs[j].id.clone());
                        }
                    }
                }
            }
            if (2..=80).contains(&rel.len()) {
                let qi = queries.iter().position(|q| q.1 == q_stmt.1).unwrap();
                families.entry("anchor".into()).or_default().push((qi, rel));
            }
        }
        if want("tag") && !r.tags.is_empty() {
            let mut rel = BTreeSet::new();
            for t in &r.tags {
                if let Some(s) = by_tag.get(t) {
                    for &j in s {
                        if j != i {
                            rel.insert(recs[j].id.clone());
                        }
                    }
                }
            }
            if (2..=80).contains(&rel.len()) {
                let qi = queries.iter().position(|q| q.1 == q_stmt.1).unwrap();
                families.entry("tag".into()).or_default().push((qi, rel));
            }
        }
    }

    eprintln!("docs: {} | consultas únicas: {}", docs.len(), queries.len());
    let t0 = std::time::Instant::now();
    let doc_vecs = embed_all(&docs);
    let q_texts: Vec<String> = queries.iter().map(|q| q.1.clone()).collect();
    let q_vecs = embed_all(&q_texts);
    eprintln!("embeddings em {:.1}s", t0.elapsed().as_secs_f64());

    // Índice e grafo reais para a fusão híbrida.
    let index = kd.index()?;
    let graph = kd.graph()?;

    // Ranking semântico puro por consulta.
    let ids: Vec<&str> = recs.iter().map(|r| r.id.as_str()).collect();
    let mut agg: BTreeMap<String, Agg> = BTreeMap::new();
    let mut agg_hybrid: BTreeMap<String, Agg> = BTreeMap::new();
    let mut dump = String::new();
    for (fam, items) in &families {
        let mut a = Agg::default();
        let mut ah = Agg::default();
        for (qi, relevant) in items {
            let qv = &q_vecs[*qi];
            let mut scored: Vec<(f32, &str)> = doc_vecs
                .iter()
                .enumerate()
                .map(|(j, dv)| (dot(qv, dv), ids[j]))
                .collect();
            scored.sort_by(|x, y| y.0.total_cmp(&x.0).then_with(|| x.1.cmp(y.1)));
            // Rank (0-based) do primeiro relevante sobre TODOS os documentos.
            let rank = scored
                .iter()
                .position(|(_, id)| relevant.contains(*id))
                .map_or(usize::MAX, |p| p);
            dump.push_str(&format!("{fam}\t{rank}\n"));
            let full: Vec<String> = scored.iter().map(|(_, s)| s.to_string()).collect();
            let ranked: Vec<String> = full.iter().take(50).cloned().collect();
            a.add(&ranked, relevant);

            // Fusão híbrida real (BM25 + âncoras + RRF + vetor do modelo).
            if args.hybrid {
                let mut q = RecallQuery::new(queries[*qi].0.clone());
                q.limit = 50;
                q.universe = Universe::All;
                q.vector = Some(full);
                if let Ok(out) = recall(&index, &graph, &q) {
                    let h: Vec<String> = out.hits.iter().map(|h| h.id.clone()).collect();
                    ah.add(&h, relevant);
                }
            }
        }
        agg.insert(fam.clone(), a);
        if args.hybrid {
            agg_hybrid.insert(fam.clone(), ah);
        }
    }
    std::fs::write(&args.dump, dump).unwrap();

    // Saída JSON manual (sem serde).
    let mut json = String::from("{\n");
    json.push_str(&format!("  \"scenario\": \"{}\",\n", args.scenario));
    json.push_str(&format!("  \"model\": \"{}\",\n", args.model));
    json.push_str(&format!("  \"dims\": {},\n", args.dims));
    json.push_str(&format!("  \"truncate\": {},\n", args.truncate));
    json.push_str(&format!("  \"prefix\": \"{}\",\n", args.prefix));
    json.push_str(&format!("  \"notes\": {},\n", recs.len()));
    json.push_str(&format!("  \"seconds\": {:.2},\n", t0.elapsed().as_secs_f64()));
    json.push_str("  \"families\": {\n");
    let mut first = true;
    for (fam, a) in &agg {
        if !first { json.push_str(",\n"); }
        first = false;
        json.push_str(&format!(
            "    \"{fam}\": {{\"n\": {}, \"R@1\": {:.4}, \"R@3\": {:.4}, \"R@5\": {:.4}, \"R@10\": {:.4}, \"MRR\": {:.4}, \"nDCG@10\": {:.4}}}",
            a.n, a.r(1), a.r(3), a.r(5), a.r(10), a.m(), a.ndcg()
        ));
    }
    // geral = média ponderada por n
    let mut overall = Agg::default();
    for a in agg.values() { overall.merge(a); }
    json.push_str(",\n");
    json.push_str(&format!(
        "    \"__overall\": {{\"n\": {}, \"R@1\": {:.4}, \"R@3\": {:.4}, \"R@5\": {:.4}, \"R@10\": {:.4}, \"MRR\": {:.4}, \"nDCG@10\": {:.4}}}",
        overall.n, overall.r(1), overall.r(3), overall.r(5), overall.r(10), overall.m(), overall.ndcg()
    ));
    json.push_str("\n  }\n");
    if args.hybrid {
        json.push_str("  ,\"families_hybrid\": {\n");
        let mut first = true;
        for (fam, a) in &agg_hybrid {
            if !first { json.push_str(",\n"); }
            first = false;
            json.push_str(&format!(
                "    \"{fam}\": {{\"n\": {}, \"R@1\": {:.4}, \"R@3\": {:.4}, \"R@5\": {:.4}, \"R@10\": {:.4}, \"MRR\": {:.4}, \"nDCG@10\": {:.4}}}",
                a.n, a.r(1), a.r(3), a.r(5), a.r(10), a.m(), a.ndcg()
            ));
        }
        let mut ov = Agg::default();
        for a in agg_hybrid.values() { ov.merge(a); }
        json.push_str(",\n");
        json.push_str(&format!(
            "    \"__overall\": {{\"n\": {}, \"R@1\": {:.4}, \"R@3\": {:.4}, \"R@5\": {:.4}, \"R@10\": {:.4}, \"MRR\": {:.4}, \"nDCG@10\": {:.4}}}\n  }}",
            ov.n, ov.r(1), ov.r(3), ov.r(5), ov.r(10), ov.m(), ov.ndcg()
        ));
    }
    json.push_str("\n}\n");
    std::fs::write(&args.out, &json).unwrap();
    eprintln!("resultado: {}", args.out.display());
    println!("{json}");
    Ok(())
}
