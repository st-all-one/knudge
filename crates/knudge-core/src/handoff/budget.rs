//! Orçamento de tokens sem tokenizer (D40/D82).
//!
//! `estimateTokens = ceil(len/4)` sobre o número de caracteres. O orçamento é aplicado item a
//! item: o último item é truncado para caber e sobras abaixo de [`MIN_TAIL`] são ignoradas.
//! [`apply_into`] escreve direto no destino (o CLI passa o `BufWriter` do stdout), sem montar
//! uma saída gigante em memória (R04/R15).

use std::fmt::Write;

/// Orçamento padrão de tokens (`kd rewind --budget`).
pub const DEFAULT_BUDGET: usize = 4000;

/// Sobra mínima para valer a pena truncar o último item.
pub const MIN_TAIL: usize = 100;

/// Estimativa determinística de tokens (`ceil(chars/4)`).
#[must_use]
pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

/// Resumo da aplicação de orçamento.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BudgetSummary {
    /// `true` se o último item foi truncado.
    pub truncated: bool,
    /// Itens descartados por falta de orçamento.
    pub dropped: usize,
}

/// Resultado da aplicação de orçamento (com o texto materializado).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Budgeted {
    /// Texto final (itens separados por `\n`).
    pub text: String,
    /// `true` se o último item foi truncado.
    pub truncated: bool,
    /// Itens descartados por falta de orçamento.
    pub dropped: usize,
}

/// Aplica o orçamento a linhas já renderizadas.
#[must_use]
pub fn apply(lines: &[String], budget: usize) -> Budgeted {
    let mut text = String::new();
    let summary = apply_into(lines, budget, &mut text);
    Budgeted {
        text,
        truncated: summary.truncated,
        dropped: summary.dropped,
    }
}

/// Aplica o orçamento escrevendo direto no destino (streaming).
pub fn apply_into(lines: &[String], budget: usize, out: &mut impl Write) -> BudgetSummary {
    let mut summary = BudgetSummary::default();
    let mut used = 0_usize;
    let mut wrote = false;
    for (index, line) in lines.iter().enumerate() {
        let cost = estimate_tokens(line);
        if used.saturating_add(cost) <= budget {
            push_line(out, &mut wrote, line);
            used = used.saturating_add(cost);
            continue;
        }
        let remaining = budget.saturating_sub(used);
        if remaining >= MIN_TAIL {
            let cut = truncate_chars(line, remaining.saturating_mul(4));
            push_line(out, &mut wrote, &format!("{cut}…"));
            summary.truncated = true;
        }
        summary.dropped = lines.len().saturating_sub(index);
        break;
    }
    summary
}

fn push_line(out: &mut impl Write, wrote: &mut bool, line: &str) {
    if *wrote {
        let _ignored = out.write_char('\n');
    }
    *wrote = true;
    let _ignored = out.write_str(line);
}

fn truncate_chars(text: &str, max: usize) -> String {
    text.chars().take(max).collect()
}
