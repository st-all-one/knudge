//! Flush coalescido do índice vetorial (E11-T06, D85).
//!
//! O índice marca *dirty* e grava no máximo a cada `flush_ms` (default 2000), com **flush
//! forçado na saída**. Rajadas de 10–20 notas causam um único `write` O(N).

/// Debounce default em milissegundos.
pub const DEFAULT_FLUSH_MS: i64 = 2000;

/// Estado de flush com *dirty flag*.
#[derive(Debug, Clone)]
pub struct FlushState {
    dirty: bool,
    last_flush_ms: i64,
}

impl FlushState {
    /// Estado limpo, ancorado em `now_ms`.
    #[must_use]
    pub const fn new(now_ms: i64) -> Self {
        Self {
            dirty: false,
            last_flush_ms: now_ms,
        }
    }

    /// Marca que há mudanças pendentes.
    pub const fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// `true` se há mudanças pendentes.
    #[must_use]
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// `true` se já passou o debounce desde o último flush.
    #[must_use]
    pub fn should_flush(&self, now_ms: i64, flush_ms: i64) -> bool {
        self.dirty && now_ms.saturating_sub(self.last_flush_ms) >= flush_ms.max(0)
    }

    /// Faz o flush se o debounce venceu; devolve `true` quando gravou.
    pub fn take(&mut self, now_ms: i64, flush_ms: i64) -> bool {
        if self.should_flush(now_ms, flush_ms) {
            self.force(now_ms);
            true
        } else {
            false
        }
    }

    /// Força o flush (saída do processo), limpando o *dirty flag*.
    pub const fn force(&mut self, now_ms: i64) {
        self.dirty = false;
        self.last_flush_ms = now_ms;
    }
}
