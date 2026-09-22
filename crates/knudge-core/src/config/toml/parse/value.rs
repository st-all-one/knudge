//! Análise de valores TOML (strings, números, booleanos e listas).

use crate::Result;
use crate::config::value::ConfigValue;

use super::Parser;

impl Parser<'_> {
    pub(super) fn value(&mut self) -> Result<ConfigValue> {
        match self.peek() {
            Some(b'"') => Ok(ConfigValue::String(self.basic_string()?)),
            Some(b'\'') => Ok(ConfigValue::String(self.literal_string()?)),
            Some(b'[') => self.array(),
            Some(b't') => self.literal("true", ConfigValue::Bool(true)),
            Some(b'f') => self.literal("false", ConfigValue::Bool(false)),
            Some(b'+' | b'-' | b'0'..=b'9') => self.number(),
            _ => Err(self.fail("valor esperado")),
        }
    }

    fn literal(&mut self, word: &str, value: ConfigValue) -> Result<ConfigValue> {
        for expected in word.bytes() {
            if self.bump() != Some(expected) {
                return Err(self.fail("literal inválido"));
            }
        }
        Ok(value)
    }

    fn array(&mut self) -> Result<ConfigValue> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        loop {
            self.skip_gap();
            match self.peek() {
                Some(b']') => {
                    self.pos += 1;
                    return Ok(ConfigValue::Array(items));
                }
                None => return Err(self.fail("lista sem fechamento")),
                _ => items.push(self.value()?),
            }
            self.skip_gap();
            match self.bump() {
                Some(b',') => {}
                Some(b']') => return Ok(ConfigValue::Array(items)),
                _ => return Err(self.fail("`,` ou `]` esperado na lista")),
            }
        }
    }

    pub(super) fn basic_string(&mut self) -> Result<String> {
        self.expect(b'"')?;
        if self.peek() == Some(b'"') && self.bytes.get(self.pos + 1) == Some(&b'"') {
            return Err(self.fail("string multilinha (`\"\"\"`) não é suportada"));
        }
        let mut out = String::new();
        let mut start = self.pos;
        loop {
            match self.peek() {
                None | Some(b'\n') => return Err(self.fail("string sem fechamento")),
                Some(b'"') => {
                    out.push_str(&self.slice(start, self.pos)?);
                    self.pos += 1;
                    return Ok(out);
                }
                Some(b'\\') => {
                    out.push_str(&self.slice(start, self.pos)?);
                    self.pos += 1;
                    out.push(self.escape()?);
                    start = self.pos;
                }
                Some(_) => self.pos += 1,
            }
        }
    }

    pub(super) fn literal_string(&mut self) -> Result<String> {
        self.expect(b'\'')?;
        if self.peek() == Some(b'\'') && self.bytes.get(self.pos + 1) == Some(&b'\'') {
            return Err(self.fail("string multilinha (`'''`) não é suportada"));
        }
        let start = self.pos;
        while !matches!(self.peek(), None | Some(b'\'' | b'\n')) {
            self.pos += 1;
        }
        if self.peek() != Some(b'\'') {
            return Err(self.fail("string literal sem fechamento"));
        }
        let text = self.slice(start, self.pos)?;
        self.pos += 1;
        Ok(text)
    }

    fn escape(&mut self) -> Result<char> {
        match self.bump() {
            Some(b'"') => Ok('"'),
            Some(b'\\') => Ok('\\'),
            Some(b'n') => Ok('\n'),
            Some(b'r') => Ok('\r'),
            Some(b't') => Ok('\t'),
            Some(b'b') => Ok('\u{8}'),
            Some(b'f') => Ok('\u{c}'),
            Some(b'u') => self.unicode(4),
            Some(b'U') => self.unicode(8),
            _ => Err(self.fail("escape inválido")),
        }
    }

    fn unicode(&mut self, digits: usize) -> Result<char> {
        let mut code: u32 = 0;
        for _ in 0..digits {
            let byte = self
                .bump()
                .ok_or_else(|| self.fail("escape unicode incompleto"))?;
            let digit = char::from(byte)
                .to_digit(16)
                .ok_or_else(|| self.fail("dígito hexadecimal esperado"))?;
            code = code.saturating_mul(16).saturating_add(digit);
        }
        char::from_u32(code).ok_or_else(|| self.fail("code point inválido"))
    }

    fn number(&mut self) -> Result<ConfigValue> {
        let start = self.pos;
        if matches!(self.peek(), Some(b'+' | b'-')) {
            self.pos += 1;
        }
        while matches!(
            self.peek(),
            Some(b'0'..=b'9' | b'_' | b'.' | b'e' | b'E' | b'+' | b'-')
        ) {
            self.pos += 1;
        }
        let raw = self.slice(start, self.pos)?;
        let cleaned: String = raw.chars().filter(|c| *c != '_').collect();
        let is_float = cleaned.contains('.')
            || cleaned.contains('e')
            || cleaned.contains('E')
            || cleaned.eq_ignore_ascii_case("inf")
            || cleaned.eq_ignore_ascii_case("nan");
        if is_float {
            cleaned
                .parse::<f64>()
                .map(ConfigValue::Float)
                .map_err(|_| self.fail("float inválido"))
        } else {
            cleaned
                .parse::<i64>()
                .map(ConfigValue::Int)
                .map_err(|_| self.fail("inteiro inválido"))
        }
    }
}
