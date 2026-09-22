//! Adaptador HTTP OpenAI-compatible para embeddings (E11-T01/T10, D101).
//!
//! O modelo roda **fora** do binário: o usuário sobe um servidor local (`llama-server` com o
//! GGUF `msmarco-MiniLM-L12-cos-v5.Q5_K_M`, TEI, Ollama, vLLM…) e o knudge só aponta a URL.
//! Cliente HTTP/1.1 bloqueante sobre `std::net` — **sem** `tokio`/`reqwest` (R16/R43). Timeout
//! tipado e retry/backoff só em HTTP idempotente (R12). `https://` exige um proxy/TLS terminator.

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use indexmap::IndexMap;

use crate::config::Config;
use crate::embeddings::EmbeddingMeta;
use crate::embeddings::vector::from_f64;
use crate::jsonl::json;
use crate::ports::{Embedder, Env};
use crate::schema::Value;
use crate::{Error, ErrorKind, Result};

/// Endpoint default (servidor local OpenAI-compatible).
pub const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:8080/v1/embeddings";

/// Embedder HTTP bloqueante.
pub struct HttpEmbedder {
    meta: EmbeddingMeta,
    endpoint: Endpoint,
    api_key: Option<String>,
    timeout: Duration,
    retries: u32,
}

impl HttpEmbedder {
    /// Monta a partir da config efetiva e do ambiente.
    ///
    /// # Errors
    /// Retorna `ErrorKind::Config` para endpoint/dimensão inválidos.
    pub fn new(config: &Config, env: &dyn Env) -> Result<Self> {
        let endpoint = Endpoint::parse(
            config
                .get_str("embeddings.endpoint")
                .unwrap_or(DEFAULT_ENDPOINT),
        )?;
        let api_key = config
            .get_str("embeddings.api_key_env")
            .and_then(|key| env.var(key))
            .filter(|value| !value.is_empty());
        let timeout_ms = config.get_int("embeddings.timeout_ms").unwrap_or(30_000);
        let timeout = Duration::from_millis(u64::try_from(timeout_ms.max(1)).unwrap_or(30_000));
        let retries = config.get_int("embeddings.retries").unwrap_or(2);
        let retries = u32::try_from(retries.max(0)).unwrap_or(0);
        Ok(Self {
            meta: EmbeddingMeta::from_config(config)?,
            endpoint,
            api_key,
            timeout,
            retries,
        })
    }

    fn request(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let body = build_body(&self.meta.model, texts)?;
        let mut attempt = 0_u32;
        loop {
            match self.post_once(&body) {
                Ok((status, response)) => {
                    if (200..300).contains(&status) {
                        return parse_vectors(&response);
                    }
                    let error =
                        Error::internal(format!("HTTP {status}: {}", truncate(&response, 200)));
                    if status >= 500 && attempt < self.retries {
                        std::thread::sleep(retry_delay(attempt));
                        attempt = attempt.saturating_add(1);
                        continue;
                    }
                    return Err(error);
                }
                Err(error) => {
                    if error.kind() == ErrorKind::Timeout && attempt < self.retries {
                        std::thread::sleep(retry_delay(attempt));
                        attempt = attempt.saturating_add(1);
                        continue;
                    }
                    return Err(error);
                }
            }
        }
    }

    fn post_once(&self, body: &str) -> Result<(u16, String)> {
        let authority = self.endpoint.authority();
        let address = (self.endpoint.host.as_str(), self.endpoint.port)
            .to_socket_addrs()
            .map_err(|error| net_error(&authority, &error))?
            .next()
            .ok_or_else(|| Error::internal(format!("endpoint não resolve: {authority}")))?;
        let mut stream = TcpStream::connect_timeout(&address, self.timeout)
            .map_err(|error| net_error(&authority, &error))?;
        stream
            .set_read_timeout(Some(self.timeout))
            .map_err(|error| net_error(&authority, &error))?;
        stream
            .set_write_timeout(Some(self.timeout))
            .map_err(|error| net_error(&authority, &error))?;
        stream
            .write_all(self.build_request(body).as_bytes())
            .map_err(|error| net_error(&authority, &error))?;
        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .map_err(|error| net_error(&authority, &error))?;
        let text = String::from_utf8_lossy(&response).into_owned();
        let (status, payload) = split_response(&text)?;
        Ok((status, payload.to_string()))
    }

