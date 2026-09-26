//! Micromb: mede cada componente puro do `knudge-core` com `std::time::Instant`.
//!
//! Duas seções: operações de custo fixo (dependem só do tamanho da entrada) e
//! operações que escalam com o corpus `N`.

use std::collections::BTreeSet;
use std::hint::black_box;

use knudge_core::Graph;
use knudge_core::config::Config;
use knudge_core::embeddings::lightweight::embed;
use knudge_core::embeddings::vector::cosine;
use knudge_core::handoff::budget::apply;
use knudge_core::handoff::estimate_tokens;
use knudge_core::handoff::{RewindMode, rank as handoff_rank};
use knudge_core::jsonl;
use knudge_core::lifecycle::clusters::structural_clusters;
use knudge_core::lifecycle::confidence::{ConfidenceInput, confidence_score};
use knudge_core::retrieval::anchor::{GlobPattern, glob_match};
use knudge_core::retrieval::filter::Filter;
use knudge_core::retrieval::rrf::{Channel, fuse};
use knudge_core::retrieval::token;
use knudge_core::retrieval::{
    Index, Postings, RankQuery, RecallQuery, Universe, compute_views, rank as composite_rank, recall,
};
use knudge_core::schema::{NoteType, body, hash, id};
use knudge_core::task::{impact, impacts};
use knudge_core::toon;
use knudge_core::write::{DedupThresholds, Draft, propose_merges};

use crate::fixture;
use crate::harness::Harness;

const SMALL_JSON: &str =
    "{\"id\":\"fact_00000001\",\"statement\":\"o indice e derivado\",\"tags\":[\"retrieval\"],\"metrics\":{\"n\":42,\"ok\":true}}";

const SAMPLE_TOML: &str = "[recall]\ndefault_limit = 5\nrrf_k = 60\nlexical_weight = 1.0\nsemantic_weight = 30.0\nsemantic = true\n\n[behavior]\nstrict = false\n\n[embeddings]\nmode = \"lazy\"\nprovider = \"none\"\ndimensions = 384\n";

/// Roda toda a micromb.
pub fn run(harness: &mut Harness, sizes: &[usize]) {
    fixed(harness);
    scaling(harness, sizes);
}

