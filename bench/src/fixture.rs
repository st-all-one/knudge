//! Geração determinística de corpus para a bancada.
//!
//! Duas saídas: [`generate_notes`] (notas em memória, para micromb) e os geradores
//! JSONL (`write --batch` / `task new --batch`) para o benchmark ponta-a-ponta.

use knudge_core::schema::{Classification, EdgeKind, NoteType, Scope, id};
use knudge_core::store::Note;
use knudge_core::write::Draft;

/// Gerador congruente linear determinístico (não usa `rand`/relógio).
struct Lcg(u64);

impl Lcg {
    const fn new(seed: u64) -> Self {
        Self(seed)
    }
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }
    fn pick(&mut self, len: usize) -> usize {
        (self.next() >> 33) as usize % len.max(1)
    }
}

const WORDS: [&str; 24] = [
    "cache", "indice", "grafo", "nota", "ancora", "fila", "evento", "retrieval", "token", "schema",
    "embedding", "cluster", "decay", "shelf", "tarefa", "epico", "sequencia", "lock", "commit",
    "rebuild", "consulta", "escrita", "leitura", "vetor",
];

const MODULES: [&str; 8] = [
    "core", "cli", "store", "retrieval", "graph", "lifecycle", "health", "embeddings",
];

fn phrase(rng: &mut Lcg, count: usize) -> String {
    let mut out = String::new();
    for index in 0..count {
        if index > 0 {
            out.push(' ');
        }
        out.push_str(WORDS[rng.pick(WORDS.len())]);
    }
    out
}

fn knowledge_statement(rng: &mut Lcg, serial: usize) -> String {
    format!("nota {serial} descreve {} e {}", phrase(rng, 3), phrase(rng, 3))
}

/// Corpo com 3 tokens únicos do serial e janela comum curta: mantém a similaridade
/// (dice) bem abaixo do limiar de dedup, para que o corpus contenha `N` notas distintas.
fn body_of(rng: &mut Lcg, serial: usize) -> String {
    format!(
        "detalhe{serial} contexto{serial} revisao{serial} {}",
        phrase(rng, 12)
    )
}

fn anchor_of(rng: &mut Lcg) -> String {
    format!(
        "src/{}/modulo_{}/arquivo_{}.rs",
        MODULES[rng.pick(MODULES.len())],
        rng.pick(16),
        rng.pick(64),
    )
}

/// Notas em memória: mistura conhecimento + itens de trabalho + épicos, com arestas.
///
/// A composição é ~70% conhecimento, ~20% trabalho, ~10% épicos. Determinística.
#[must_use]
pub fn generate_notes(n: usize) -> Vec<Note> {
    let now_ms = 1_700_000_000_000_i64;
    let mut rng = Lcg::new(0x1234_5678_9abc_def0);
    let knowledge_kinds = [
        NoteType::Fact,
        NoteType::Decision,
        NoteType::Error,
        NoteType::Snippet,
        NoteType::Def,
        NoteType::Link,
        NoteType::Meta,
    ];
    let work_kinds = [
        NoteType::Task,
        NoteType::Error,
        NoteType::Question,
        NoteType::Risk,
        NoteType::Decision,
    ];

    // Passo 1: decide tipo + statement + id de cada nota.
    let mut kinds: Vec<NoteType> = Vec::with_capacity(n);
    let mut statements: Vec<String> = Vec::with_capacity(n);
    let mut ids: Vec<String> = Vec::with_capacity(n);
    let mut epics: Vec<String> = Vec::new();
    for index in 0..n {
        let bucket = index % 10;
        let kind = if bucket == 0 {
            NoteType::Epic
        } else if bucket < 3 {
            work_kinds[index % work_kinds.len()]
        } else {
            knowledge_kinds[index % knowledge_kinds.len()]
        };
        let statement = if kind == NoteType::Epic {
            format!("epico {index} sobre {}", phrase(&mut rng, 4))
        } else {
            knowledge_statement(&mut rng, index)
        };
        let note_id = id::note_id(kind, &statement);
        if kind == NoteType::Epic {
            epics.push(note_id.clone());
        }
        kinds.push(kind);
        statements.push(statement);
        ids.push(note_id);
    }

    // Passo 2: materializa as notas com arestas coerentes.
    let mut notes = Vec::with_capacity(n);
    for index in 0..n {
        let kind = kinds[index];
        let mut draft = Draft::new(kind, statements[index].clone());
        draft.body = body_of(&mut rng, index);
        if kind != NoteType::Epic {
            draft.tags = vec![WORDS[rng.pick(WORDS.len())].to_string()];
            draft.anchors = vec![anchor_of(&mut rng)];
        }
        draft.classification = Some(match index % 3 {
            0 => Classification::Foundational,
            1 => Classification::Tactical,
            _ => Classification::Observational,
        });
        if kind == NoteType::Epic {
            draft.scope = Some(Scope::Epic);
        } else if kind.is_work_kind() && index % 10 < 5 {
            draft.scope = Some(Scope::Task);
            if !epics.is_empty() {
                let epic = epics[index % epics.len()].clone();
                draft.edges.push((EdgeKind::ResultsIn, epic));
            }
        }
        if index > 3 {
            let target = ids[(index * 13 + 7) % index].clone();
            draft.edges.push((EdgeKind::References, target));
        }
        if let Ok(note) = draft.to_note(now_ms) {
            notes.push(note);
        }
    }
    notes
}

