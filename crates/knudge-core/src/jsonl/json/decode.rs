//! Decodificação JSON estrita de um único valor.

#![allow(
    clippy::arithmetic_side_effects,
    reason = "índices de parser com domínio limitado ao tamanho do buffer"
)]

use indexmap::IndexMap;

use crate::schema::Value;
use crate::{Error, Result};

/// Decodifica um único valor JSON (sem lixo após o fim).
///
/// # Errors
/// Retorna `ErrorKind::InvalidInput` para qualquer sintaxe inválida.
pub fn decode(text: &str) -> Result<Value> {
    let mut parser = Parser {
        bytes: text.as_bytes(),
        pos: 0,
    };
    parser.skip_ws();
    let value = parser.parse_value()?;
    parser.skip_ws();
    if parser.pos != parser.bytes.len() {
        return Err(parser.fail("conteúdo extra após o valor JSON"));
    }
    Ok(value)
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn fail(&self, message: &str) -> Error {
        Error::invalid_input(format!("JSON inválido na posição {}: {message}", self.pos))
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let byte = self.peek();
        if byte.is_some() {
            self.pos += 1;
        }
        byte
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, byte: u8) -> Result<()> {
        if self.bump() == Some(byte) {
            Ok(())
        } else {
            Err(self.fail("byte inesperado"))
        }
    }

    fn parse_value(&mut self) -> Result<Value> {
        match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => Ok(Value::Str(self.parse_string()?)),
            Some(b't') => self.parse_literal("true", Value::Bool(true)),
            Some(b'f') => self.parse_literal("false", Value::Bool(false)),
            Some(b'n') => Err(self.fail("`null` não é suportado")),
            Some(b'-' | b'0'..=b'9') => self.parse_number(),
            _ => Err(self.fail("valor esperado")),
        }
    }

    fn parse_literal(&mut self, word: &str, value: Value) -> Result<Value> {
        for expected in word.bytes() {
            if self.bump() != Some(expected) {
                return Err(self.fail("literal inválido"));
            }
        }
        Ok(value)
    }

    fn parse_object(&mut self) -> Result<Value> {
        self.expect(b'{')?;
        let mut map = IndexMap::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Value::Map(map));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            self.expect(b':')?;
            self.skip_ws();
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_ws();
            match self.bump() {
                Some(b',') => {}
                Some(b'}') => return Ok(Value::Map(map)),
                _ => return Err(self.fail("`,` ou `}` esperado no objeto")),
            }
        }
    }

    fn parse_array(&mut self) -> Result<Value> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Value::List(items));
        }
        loop {
            self.skip_ws();
            items.push(self.parse_value()?);
            self.skip_ws();
            match self.bump() {
                Some(b',') => {}
                Some(b']') => return Ok(Value::List(items)),
                _ => return Err(self.fail("`,` ou `]` esperado no array")),
            }
        }
    }

    fn parse_string(&mut self) -> Result<String> {
        self.expect(b'"')?;
        let mut out = String::new();
        let mut start = self.pos;
        loop {
            match self.peek() {
                None => return Err(self.fail("string sem fechamento")),
                Some(b'"') => {
                    out.push_str(self.slice(start, self.pos)?);
                    self.pos += 1;
                    return Ok(out);
                }
                Some(b'\\') => {
                    out.push_str(self.slice(start, self.pos)?);
                    self.pos += 1;
                    out.push(self.parse_escape()?);
                    start = self.pos;
                }
                Some(_) => self.pos += 1,
            }
        }
    }

    fn slice(&self, start: usize, end: usize) -> Result<&str> {
        let bytes = self.bytes.get(start..end).unwrap_or_default();
        std::str::from_utf8(bytes).map_err(|_| self.fail("UTF-8 inválido"))
    }

    fn parse_escape(&mut self) -> Result<char> {
        match self.bump() {
            Some(b'"') => Ok('"'),
            Some(b'\\') => Ok('\\'),
            Some(b'/') => Ok('/'),
            Some(b'n') => Ok('\n'),
            Some(b'r') => Ok('\r'),
            Some(b't') => Ok('\t'),
            Some(b'b') => Ok('\u{8}'),
            Some(b'f') => Ok('\u{c}'),
            Some(b'u') => self.parse_unicode(),
            _ => Err(self.fail("escape inválido")),
        }
    }

    fn parse_unicode(&mut self) -> Result<char> {
        let high = self.parse_hex4()?;
        if (0xD800..=0xDBFF).contains(&high) {
            self.expect(b'\\')?;
            self.expect(b'u')?;
            let low = self.parse_hex4()?;
            if !(0xDC00..=0xDFFF).contains(&low) {
                return Err(self.fail("par surrogate inválido"));
            }
            let combined =
                0x1_0000 + ((u32::from(high) - 0xD800) << 10) + (u32::from(low) - 0xDC00);
            return char::from_u32(combined).ok_or_else(|| self.fail("code point inválido"));
        }
        char::from_u32(u32::from(high)).ok_or_else(|| self.fail("code point inválido"))
    }

    fn parse_hex4(&mut self) -> Result<u16> {
        let mut value: u16 = 0;
        for _ in 0..4 {
            let byte = self.bump().ok_or_else(|| self.fail("\\u incompleto"))?;
            let digit = char::from(byte)
                .to_digit(16)
                .ok_or_else(|| self.fail("dígito hexadecimal esperado"))?;
            value = value * 16 + u16::try_from(digit).unwrap_or(0);
        }
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<Value> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        let mut float = false;
        if self.peek() == Some(b'.') {
            float = true;
            self.pos += 1;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            float = true;
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        let text = self.slice(start, self.pos)?;
        if float {
            text.parse::<f64>()
                .map(Value::Float)
                .map_err(|_| self.fail("float inválido"))
        } else {
            text.parse::<i64>()
                .map(Value::Int)
                .map_err(|_| self.fail("inteiro inválido"))
        }
    }
}
