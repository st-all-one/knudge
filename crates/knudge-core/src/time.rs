//! Tempo determinístico: `Timestamp` em milissegundos UTC (D07).
//!
//! Formatação e parsing são implementados aqui (algoritmos civis de Hinnant) para não depender
//! de crate de data e manter round-trip byte-exato. O relógio real é acessado só pela porta
//! [`crate::ports::Clock`].

use std::fmt;
use std::str::FromStr;

use crate::{Error, Result};

/// Instante em milissegundos desde `1970-01-01T00:00:00Z`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Timestamp(i64);

impl Timestamp {
    /// Época Unix (`1970-01-01T00:00:00.000Z`).
    pub const EPOCH: Self = Self(0);

    /// Constrói a partir de milissegundos desde a época.
    #[must_use]
    pub const fn from_millis(ms: i64) -> Self {
        Self(ms)
    }

    /// Milissegundos desde a época.
    #[must_use]
    pub const fn as_millis(self) -> i64 {
        self.0
    }

    /// Soma milissegundos saturando nos limites de `i64`.
    #[must_use]
    pub const fn saturating_add_millis(self, ms: i64) -> Self {
        Self(self.0.saturating_add(ms))
    }

    /// Representação canônica: `YYYY-MM-DDTHH:MM:SS.mmmZ` (UTC).
    #[must_use]
    pub fn to_rfc3339(self) -> String {
        format_utc(self.0)
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_rfc3339())
    }
}

impl FromStr for Timestamp {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        parse_utc(s)
    }
}

#[allow(
    clippy::arithmetic_side_effects,
    reason = "aritmética civil com domínio conhecido e entradas limitadas a 4 dígitos de ano"
)]
fn format_utc(ms: i64) -> String {
    let secs = ms.div_euclid(1000);
    let millis = ms.rem_euclid(1000);
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = sod / 3600;
    let minute = (sod % 3600) / 60;
    let second = sod % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z")
}

/// Converte dias desde a época em `(ano, mês, dia)` (Hinnant).
#[allow(
    clippy::arithmetic_side_effects,
    reason = "algoritmo civil com entradas limitadas"
)]
const fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Converte `(ano, mês, dia)` em dias desde a época (Hinnant).
#[allow(
    clippy::arithmetic_side_effects,
    reason = "algoritmo civil com entradas limitadas"
)]
const fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[allow(
    clippy::arithmetic_side_effects,
    reason = "composição de tempo com entradas validadas"
)]
fn parse_utc(s: &str) -> Result<Timestamp> {
    let invalid = || {
        Error::invalid_input(format!(
            "timestamp inválido: {s:?} (esperado YYYY-MM-DDTHH:MM:SS.mmmZ)"
        ))
    };

    let (date, rest) = s.split_once('T').ok_or_else(invalid)?;
    let rest = rest.strip_suffix('Z').ok_or_else(invalid)?;
    let (time, frac) = match rest.split_once('.') {
        Some((t, f)) => (t, Some(f)),
        None => (rest, None),
    };

    let mut dp = date.split('-');
    let year = next_num(&mut dp, invalid)?;
    let month = next_num(&mut dp, invalid)?;
    let day = next_num(&mut dp, invalid)?;
    if dp.next().is_some() {
        return Err(invalid());
    }

    let mut tp = time.split(':');
    let hour = next_num(&mut tp, invalid)?;
    let minute = next_num(&mut tp, invalid)?;
    let second = next_num(&mut tp, invalid)?;
    if tp.next().is_some() {
        return Err(invalid());
    }

    let millis = match frac {
        Some(f) => parse_fraction(f, invalid)?,
        None => 0,
    };

    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || day > days_in_month(year, month)
        || !(0..=23).contains(&hour)
        || !(0..=59).contains(&minute)
        || !(0..=59).contains(&second)
    {
        return Err(invalid());
    }

    let days = days_from_civil(year, month, day);
    let secs = days * 86_400 + hour * 3600 + minute * 60 + second;
    Ok(Timestamp(secs * 1000 + millis))
}

#[allow(
    clippy::arithmetic_side_effects,
    reason = "anos de 4 dígitos; sem overflow"
)]
const fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[allow(
    clippy::arithmetic_side_effects,
    reason = "anos de 4 dígitos; sem overflow"
)]
const fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(year) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Lê o próximo componente numérico de um iterador de partes.
fn next_num<'a, I>(parts: &mut I, invalid: impl Fn() -> Error) -> Result<i64>
where
    I: Iterator<Item = &'a str>,
{
    parts
        .next()
        .and_then(|p| p.parse::<i64>().ok())
        .ok_or_else(invalid)
}

/// Interpreta a fração de segundo, normalizando para milissegundos.
#[allow(clippy::arithmetic_side_effects, reason = "fração de até 9 dígitos")]
fn parse_fraction(frac: &str, invalid: impl Fn() -> Error) -> Result<i64> {
    if frac.is_empty() || frac.len() > 9 || !frac.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid());
    }
    let mut value: i64 = frac.parse().map_err(|_| invalid())?;
    let mut digits = i64::try_from(frac.len()).map_err(|_| invalid())?;
    while digits < 3 {
        value *= 10;
        digits += 1;
    }
    while digits > 3 {
        value /= 10;
        digits -= 1;
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::ProptestConfig;

    #[test]
    fn epoch_formats_and_parses() {
        let t = Timestamp::EPOCH;
        assert_eq!(t.to_rfc3339(), "1970-01-01T00:00:00.000Z");
        assert_eq!(
            "1970-01-01T00:00:00.000Z".parse::<Timestamp>().ok(),
            Some(t)
        );
    }

    #[test]
    fn known_instant_round_trips() {
        let t = Timestamp::from_millis(1_700_000_000_123);
        assert_eq!(t.to_rfc3339(), "2023-11-14T22:13:20.123Z");
        assert_eq!(t.to_rfc3339().parse::<Timestamp>().ok(), Some(t));
    }

    #[test]
    fn lexicographic_order_matches_chronological() {
        let a = Timestamp::from_millis(1_000);
        let b = Timestamp::from_millis(2_000_000);
        assert!(a < b);
        assert!(a.to_rfc3339() < b.to_rfc3339());
    }

    #[test]
    fn accepts_short_fraction_and_missing_fraction() {
        assert_eq!(
            "2023-11-14T22:13:20.1Z".parse::<Timestamp>().ok(),
            Some(Timestamp::from_millis(1_700_000_000_100))
        );
        assert_eq!(
            "2023-11-14T22:13:20Z".parse::<Timestamp>().ok(),
            Some(Timestamp::from_millis(1_700_000_000_000))
        );
    }

    #[test]
    fn rejects_invalid_input() {
        for bad in [
            "2023-11-14 22:13:20Z",
            "2023-13-01T00:00:00Z",
            "2023-02-30T00:00:00Z",
            "2023-11-14T24:00:00Z",
            "2023-11-14T22:13:20+00:00",
        ] {
            assert!(bad.parse::<Timestamp>().is_err(), "deveria rejeitar {bad}");
        }
    }

    proptest::proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        fn format_parse_round_trip(ms in -62_135_596_800_000_i64..=253_402_300_799_999) {
            let t = Timestamp::from_millis(ms);
            let text = t.to_rfc3339();
            proptest::prop_assert_eq!(text.parse::<Timestamp>().ok(), Some(t));
        }
    }
}
