//! Handshake e versão do protocolo MCP (E14-T02, D68).

/// Nome do servidor anunciado ao cliente.
pub const SERVER_NAME: &str = "knudge";

/// Versão mais nova do protocolo suportada.
pub const PROTOCOL_VERSION: &str = "2025-06-18";

/// Versões suportadas, da mais nova para a mais antiga.
pub const SUPPORTED_VERSIONS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];

/// Negocia a versão: ecoa a pedida se suportada; senão, devolve a mais nova.
#[must_use]
pub fn negotiate(requested: Option<&str>) -> &'static str {
    match requested {
        Some(version) if SUPPORTED_VERSIONS.contains(&version) => SUPPORTED_VERSIONS
            .iter()
            .copied()
            .find(|supported| *supported == version)
            .unwrap_or(PROTOCOL_VERSION),
        _ => PROTOCOL_VERSION,
    }
}
