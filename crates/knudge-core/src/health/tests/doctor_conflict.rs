//! Nota com marcadores de conflito do git reportada pelo doctor (D153).

use crate::Result;
use crate::config::Config;
use crate::health::{CheckId, doctor};
use crate::ports::fakes::MemFs;
use crate::schema::NoteType;
use crate::store::EventLog;
use crate::write::dedup::DedupThresholds;

use super::doctor::World;
use super::{ROOT, built, note, seeded};

#[test]
fn conflict_note_is_reported_by_schema_check() -> Result<()> {
    let fs = MemFs::new();
    let fact = note(NoteType::Fact, "válida", "")?;
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

    // Arquivo com marcadores de conflito do git (mesma statement, corpos divergentes — D153).
    fs.insert(
        format!("{ROOT}/notas/fact/fact_00000009.md"),
        b"<<<<<<< HEAD\n---\nid: fact_00000009\n---\nA\n=======\nB\n>>>>>>> other\n".to_vec(),
    );

    let report = doctor(&world.input())?;
    assert_eq!(report.check(CheckId::Schema).map(|c| c.ok), Some(false));
    Ok(())
}
