//! Portão de evidência (D156): veredito de um gate de proposta.
//!
//! O núcleo é **puro**: só decide se um veredito (`passed` + delta) satisfaz o `min_delta`.
//! A execução do comando externo (stdin JSON → stdout JSON) fica na borda, via `HookRunner`
//! (sem shell — R12).

/// Veredito de um gate sobre uma transformação `before → after`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GateOutcome {
    /// O gate aprovou a transformação.
    pub passed: bool,
    /// Placar antes da transformação.
    pub score_before: f64,
    /// Placar depois da transformação.
    pub score_after: f64,
}

impl GateOutcome {
    /// Ganho da transformação (`score_after - score_before`).
    #[allow(
        clippy::arithmetic_side_effects,
        reason = "subtração de floats finitos (placares do gate), sem overflow"
    )]
    #[must_use]
    pub fn delta(&self) -> f64 {
        self.score_after - self.score_before
    }
}

/// `true` se o veredito aprova **e** o ganho atinge `min_delta` (puro).
#[must_use]
pub fn accept(outcome: &GateOutcome, min_delta: f64) -> bool {
    outcome.passed && outcome.delta() >= min_delta
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(before: f64, after: f64) -> GateOutcome {
        GateOutcome {
            passed: true,
            score_before: before,
            score_after: after,
        }
    }

    fn fail(before: f64, after: f64) -> GateOutcome {
        GateOutcome {
            passed: false,
            score_before: before,
            score_after: after,
        }
    }

    #[test]
    fn accepts_when_passed_and_delta_reached() {
        assert!(accept(&ok(0.2, 0.5), 0.1));
        assert!(!accept(&ok(0.2, 0.25), 0.1));
        assert!(!accept(&fail(0.2, 0.9), 0.0));
    }

    #[test]
    fn delta_can_be_negative() {
        assert!(ok(0.9, 0.4).delta() < 0.0);
    }
}
