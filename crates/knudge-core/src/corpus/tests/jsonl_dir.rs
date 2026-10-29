//! Testes do diretório de shards NDJSON (D220/D223) e da identidade de agente (D221).

use std::path::Path;

use crate::Error;
use crate::Result;
use crate::corpus::{Source, shard_file_name};
use crate::ports::Fs;
use crate::ports::fakes::MemFs;
use crate::schema::{AgentRole, NoteType, Provenance, provenance};
use crate::store::{Note, Store};

use super::{NOW, jsonl_of, note, note_at};

/// Nota com identidade de agente explícita (D221).
fn identified(mut note: Note, agent_id: &str, role: AgentRole) -> Result<Note> {
    let provenance = Provenance {
        agent_id: Some(agent_id.to_string()),
        agent_role: Some(role),
        ..Provenance::default()
    };
    note.frontmatter.set("provenance", provenance.to_value())?;
    Ok(note)
}

#[test]
fn jsonl_dir_merges_shards_deterministically() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, "/p/.knudge");
    let first = note(NoteType::Fact, "o shard a tem esta nota")?;
    let second = note(NoteType::Fact, "o shard b tem esta nota")?;
    fs.create_dir_all(Path::new("/p/ai/memory"))?;
    fs.write_atomic(
        Path::new("/p/ai/memory/a.jsonl"),
        jsonl_of(&[first])?.as_bytes(),
    )?;
    fs.write_atomic(
        Path::new("/p/ai/memory/b.jsonl"),
        jsonl_of(&[second])?.as_bytes(),
    )?;

    let (loaded, warnings) = Source::jsonl_dir("/p/ai/memory").load_notes(&store)?;
    assert!(warnings.is_empty());
    assert_eq!(loaded.len(), 2);
    let ids: Vec<&str> = loaded.iter().map(|item| item.id().unwrap_or("")).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted, "merge determinístico por id");
    // Ler o mesmo diretório duas vezes dá o mesmo corpus (sem manifesto mutável).
    let again = Source::jsonl_dir("/p/ai/memory").load_notes(&store)?.0;
    assert_eq!(loaded, again);
    Ok(())
}

#[test]
fn jsonl_dir_same_id_conflict_uses_created_at_then_agent_id() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, "/p/.knudge");
    fs.create_dir_all(Path::new("/p/ai/memory"))?;
    let mut older = note(NoteType::Fact, "conflito entre shards")?;
    older.body = "versao antiga".to_string();
    let older = identified(older, "a", AgentRole::Main)?;
    let mut newer = note_at(NoteType::Fact, "conflito entre shards", 1_700_000_001_000)?;
    newer.body = "versao nova".to_string();
    let newer = identified(newer, "b", AgentRole::Subagent)?;
    fs.write_atomic(
        Path::new("/p/ai/memory/a.jsonl"),
        jsonl_of(&[older])?.as_bytes(),
    )?;
    fs.write_atomic(
        Path::new("/p/ai/memory/b.jsonl"),
        jsonl_of(&[newer])?.as_bytes(),
    )?;

    let (loaded, _warnings) = Source::jsonl_dir("/p/ai/memory").load_notes(&store)?;
    assert_eq!(loaded.len(), 1);
    assert_eq!(
        loaded.first().map(|note| note.body.as_str()),
        Some("versao nova"),
        "created_at maior vence (D223)"
    );

    // Empate de `created_at`: o maior `agent_id` decide.
    let tie_a = identified(
        note_at(NoteType::Fact, "empate de tempo", NOW)?,
        "a",
        AgentRole::Main,
    )?;
    let tie_b = identified(
        note_at(NoteType::Fact, "empate de tempo", NOW)?,
        "b",
        AgentRole::Subagent,
    )?;
    fs.write_atomic(
        Path::new("/p/ai/memory/a.jsonl"),
        jsonl_of(&[tie_a])?.as_bytes(),
    )?;
    fs.write_atomic(
        Path::new("/p/ai/memory/b.jsonl"),
        jsonl_of(&[tie_b])?.as_bytes(),
    )?;
    let (tied, _warnings) = Source::jsonl_dir("/p/ai/memory").load_notes(&store)?;
    let prov = provenance(
        &tied
            .first()
            .ok_or_else(|| Error::internal("nota em falta no conflito"))?
            .frontmatter,
    )?;
    assert_eq!(prov.agent_id.as_deref(), Some("b"));
    Ok(())
}

