//! Confiança bayesiana Beta-Bernoulli (D189, E19-T01).
//!
//! `outcomes` são ensaios de Bernoulli: `success` soma um sucesso, `partial` soma 0,5 a cada
//! lado, `failure`/`abandoned` somam uma falha. O posterior `Beta(1+s, 1+f)` dá a **média**
//! (usada no canal `stars`) e o **limite inferior** do intervalo de credibilidade de 95 %
//! (Wilson) — a confiança **conservadora**: uma nota com 1 sucesso (≈0,21) não empata com uma
//! com 20 (≈0,87). Puro, determinístico, `O(1)` e sem dependência.

/// Sucesso anterior (uniforme) — posterior `Beta(1+s, 1+f)`.
pub const PRIOR_SUCCESS: f64 = 1.0;

/// Falha anterior (uniforme) — posterior `Beta(1+s, 1+f)`.
pub const PRIOR_FAILURE: f64 = 1.0;

/// `z` do intervalo de 95 % (bicaudal).
pub const Z_95: f64 = 1.96;

/// Média posterior `(α+s)/(α+β+n)` em `[0,1]`; `0` sem evidência.
#[must_use]
pub fn posterior_mean(successes: f64, failures: f64) -> f64 {
    let successes = successes.max(0.0);
    let failures = failures.max(0.0);
    if successes == 0.0 && failures == 0.0 {
        return 0.0;
    }
    let alpha = PRIOR_SUCCESS + successes;
    let beta = PRIOR_FAILURE + failures;
    clamp01(alpha / (alpha + beta))
}

/// Limite inferior do intervalo de credibilidade de 95 % (Wilson) em `[0,1]`; `0` sem
/// evidência.
#[must_use]
pub fn lower_bound(successes: f64, failures: f64) -> f64 {
    let successes = successes.max(0.0);
    let failures = failures.max(0.0);
    let trials = successes + failures;
    if trials <= 0.0 {
        return 0.0;
    }
    let proportion = successes / trials;
    let z2 = Z_95 * Z_95;
    let inv_trials = 1.0 / trials;
    let z2_over_n = z2 * inv_trials;
    let denom = 1.0 + z2_over_n;
    let center = 0.5_f64.mul_add(z2_over_n, proportion);
    let variance = inv_trials * 0.25_f64.mul_add(z2_over_n, proportion * (1.0 - proportion));
    let margin = Z_95 * variance.sqrt();
    clamp01((center - margin) / denom)
}

fn clamp01(value: f64) -> f64 {
    if value.is_nan() {
        return 0.0;
    }
    value.clamp(0.0, 1.0)
}
