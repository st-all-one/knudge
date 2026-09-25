//! Check de corpo/lastro do doctor (D162).

use crate::Result;
use crate::config::Config;
use crate::health::{CheckId, DoctorCheck, doctor};
use crate::ports::fakes::MemFs;
use crate::schema::NoteType;
use crate::store::EventLog;
use crate::write::dedup::DedupThresholds;

use super::doctor::World;
use super::{ROOT, anchored, built, note, seeded};

#[test]
fn body_check_is_advisory_and_counts_missing_body() -> Result<()> {
    let fs = MemFs::new();
    let bare = note(NoteType::Fact, "fato sem corpo nem lastro", "")?;
    let anchored_note = anchored("fato sem corpo mas ancorado", &["src/x.rs"], "")?;
    let with_body = note(NoteType::Decision, "decisão com corpo", "Por quê: x.")?;
    let notes = vec![bare, anchored_note, with_body];
    let ctx = seeded(&fs, &notes)?;
    let events = EventLog::new(&fs, ROOT, EventLog::DEFAULT_MAX_BYTES);
    let config = Config::defaults();
    let (_index, graph) = built(&notes)?;
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
    let body = report.check(CheckId::Body);
    assert_eq!(body.map(|check| check.ok), Some(false));
    assert!(
        body.is_some_and(DoctorCheck::is_warning),
        "check do corpo deve ser advisório"
    );
    let detail = body.map(|check| check.detail.as_str()).unwrap_or_default();
    assert!(detail.contains("2 sem corpo"), "detalhe: {detail}");
    assert!(detail.contains("1 sem lastro"), "detalhe: {detail}");
    Ok(())
}
