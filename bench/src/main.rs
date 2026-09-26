//! Bancada de benchmark do knudge (sem dependências externas).
//!
//! ```sh
//! make bench                 # micro + e2e, corpora 200 e 1000
//! cargo run --release --manifest-path bench/Cargo.toml -- micro --sizes 1000
//! cargo run --release --manifest-path bench/Cargo.toml -- e2e --kd target/release/kd
//! ```

#![allow(dead_code)]

mod e2e;
mod fixture;
mod harness;
mod micro;

use std::path::PathBuf;

use harness::Harness;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut mode = String::from("all");
    let mut sizes = String::from("200,1000");
    let mut samples = 8_usize;
    let mut kd = std::env::var("KD_BIN").unwrap_or_else(|_| "target/release/kd".to_string());
    let mut no_idle = false;
    let mut json_out: Option<PathBuf> = None;
    let mut md_out: Option<PathBuf> = None;

    let mut index = 0;
    while index < args.len() {
        let arg = args[index].as_str();
        let next = |index: &mut usize| -> String {
            *index += 1;
            args.get(*index).cloned().unwrap_or_default()
        };
        match arg {
            "micro" | "e2e" | "all" => mode = arg.to_string(),
            "--sizes" => sizes = next(&mut index),
            "--samples" => samples = next(&mut index).parse().unwrap_or(samples),
            "--kd" => kd = next(&mut index),
            "--no-idle" => no_idle = true,
            "--json" => json_out = Some(PathBuf::from(next(&mut index))),
            "--out" => md_out = Some(PathBuf::from(next(&mut index))),
            "--quick" => {
                sizes = "200".to_string();
                samples = 5;
            }
            "--help" | "-h" => {
                println!("uso: knudge-bench [micro|e2e|all] [--sizes 200,1000] [--samples 8] [--kd PATH] [--json out.json] [--out report.md] [--quick]");
                return;
            }
            other => {
                eprintln!("argumento desconhecido: {other}");
                std::process::exit(2);
            }
        }
        index += 1;
    }

    let sizes: Vec<usize> = sizes
        .split(',')
        .filter_map(|part| part.trim().parse().ok())
        .collect();
    let sizes = if sizes.is_empty() { vec![200, 1000] } else { sizes };

    let mut harness = Harness::new();
    if mode == "micro" || mode == "all" {
        eprintln!("== micromb (sizes={sizes:?}) ==");
        micro::run(&mut harness, &sizes);
    }
    if mode == "e2e" || mode == "all" {
        let kd = PathBuf::from(&kd);
        let absolute = if kd.is_absolute() {
            kd
        } else {
            std::env::current_dir().unwrap_or_default().join(kd)
        };
        eprintln!("== ponta-a-ponta (kd={}, sizes={sizes:?}) ==", absolute.display());
        e2e::run(&mut harness, &absolute, &sizes, samples, no_idle);
    }

    let markdown = harness.render_markdown();
    print!("{markdown}");
    if let Some(path) = md_out {
        let _ = std::fs::write(path, &markdown);
    }
    if let Some(path) = json_out {
        let _ = std::fs::write(path, harness.render_json());
    }
}
