//! Harness de medição determinístico, sem dependências externas.
//!
//! Cada amostra (`sample`) executa `ops_per_sample` operações; guardamos o tempo
//! por operação. As amostras são ordenadas para extrair mínimo/mediana/p95/máximo.

use std::fmt::Write as _;
use std::time::{Duration, Instant};

/// Estatísticas de uma célula da bancada.
pub struct Cell {
    /// Grupo (ex.: `micro`, `e2e/<size>`).
    pub group: String,
    /// Nome da operação.
    pub name: String,
    /// Operações por amostra.
    pub ops_per_sample: u64,
    /// Amostras (ns por operação), ordenadas.
    per_op_ns: Vec<f64>,
}

impl Cell {
    /// Mínimo (ns/op).
    #[must_use]
    pub fn min_ns(&self) -> f64 {
        self.per_op_ns.first().copied().unwrap_or(0.0)
    }
    /// Mediana (ns/op).
    #[must_use]
    pub fn median_ns(&self) -> f64 {
        percentile(&self.per_op_ns, 0.5)
    }
    /// Média (ns/op).
    #[must_use]
    pub fn mean_ns(&self) -> f64 {
        if self.per_op_ns.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.per_op_ns.iter().sum();
        sum / self.per_op_ns.len() as f64
    }
    /// p95 (ns/op).
    #[must_use]
    pub fn p95_ns(&self) -> f64 {
        percentile(&self.per_op_ns, 0.95)
    }
    /// Máximo (ns/op).
    #[must_use]
    pub fn max_ns(&self) -> f64 {
        self.per_op_ns.last().copied().unwrap_or(0.0)
    }
    /// Desvio padrão amostral (ns/op).
    #[must_use]
    pub fn stddev_ns(&self) -> f64 {
        if self.per_op_ns.len() < 2 {
            return 0.0;
        }
        let mean = self.mean_ns();
        let var: f64 = self
            .per_op_ns
            .iter()
            .map(|sample| (sample - mean).powi(2))
            .sum::<f64>()
            / (self.per_op_ns.len() - 1) as f64;
        var.sqrt()
    }
    /// Total acumulado em segundos.
    #[must_use]
    pub fn total_s(&self) -> f64 {
        self.per_op_ns.iter().sum::<f64>() * self.ops_per_sample as f64 / 1e9
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted.get(idx).copied().unwrap_or(0.0)
}

/// Resultado agregado da bancada.
#[derive(Default)]
pub struct Harness {
    cells: Vec<Cell>,
}

impl Harness {
    /// Cria uma bancada vazia.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Mede `samples` amostras de `ops_per_sample` execuções de `f`.
    pub fn measure<F: FnMut()>(&mut self, group: &str, name: &str, samples: usize, ops_per_sample: u64, mut f: F) {
        // Aquecimento: deixa caches/allocator/jit-de-página assentarem.
        for _ in 0..3 {
            for _ in 0..ops_per_sample {
                f();
            }
        }
        let mut per_op_ns = Vec::with_capacity(samples);
        for _ in 0..samples {
            let start = Instant::now();
            for _ in 0..ops_per_sample {
                f();
            }
            let elapsed = start.elapsed();
            per_op_ns.push(elapsed.as_nanos() as f64 / ops_per_sample as f64);
        }
        per_op_ns.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        self.cells.push(Cell {
            group: group.to_string(),
            name: name.to_string(),
            ops_per_sample,
            per_op_ns,
        });
    }

    /// Mede uma ação que já devolve a própria `Duration` (ex.: subprocesso).
    pub fn record(&mut self, group: &str, name: &str, duration: Duration) {
        let ns = duration.as_nanos() as f64;
        self.cells.push(Cell {
            group: group.to_string(),
            name: name.to_string(),
            ops_per_sample: 1,
            per_op_ns: vec![ns],
        });
    }

    /// Mede um comando repetido, deixando o harness cuidar do loop.
    pub fn measure_cmd<F: FnMut() -> Duration>(
        &mut self,
        group: &str,
        name: &str,
        samples: usize,
        mut f: F,
    ) {
        let mut per_op_ns = Vec::with_capacity(samples);
        // Uma execução de aquecimento.
        let _ = f();
        for _ in 0..samples {
            let duration = f();
            per_op_ns.push(duration.as_nanos() as f64);
        }
        per_op_ns.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        self.cells.push(Cell {
            group: group.to_string(),
            name: name.to_string(),
            ops_per_sample: 1,
            per_op_ns,
        });
    }

    /// Células coletadas.
    #[must_use]
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Renderiza uma tabela legível em Markdown.
    #[must_use]
    pub fn render_markdown(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(
            out,
            "| grupo | operação | n | min | mediana | p95 | máx | desvio |"
        );
        let _ = writeln!(out, "|---|---|---:|---:|---:|---:|---:|---:|");
        for cell in &self.cells {
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} | {} | {} | {} |",
                cell.group,
                cell.name,
                cell.per_op_ns.len(),
                fmt_ns(cell.min_ns()),
                fmt_ns(cell.median_ns()),
                fmt_ns(cell.p95_ns()),
                fmt_ns(cell.max_ns()),
                fmt_ns(cell.stddev_ns()),
            );
        }
        out
    }

    /// Emite os resultados como JSON (contrato de máquina da bancada).
    #[must_use]
    pub fn render_json(&self) -> String {
        let mut out = String::from("[\n");
        for (index, cell) in self.cells.iter().enumerate() {
            let comma = if index + 1 == self.cells.len() { "" } else { "," };
            let _ = writeln!(
                out,
                "  {{\"group\":{},\"name\":{},\"n\":{},\"min_ns\":{:.1},\"median_ns\":{:.1},\"mean_ns\":{:.1},\"p95_ns\":{:.1},\"max_ns\":{:.1},\"stddev_ns\":{:.1}}}{comma}",
                json_str(&cell.group),
                json_str(&cell.name),
                cell.per_op_ns.len(),
                cell.min_ns(),
                cell.median_ns(),
                cell.mean_ns(),
                cell.p95_ns(),
                cell.max_ns(),
                cell.stddev_ns(),
            );
        }
        out.push_str("]\n");
        out
    }
}

/// Formata nanossegundos numa unidade legível.
#[must_use]
pub fn fmt_ns(ns: f64) -> String {
    if ns >= 1e9 {
        format!("{:.3} s", ns / 1e9)
    } else if ns >= 1e6 {
        format!("{:.3} ms", ns / 1e6)
    } else if ns >= 1e3 {
        format!("{:.2} µs", ns / 1e3)
    } else {
        format!("{ns:.0} ns")
    }
}

fn json_str(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