/// Renderiza uma string JSON segura (só conteúdo `[a-z0-9 _-]` é usado aqui).
fn json_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c => out.push(c),
        }
    }
    out
}

/// Lote JSONL de conhecimento para `kd write --batch`.
#[must_use]
pub fn knowledge_jsonl(start: usize, count: usize) -> String {
    let mut rng = Lcg::new(0x2545_f491_4f6c_dd1d ^ (start as u64));
    let mut out = String::new();
    for offset in 0..count {
        let index = start + offset;
        let statement = knowledge_statement(&mut rng, index);
        let body = body_of(&mut rng, index);
        let tag = WORDS[rng.pick(WORDS.len())];
        let anchor = anchor_of(&mut rng);
        out.push_str(&format!(
            "{{\"type\":\"fact\",\"statement\":\"{}\",\"body\":\"{}\",\"tags\":[\"{}\"],\"anchors\":[\"{}\"]}}\n",
            json_escape(&statement),
            json_escape(&body),
            tag,
            anchor,
        ));
    }
    out
}

/// Lote JSONL de tarefas para `kd task new --batch`.
///
/// Cria `epics` épicos e `tasks` itens de trabalho pendurados neles. Os itens
/// referenciam o **id real** do épico (não a `key` local), então o lote pode ser
/// dividido em vários arquivos sem quebrar o `parent`.
#[must_use]
pub fn task_jsonl(epics: usize, tasks: usize) -> String {
    let mut rng = Lcg::new(0x9e37_79b9_7f4a_7c15);
    let mut out = String::new();
    let mut epic_ids = Vec::with_capacity(epics);
    for index in 0..epics {
        let statement = format!("epico de trabalho {index} sobre {}", phrase(&mut rng, 4));
        epic_ids.push(id::note_id(NoteType::Epic, &statement));
        out.push_str(&format!(
            "{{\"key\":\"epic{index}\",\"scope\":\"epic\",\"statement\":\"{}\"}}\n",
            json_escape(&statement),
        ));
    }
    for index in 0..tasks {
        let statement = format!("tarefa {index} sobre {}", phrase(&mut rng, 5));
        let anchor = anchor_of(&mut rng);
        let tag = WORDS[rng.pick(WORDS.len())];
        let parent = epic_ids
            .get(index % epic_ids.len().max(1))
            .cloned()
            .unwrap_or_default();
        out.push_str(&format!(
            "{{\"scope\":\"task\",\"kind\":\"task\",\"parent\":\"{parent}\",\"statement\":\"{}\",\"anchors\":[\"{}\"],\"tags\":[\"{}\"],\"checks\":[\"cargo test\"]}}\n",
            json_escape(&statement),
            anchor,
            tag,
        ));
    }
    out
}

/// Consultas representativas para o `ask` (comuns, raras, filtradas).
#[must_use]
pub fn queries() -> &'static [&'static str] {
    &[
        "retrieval indice grafo",
        "cache embedding vetor",
        "shelf life decay ancora",
        "lock commit rebuild",
        "tarefa epico bloqueio",
    ]
}
