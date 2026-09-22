//! Parser TOML do subconjunto usado pela configuração do knudge.
//!
//! Suporta: comentários (`#`), `[tabela]` e `[tabela.sub]`, `chave = valor` (chaves bare ou
//! citadas, pontilhadas), strings básicas/literais de uma linha, inteiros, floats, booleanos e
//! listas (inclusive multilinha). Não suporta `[[]]`, strings multilinha `"""` nem `null`.
//! A leitura é tolerante a chaves desconhecidas (validadas só no `config set` — D64).

#![allow(
    clippy::arithmetic_side_effects,
    reason = "índices de scanner com domínio limitado ao tamanho do buffer"
)]

use crate::config::value::Table;
use crate::{Error, Result};

mod insert;
mod value;

use insert::{ensure_table, insert_leaf};

/// Interpreta um documento TOML.
///
/// # Errors
/// Retorna `ErrorKind::Config` para sintaxe inválida.
pub fn parse(text: &str) -> Result<Table> {
    Parser {
        bytes: text.as_bytes(),
        pos: 0,
    }
    .document()
}

/// Scanner de TOML; os `impl` são distribuídos em `value.rs`/`insert.rs`.
struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn fail(&self, message: &str) -> Error {
        Error::config(format!("TOML inválido na posição {}: {message}", self.pos))
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

    fn at_end(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn skip_inline_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t')) {
            self.pos += 1;
        }
    }

    fn skip_to_line_end(&mut self) {
        if self.peek() == Some(b'#') {
            while !matches!(self.peek(), None | Some(b'\n')) {
                self.pos += 1;
            }
        }
    }

    fn skip_gap(&mut self) {
        loop {
            match self.peek() {
                Some(b' ' | b'\t' | b'\r' | b'\n') => self.pos += 1,
                Some(b'#') => self.skip_to_line_end(),
                _ => break,
            }
        }
    }

    fn expect_line_end(&mut self) -> Result<()> {
        self.skip_inline_ws();
        self.skip_to_line_end();
        match self.peek() {
            None => Ok(()),
            Some(b'\n') => {
                self.pos += 1;
                Ok(())
            }
            Some(b'\r') => {
                self.pos += 1;
                if self.peek() == Some(b'\n') {
                    self.pos += 1;
                }
                Ok(())
            }
            _ => Err(self.fail("conteúdo inesperado após o valor")),
        }
    }

    fn expect(&mut self, byte: u8) -> Result<()> {
        if self.bump() == Some(byte) {
            Ok(())
        } else {
            Err(self.fail("byte inesperado"))
        }
    }

    fn document(&mut self) -> Result<Table> {
        let mut root = Table::new();
        let mut current: Vec<String> = Vec::new();
        loop {
            self.skip_gap();
            if self.at_end() {
                return Ok(root);
            }
            if self.peek() == Some(b'[') {
                current = self.header()?;
                self.expect_line_end()?;
                ensure_table(&mut root, &current)?;
            } else {
                let mut path = current.clone();
                path.extend(self.key_path()?);
                self.skip_inline_ws();
                self.expect(b'=')?;
                self.skip_inline_ws();
                let value = self.value()?;
                self.expect_line_end()?;
                insert_leaf(&mut root, &path, value)?;
            }
        }
    }

    fn header(&mut self) -> Result<Vec<String>> {
        self.expect(b'[')?;
        if self.peek() == Some(b'[') {
            return Err(self.fail("array de tabelas (`[[...]]`) não é suportado"));
        }
        let path = self.key_path()?;
        self.skip_inline_ws();
        self.expect(b']')?;
        Ok(path)
    }

    fn key_path(&mut self) -> Result<Vec<String>> {
        let mut parts = vec![self.key_part()?];
        loop {
            self.skip_inline_ws();
            if self.peek() != Some(b'.') {
                return Ok(parts);
            }
            self.pos += 1;
            self.skip_inline_ws();
            parts.push(self.key_part()?);
        }
    }

    fn key_part(&mut self) -> Result<String> {
        match self.peek() {
            Some(b'"') => self.basic_string(),
            Some(b'\'') => self.literal_string(),
            Some(_) => {
                let start = self.pos;
                while matches!(self.peek(), Some(b) if b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                {
                    self.pos += 1;
                }
                if self.pos == start {
                    Err(self.fail("chave esperada"))
                } else {
                    self.slice(start, self.pos)
                }
            }
            None => Err(self.fail("chave esperada")),
        }
    }

    fn slice(&self, start: usize, end: usize) -> Result<String> {
        let bytes = self.bytes.get(start..end).unwrap_or_default();
        std::str::from_utf8(bytes)
            .map(str::to_string)
            .map_err(|_| self.fail("UTF-8 inválido"))
    }
}