fn fixed(harness: &mut Harness) {
    let notes = fixture::generate_notes(1);
    let sample = notes.first().expect("fixture gera ao menos uma nota");
    let statement = sample.frontmatter.statement().unwrap_or_default().to_string();
    let note_body = sample.body.clone();
    let rendered = sample.render();
    let front = sample.frontmatter.to_string();
    let parsed_value = toon::parse(&front).expect("frontmatter gerado é TOON válido");
    let decoded_json = jsonl::decode(SMALL_JSON).expect("JSON fixo é válido");
    let long_text = note_body.repeat(8);
    let query = format!("{statement} {}", note_body);
    let statement2 = format!("{statement} (derivada)");

    // --- Schema: normalização, hash e id ---
    harness.measure("micro/fixo", "schema::body::normalize (curto ~200B)", 25, 2000, || {
        let _ = black_box(body::normalize(black_box(&note_body)));
    });
    harness.measure("micro/fixo", "schema::body::normalize (longo ~1.6KB)", 25, 500, || {
        let _ = black_box(body::normalize(black_box(&long_text)));
    });
    harness.measure("micro/fixo", "schema::body::body_hash", 25, 2000, || {
        let _ = black_box(body::body_hash(black_box(&statement), black_box(&note_body)));
    });
    harness.measure("micro/fixo", "schema::id::note_id", 25, 2000, || {
        let _ = black_box(id::note_id(black_box(NoteType::Fact), black_box(&statement2)));
    });
    harness.measure("micro/fixo", "schema::hash::short_hash", 25, 5000, || {
        let _ = black_box(hash::short_hash(black_box(long_text.as_bytes())));
    });
    harness.measure("micro/fixo", "schema::hash::base36_8", 25, 20000, || {
        let _ = black_box(hash::base36_8(black_box(0xdead_beef_u32)));
    });

    // --- TOON ---
    harness.measure("micro/fixo", "toon::parse (frontmatter)", 25, 1000, || {
        let _ = black_box(toon::parse(black_box(&front)));
    });
    harness.measure("micro/fixo", "toon::emit (frontmatter)", 25, 1000, || {
        let _ = black_box(toon::emit(black_box(&parsed_value)));
    });
    harness.measure("micro/fixo", "Note::parse (render completo)", 25, 1000, || {
        let _ = black_box(knudge_core::store::Note::parse(black_box(rendered.as_bytes())));
    });
    harness.measure("micro/fixo", "Note::render", 25, 2000, || {
        let _ = black_box(sample.render());
    });

    // --- JSONL ---
    harness.measure("micro/fixo", "jsonl::decode", 25, 3000, || {
        let _ = black_box(jsonl::decode(black_box(SMALL_JSON)));
    });
    harness.measure("micro/fixo", "jsonl::encode", 25, 3000, || {
        let _ = black_box(jsonl::encode(black_box(&decoded_json)));
    });

    // --- Tokenização ---
    harness.measure("micro/fixo", "retrieval::token::tokenize (corpo)", 25, 1000, || {
        let _ = black_box(token::tokenize(black_box(&note_body)));
    });
    harness.measure("micro/fixo", "retrieval::token::content_terms", 25, 1000, || {
        let _ = black_box(token::content_terms(black_box(&query)));
    });

    // --- RRF ---
    let lists: Vec<Vec<String>> = (0..3)
        .map(|channel| {
            (0..200)
                .map(|rank| format!("fact_{channel}_{rank:08}"))
                .collect()
        })
        .collect();
    let channels: Vec<Channel<'_>> = lists
        .iter()
        .enumerate()
        .map(|(index, list)| {
            let weight = match index {
                0 => 1.0,
                1 => 1.0,
                _ => 30.0,
            };
            Channel::new(list, weight)
        })
        .collect();
    harness.measure("micro/fixo", "retrieval::rrf::fuse (3x200)", 25, 200, || {
        let _ = black_box(fuse(black_box(&channels), 60));
    });

    // --- Confiança e orçamento ---
    let confidence_input = ConfidenceInput {
        similarity: 0.7,
        confirmation: 0.4,
        age_days: 12.0,
        task_confirmation: 0.1,
        ..ConfidenceInput::default()
    };
    harness.measure("micro/fixo", "lifecycle::confidence_score", 25, 100000, || {
        let _ = black_box(confidence_score(black_box(&confidence_input)));
    });
    let lines: Vec<String> = (0..1000)
        .map(|index| format!("item {index}|statement sintetica de teste com cerca de 60 chars"))
        .collect();
    harness.measure("micro/fixo", "handoff::budget::estimate_tokens", 25, 2000, || {
        let _ = black_box(estimate_tokens(black_box(&long_text)));
    });
    harness.measure("micro/fixo", "handoff::budget::apply (1000 linhas)", 25, 200, || {
        let _ = black_box(apply(black_box(&lines), 4000));
    });

    // --- Config ---
    harness.measure("micro/fixo", "config::Config::parse", 25, 2000, || {
        let _ = black_box(Config::parse(black_box(SAMPLE_TOML)));
    });

    // --- Globs (E15-T07/O2.3) ---
    let pattern = GlobPattern::new("src/**/*.rs");
    harness.measure("micro/fixo", "retrieval::anchor::glob_match", 25, 20000, || {
        let _ = black_box(glob_match(
            black_box("src/**/*.rs"),
            black_box("src/a/b/main.rs"),
        ));
    });
    harness.measure(
        "micro/fixo",
        "retrieval::anchor::GlobPattern::matches",
        25,
        20000,
        || {
            let _ = black_box(pattern.matches(black_box("src/a/b/main.rs")));
        },
    );

    // --- Embeddings lightweight (384d) ---
    let vector_a = embed(&note_body, 384);
    let vector_b = embed(&long_text, 384);
    harness.measure("micro/fixo", "embeddings::lightweight::embed (384d)", 25, 1000, || {
        let _ = black_box(embed(black_box(&note_body), 384));
    });
    harness.measure("micro/fixo", "embeddings::vector::cosine (384d)", 25, 100000, || {
        let _ = black_box(cosine(black_box(&vector_a), black_box(&vector_b)));
    });
}

