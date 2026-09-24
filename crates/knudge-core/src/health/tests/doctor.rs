//! Testes do `doctor [--fix]` (E09-T04).

use std::path::Path;

use crate::Result;
use crate::config::Config;
use crate::graph::Graph;
use crate::health::{CheckId, DoctorInput, doctor, doctor_fix};
use crate::ports::Fs;
use crate::ports::fakes::MemFs;
use crate::retrieval::Index;
use crate::schema::{NoteType, Scope, Value};
use crate::store::EventLog;
use crate::write::dedup::DedupThresholds;
use crate::write::{Draft, WriteContext};

use super::{NOW, PROJECT, ROOT, anchored, built, note, seeded};

struct World<'a> {
    fs: &'a MemFs,
    ctx: &'a WriteContext<'a>,
    events: &'a EventLog<'a>,
    config: &'a Config,
    graph: &'a Graph,
    thresholds: &'a DedupThresholds,
}

impl World<'_> {
    fn input(&self) -> DoctorInput<'_> {
        DoctorInput {
            fs: self.fs,
            root: Path::new(ROOT),
            project_root: Path::new(PROJECT),
            store: self.ctx.store(),
            events: self.events,
            config: self.config,
            graph: self.graph,
            now_ms: NOW,
            lock_stale_ms: 30_000,
            thresholds: self.thresholds,
        }
    }
}

#[test]
fn doctor_fix_is_idempotent() -> Result<()> {
    let fs = MemFs::new();
    let fact = note(NoteType::Fact, "saúde", "")?;
    let id = fact.id()?.to_string();
    let ctx = seeded(&fs, std::slice::from_ref(&fact))?;
    let events = EventLog::new(&fs, ROOT, EventLog::DEFAULT_MAX_BYTES);
    let config = Config::defaults();
    let (_index, graph) = built(std::slice::from_ref(&fact))?;
    let thresholds = DedupThresholds::default();
    let world = World {
        fs: &fs,
        ctx: &ctx,
        events: &events,
        config: &config,
        graph: &graph,
        thresholds: &thresholds,
    };

    // Corrompe o body_hash.
    let mut stored = ctx.store().read(&id)?;
    stored
        .frontmatter
        .set("body_hash", Value::Str("00000000".to_string()))?;
    ctx.store().write(&stored)?;

    let args = world.input();
    let before = doctor(&args)?;
    assert!(!before.is_healthy());
    assert_eq!(before.check(CheckId::BodyHash).map(|c| c.ok), Some(false));
    assert_eq!(before.check(CheckId::Derived).map(|c| c.ok), Some(false));

    let fixed = doctor_fix(&args)?;
    assert!(fixed.is_healthy(), "{fixed:?}");
    assert!(!fixed.fixed.is_empty());

    let again = doctor_fix(&args)?;
    assert!(
        again.fixed.is_empty(),
        "segunda passada mexeu: {:?}",
        again.fixed
    );
    Ok(())
}

#[test]
fn missing_index_is_rebuilt() -> Result<()> {
    let fs = MemFs::new();
    let fact = note(NoteType::Fact, "índice", "")?;
    let ctx = seeded(&fs, std::slice::from_ref(&fact))?;
    let events = EventLog::new(&fs, ROOT, EventLog::DEFAULT_MAX_BYTES);
    let config = Config::defaults();
    let (_index, graph) = built(std::slice::from_ref(&fact))?;
    let thresholds = DedupThresholds::default();
    let world = World {
        fs: &fs,
        ctx: &ctx,
        events: &events,
        config: &config,
        graph: &graph,
        thresholds: &thresholds,
    };
    let args = world.input();

    let _fixed = doctor_fix(&args)?;
    assert!(fs.exists(&Index::path(Path::new(ROOT))));
    let report = doctor(&args)?;
    assert_eq!(report.check(CheckId::Derived).map(|c| c.ok), Some(true));
    Ok(())
}

#[test]
fn stale_lock_is_removed() -> Result<()> {
    let fs = MemFs::new();
    let fact = note(NoteType::Fact, "lock", "")?;
    let ctx = seeded(&fs, std::slice::from_ref(&fact))?;
    let events = EventLog::new(&fs, ROOT, EventLog::DEFAULT_MAX_BYTES);
    let config = Config::defaults();
    let (_index, graph) = built(std::slice::from_ref(&fact))?;
    let thresholds = DedupThresholds::default();
    fs.insert_at("/p/.knudge/.locks/a.lock", b"{}".to_vec(), NOW - 60_000);
    let world = World {
        fs: &fs,
        ctx: &ctx,
        events: &events,
        config: &config,
        graph: &graph,
        thresholds: &thresholds,
    };

    let _fixed = doctor_fix(&world.input())?;
    assert!(!fs.exists(Path::new("/p/.knudge/.locks/a.lock")));
    Ok(())
}