#[test]
fn jsonl_dir_ignores_malformed_line_with_shard_warning() -> Result<()> {
    let fs = MemFs::new();
    let store = Store::new(&fs, "/p/.knudge");
    let good = note(NoteType::Fact, "linha boa")?;
    fs.create_dir_all(Path::new("/p/ai/memory"))?;
    fs.write_atomic(
        Path::new("/p/ai/memory/a.jsonl"),
        jsonl_of(&[good])?.as_bytes(),
    )?;
    fs.write_atomic(Path::new("/p/ai/memory/b.jsonl"), b"isto nao e json\n")?;
    let (loaded, warnings) = Source::jsonl_dir("/p/ai/memory").load_notes(&store)?;
    assert_eq!(loaded.len(), 1);
    assert_eq!(warnings.len(), 1);
    assert!(
        warnings
            .first()
            .is_some_and(|text| text.contains("b.jsonl") && text.contains("linha 1 ignorada")),
        "{warnings:?}"
    );
    Ok(())
}

#[test]
fn jsonl_dir_reads_while_writer_appends() -> Result<()> {
    let fs = MemFs::new();
    fs.create_dir_all(Path::new("/p/ai/memory"))?;
    let agents = ["a", "b", "c", "d"];
    let per_agent = 25_usize;
    let expected = agents.len().saturating_mul(per_agent);
    std::thread::scope(|scope| {
        for agent in agents {
            let fs = &fs;
            scope.spawn(move || {
                let path = Path::new("/p/ai/memory").join(format!("{agent}.jsonl"));
                for index in 0..per_agent {
                    let Ok(note) = note_at(
                        NoteType::Fact,
                        &format!("nota {agent} numero {index}"),
                        NOW.saturating_add(i64::try_from(index).unwrap_or(0)),
                    ) else {
                        return;
                    };
                    let Ok(line) = note.to_jsonl() else {
                        return;
                    };
                    let mut bytes = line.into_bytes();
                    bytes.push(b'\n');
                    let _ignored = fs.append(&path, &bytes);
                }
            });
        }
        let fs = &fs;
        scope.spawn(move || {
            let store = Store::new(fs, "/p/.knudge");
            let mut seen = 0_usize;
            for _ in 0..50 {
                let Ok((loaded, _)) = Source::jsonl_dir("/p/ai/memory").load_notes(&store) else {
                    continue;
                };
                assert!(
                    loaded.len() >= seen,
                    "a leitura nunca perde notas já vistas"
                );
                seen = loaded.len();
            }
        });
    });
    let store = Store::new(&fs, "/p/.knudge");
    let (loaded, warnings) = Source::jsonl_dir("/p/ai/memory").load_notes(&store)?;
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(loaded.len(), expected);
    Ok(())
}

#[test]
fn shard_file_name_is_safe_and_deterministic() {
    assert_eq!(shard_file_name("omp-01j"), "omp-01j.jsonl");
    assert_eq!(shard_file_name("omp:01j"), "omp-01j-04e12bf5.jsonl");
    assert_eq!(shard_file_name("omp:01j"), shard_file_name("omp:01j"));
    for unsafe_id in ["..", &"a".repeat(200)] {
        let name = shard_file_name(unsafe_id);
        assert_eq!(
            Path::new(&name).extension().and_then(|ext| ext.to_str()),
            Some("jsonl")
        );
        assert!(!name.contains(".."), "{name}");
    }
}

#[test]
fn source_at_detects_file_and_directory() -> Result<()> {
    let fs = MemFs::new();
    fs.create_dir_all(Path::new("/p/ai"))?;
    fs.write_atomic(Path::new("/p/one.jsonl"), b"")?;
    assert_eq!(Source::at(&fs, "/p/ai"), Source::jsonl_dir("/p/ai"));
    assert_eq!(
        Source::at(&fs, "/p/one.jsonl"),
        Source::jsonl("/p/one.jsonl")
    );
    assert_eq!(Source::at(&fs, "/p/missing"), Source::jsonl("/p/missing"));
    Ok(())
}