fn scaling(harness: &mut Harness, sizes: &[usize]) {
    let filter = Filter::new();
    let thresholds = DedupThresholds::default();
    for &n in sizes {
        let group = format!("micro/N={n}");
        let notes = fixture::generate_notes(n);
        let index = Index::build(&notes).expect("corpus gerado é válido");
        let sparse = sparse_index(n);
        let graph = Graph::from_notes(notes.clone()).expect("grafo gerado é válido");
        let allowed: BTreeSet<String> = index.docs.iter().map(|doc| doc.meta.id.clone()).collect();
        let query_text = "retrieval indice grafo cache embedding vetor";
        let recall_query = RecallQuery {
            text: query_text.to_string(),
            limit: 5,
            universe: Universe::All,
            ..RecallQuery::default()
        };
        let rank_query = RankQuery {
            universe: Universe::All,
            limit: 5,
            ..RankQuery::default()
        };
        let serialized = index.serialize().expect("índice serializa");

        harness.measure(&group, "retrieval::Index::build", 15, 1, || {
            let _ = black_box(Index::build(black_box(&notes)));
        });
        harness.measure(&group, "retrieval::Postings::build", 15, 1, || {
            let _ = black_box(Postings::build(black_box(&index)));
        });
        harness.measure(&group, "retrieval::Index::score (BM25)", 15, 1, || {
            let _ = black_box(index.score(black_box(query_text), black_box(&allowed)));
        });
        harness.measure(&group, "write::propose_merges (denso)", 15, 1, || {
            let _ = black_box(propose_merges(black_box(&index), black_box(&thresholds)));
        });
        harness.measure(&group, "write::propose_merges (esparso)", 15, 1, || {
            let _ = black_box(propose_merges(black_box(&sparse), black_box(&thresholds)));
        });
        harness.measure(&group, "retrieval::recall (limit 5)", 15, 1, || {
            let _ = black_box(recall(black_box(&index), black_box(&graph), black_box(&recall_query)));
        });
        let big_query = RecallQuery {
            text: query_text.to_string(),
            limit: 0,
            universe: Universe::All,
            ..RecallQuery::default()
        };
        harness.measure(&group, "retrieval::recall (sem limite)", 15, 1, || {
            let _ = black_box(recall(black_box(&index), black_box(&graph), black_box(&big_query)));
        });
        harness.measure(&group, "retrieval::rank (confiança)", 15, 1, || {
            let _ = black_box(composite_rank(
                black_box(&index),
                black_box(&filter),
                black_box(&rank_query),
            ));
        });
        harness.measure(&group, "lifecycle::structural_clusters", 15, 1, || {
            let _ = black_box(structural_clusters(black_box(&index), black_box(&graph)));
        });
        harness.measure(&group, "Graph::from_notes", 15, 1, || {
            let _ = black_box(Graph::from_notes(black_box(notes.clone())));
        });
        harness.measure(&group, "Graph::integrity", 15, 1, || {
            let _ = black_box(graph.integrity());
        });
        harness.measure(&group, "Graph::supersession_cycles", 15, 1, || {
            let _ = black_box(graph.supersession_cycles());
        });
        harness.measure(&group, "Graph::dependency_cycles", 15, 1, || {
            let _ = black_box(graph.dependency_cycles());
        });
        harness.measure(&group, "retrieval::compute_views", 15, 1, || {
            let _ = black_box(compute_views(black_box(&graph)));
        });
        let some_id = index.docs.first().map(|doc| doc.meta.id.clone());
        harness.measure(&group, "task::impact (1 id)", 15, 1, || {
            if let Some(id) = &some_id {
                let _ = black_box(impact(black_box(&graph), black_box(id)));
            }
        });
        harness.measure(&group, "task::impacts (todos)", 15, 1, || {
            let _ = black_box(impacts(black_box(&graph)));
        });
        harness.measure(&group, "handoff::rank (manifest)", 15, 1, || {
            let _ = black_box(handoff_rank(
                black_box(&index),
                black_box(&graph),
                black_box(&RewindMode::Manifest),
            ));
        });
        harness.measure(&group, "Index::serialize + parse", 15, 1, || {
            let text = index.serialize().expect("serializa");
            let _ = black_box(Index::parse(black_box(&text)));
        });
        let index_text = index.serialize().expect("serializa");
        harness.measure(&group, "Index::serialize", 15, 1, || {
            let _ = black_box(index.serialize());
        });
        harness.measure(&group, "Index::parse", 15, 1, || {
            let _ = black_box(Index::parse(black_box(&index_text)));
        });
        harness.measure(&group, "write::Draft::to_note", 15, 200, || {
            let draft = Draft::new(NoteType::Fact, "afirmacao sintetica para medir to_note");
            let _ = black_box(draft.to_note(1_700_000_000_000));
        });
        // Guarda o serializado para evitar dead code em builds futuros.
        let _ = black_box(serialized.len());
    }
}

/// Índice com vocabulário **esparso** (cada doc tem só termos próprios): a peneira do dedup
/// (O3) deve pular todo o corpus, mostrando o ganho sobre a varredura completa.
fn sparse_index(n: usize) -> Index {
    let mut notes = Vec::new();
    for serial in 0..n {
        let statement = format!("proprio{serial} exclusivo{serial} unico{serial}");
        if let Ok(note) = Draft::new(NoteType::Fact, statement).to_note(1_700_000_000_000) {
            notes.push(note);
        }
    }
    Index::build(&notes).expect("corpus esparso é válido")
}