    fn build_request(&self, body: &str) -> String {
        let mut request = format!(
            "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
            self.endpoint.path,
            self.endpoint.authority(),
            body.len()
        );
        if let Some(key) = &self.api_key {
            request.push_str("Authorization: Bearer ");
            request.push_str(key);
            request.push_str("\r\n");
        }
        request.push_str("\r\n");
        request.push_str(body);
        request
    }
}

impl Embedder for HttpEmbedder {
    fn meta(&self) -> &EmbeddingMeta {
        &self.meta
    }

    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        self.request(texts)
    }
}

/// Endpoint HTTP já decomposto.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Endpoint {
    host: String,
    port: u16,
    path: String,
}

impl Endpoint {
    fn parse(url: &str) -> Result<Self> {
        let rest = url.strip_prefix("http://").ok_or_else(|| {
            Error::config(
                "embeddings.endpoint deve começar com http:// (https exige proxy/TLS terminator)",
            )
        })?;
        let (authority, path) = match rest.split_once('/') {
            Some((authority, path)) => (authority, format!("/{path}")),
            None => (rest, "/".to_string()),
        };
        let (host, port) = match authority.rsplit_once(':') {
            Some((host, port)) => (
                host,
                port.parse::<u16>()
                    .map_err(|_| Error::config(format!("porta inválida: {port}")))?,
            ),
            None => (authority, 80),
        };
        if host.is_empty() {
            return Err(Error::config("endpoint sem host"));
        }
        Ok(Self {
            host: host.to_string(),
            port,
            path,
        })
    }

    fn authority(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

fn build_body(model: &str, texts: &[String]) -> Result<String> {
    let mut map = IndexMap::new();
    map.insert("model".to_string(), Value::Str(model.to_string()));
    map.insert(
        "input".to_string(),
        Value::List(texts.iter().map(|text| Value::Str(text.clone())).collect()),
    );
    json::encode(&Value::Map(map))
}

fn parse_vectors(body: &str) -> Result<Vec<Vec<f32>>> {
    let value = json::decode(body)?;
    let map = value
        .as_map()
        .ok_or_else(|| Error::internal("resposta de embeddings não é objeto"))?;
    if let Some(data) = map.get("data").and_then(Value::as_list) {
        return data
            .iter()
            .map(|item| {
                let embedding = item
                    .as_map()
                    .and_then(|map| map.get("embedding"))
                    .and_then(Value::as_list)
                    .ok_or_else(|| Error::internal("item de `data` sem `embedding`"))?;
                Ok(embedding
                    .iter()
                    .filter_map(Value::as_f64)
                    .map(from_f64)
                    .collect())
            })
            .collect();
    }
    if let Some(list) = map.get("embeddings").and_then(Value::as_list) {
        return list
            .iter()
            .map(|item| {
                item.as_list()
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(Value::as_f64)
                            .map(from_f64)
                            .collect()
                    })
                    .ok_or_else(|| Error::internal("`embeddings` deve conter listas"))
            })
            .collect();
    }
    Err(Error::internal(
        "resposta de embeddings sem `data`/`embeddings`",
    ))
}

fn split_response(response: &str) -> Result<(u16, &str)> {
    let (head, body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| Error::internal("resposta HTTP malformada"))?;
    let status_line = head
        .lines()
        .next()
        .ok_or_else(|| Error::internal("resposta HTTP sem status"))?;
    let mut parts = status_line.split_whitespace();
    let _version = parts.next();
    let status = parts
        .next()
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| Error::internal(format!("status HTTP inválido: {status_line:?}")))?;
    Ok((status, body))
}

fn net_error(authority: &str, error: &std::io::Error) -> Error {
    if matches!(
        error.kind(),
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
    ) {
        Error::timeout(format!("{authority}: {error}"))
    } else {
        Error::internal(format!("{authority}: {error}"))
    }
}

fn truncate(text: &str, max: usize) -> String {
    text.chars().take(max).collect()
}

fn retry_delay(attempt: u32) -> Duration {
    let factor = 1_u64.checked_shl(attempt).unwrap_or(u64::MAX);
    let ms = 100_u64.saturating_mul(factor).min(2_000);
    Duration::from_millis(ms)
}

#[cfg(test)]
mod tests;