#[test]
fn broken_anchor_is_removed() -> Result<()> {
    let fs = MemFs::new();
    let fact = anchored("âncora quebrada", &["src/sumiu.rs"], "")?;
    let id = fact.id()?.to_string();
    let ctx = seeded(&fs, std::slice::from_ref(&fact))?;
    let events = EventLog::new(&fs, ROOT, EventLog::DEFAULT_MAX_BYTES);
    let config = Config::defaults();
    let (_index, graph) = built(std::slice::from_ref(&fact))?;
    let thresholds = DedupThresholds::default();
    let world = World {
        fs: &fs,
        ctx: &ctx,
        events: &events,
        config: &config,
        graph: &graph,
        thresholds: &thresholds,
    };

    let _fixed = doctor_fix(&world.input())?;
    let stored = ctx.store().read(&id)?;
    assert!(stored.frontmatter.string_list("anchors")?.is_empty());
    Ok(())
}

#[test]
fn embeddings_check_reports_sizes() -> Result<()> {
    let fs = MemFs::new();
    let fact = note(NoteType::Fact, "vetor", "")?;
    let ctx = seeded(&fs, std::slice::from_ref(&fact))?;
    let events = EventLog::new(&fs, ROOT, EventLog::DEFAULT_MAX_BYTES);
    let config = Config::defaults();
    let (_index, graph) = built(std::slice::from_ref(&fact))?;
    let thresholds = DedupThresholds::default();
    let world = World {
        fs: &fs,
        ctx: &ctx,
        events: &events,
        config: &config,
        graph: &graph,
        thresholds: &thresholds,
    };
    fs.insert("/p/.knudge/.idx/embeddings.jsonl", "{\"meta\":{}}\n");
    let report = doctor(&world.input())?;
    let check = report.check(CheckId::Embeddings);
    assert!(check.is_some());
    assert_eq!(check.map(|c| c.ok), Some(true));
    Ok(())
}

#[test]
fn program_anchor_reports_epic_without_program() -> Result<()> {
    let fs = MemFs::new();
    let mut draft = Draft::new(NoteType::Epic, "Épico solto");
    draft.scope = Some(Scope::Epic);
    let epic = draft.to_note(NOW)?;
    let ctx = seeded(&fs, std::slice::from_ref(&epic))?;
    let events = EventLog::new(&fs, ROOT, EventLog::DEFAULT_MAX_BYTES);
    let config = Config::defaults();
    let (_index, graph) = built(std::slice::from_ref(&epic))?;
    let thresholds = DedupThresholds::default();
    let world = World {
        fs: &fs,
        ctx: &ctx,
        events: &events,
        config: &config,
        graph: &graph,
        thresholds: &thresholds,
    };
    let report = doctor(&world.input())?;
    assert_eq!(
        report.check(CheckId::ProgramAnchor).map(|c| c.ok),
        Some(false)
    );
    Ok(())
}

#[test]
fn program_anchor_reports_orphan_program() -> Result<()> {
    let fs = MemFs::new();
    let fact = note(NoteType::Fact, "só conhecimento", "")?;
    let ctx = seeded(&fs, std::slice::from_ref(&fact))?;
    let events = EventLog::new(&fs, ROOT, EventLog::DEFAULT_MAX_BYTES);
    let config = Config::defaults();
    let (_index, graph) = built(std::slice::from_ref(&fact))?;
    let thresholds = DedupThresholds::default();
    let world = World {
        fs: &fs,
        ctx: &ctx,
        events: &events,
        config: &config,
        graph: &graph,
        thresholds: &thresholds,
    };
    fs.create_dir_all(Path::new("/p/plan"))?;
    fs.write_atomic(Path::new("/p/plan/orfao.md"), b"# orfao")?;
    let report = doctor(&world.input())?;
    assert_eq!(
        report.check(CheckId::ProgramAnchor).map(|c| c.ok),
        Some(false)
    );
    Ok(())
}
