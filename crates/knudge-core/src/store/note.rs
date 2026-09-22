//! Nota canônica = frontmatter TOON + corpo (a verdade em `notas/<id>.md`).

use crate::schema::{Frontmatter, Value, body};
use crate::{Error, Result, toon};

/// Fronteira do frontmatter (mesma constante do parser TOON).
const FENCE: &str = toon::FENCE;

/// Unidade de recuperação e fonte da verdade.
#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    /// Frontmatter canônico (ordem e tipos travados por E02).
    pub frontmatter: Frontmatter,
    /// Corpo: contexto que o `statement` não diz. Pode ser vazio.
    pub body: String,
}

impl Note {
    /// Cria uma nota.
    #[must_use]
    pub fn new(frontmatter: Frontmatter, body: impl Into<String>) -> Self {
        Self {
            frontmatter,
            body: body.into(),
        }
    }

    /// Parseia e **valida** os bytes de um arquivo de nota.
    ///
    /// # Errors
    /// Retorna `ErrorKind::InvalidInput` se não for UTF-8 e `ErrorKind::Schema` se o
    /// frontmatter for inválido.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let text =
            std::str::from_utf8(bytes).map_err(|_| Error::invalid_input("nota não é UTF-8"))?;
        let (front, body) = toon::split_frontmatter(text)?;
        let (frontmatter, _warnings) = Frontmatter::parse(&front)?;
        frontmatter.validate()?;
        Ok(Self { frontmatter, body })
    }

    /// Serializa para o formato de arquivo (`---` + TOON + `---` + corpo).
    #[must_use]
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(FENCE);
        out.push('\n');
        out.push_str(&self.frontmatter.to_string());
        out.push_str(FENCE);
        out.push('\n');
        out.push_str(&self.body);
        out
    }

    /// `id` da nota.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Schema` se o campo estiver ausente/inválido.
    pub fn id(&self) -> Result<&str> {
        self.frontmatter.id()
    }

    /// `revision` (contador de versões; default 1 na criação).
    #[must_use]
    pub fn revision(&self) -> u32 {
        self.frontmatter
            .get("revision")
            .and_then(Value::as_int)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(1)
    }

    /// Define `revision`.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Schema` se a chave não for canônica (nunca ocorre).
    pub fn set_revision(&mut self, revision: u32) -> Result<()> {
        self.frontmatter
            .set("revision", Value::Int(i64::from(revision)))
    }

    /// Recalcula `body_hash` a partir de `statement` + corpo (D06).
    ///
    /// # Errors
    /// Retorna `ErrorKind::Schema` se `statement` estiver ausente.
    pub fn refresh_body_hash(&mut self) -> Result<()> {
        let statement = self.frontmatter.statement()?.to_string();
        let hash = body::body_hash(&statement, &self.body);
        self.frontmatter.set("body_hash", Value::Str(hash))
    }
}
