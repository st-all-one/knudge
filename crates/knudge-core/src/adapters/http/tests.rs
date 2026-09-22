//! Testes do adaptador HTTP de embeddings (E11-T01/T10).

use super::*;

#[test]
fn parses_endpoint_with_port_and_path() -> Result<()> {
    let endpoint = Endpoint::parse("http://127.0.0.1:8080/v1/embeddings")?;
    assert_eq!(endpoint.host, "127.0.0.1");
    assert_eq!(endpoint.port, 8080);
    assert_eq!(endpoint.path, "/v1/embeddings");
    assert_eq!(endpoint.authority(), "127.0.0.1:8080");
    Ok(())
}

#[test]
fn parses_endpoint_default_port_and_root() -> Result<()> {
    let endpoint = Endpoint::parse("http://localhost")?;
    assert_eq!(endpoint.port, 80);
    assert_eq!(endpoint.path, "/");
    Ok(())
}

#[test]
fn rejects_https_and_bad_port() {
    assert!(Endpoint::parse("https://api.openai.com/v1/embeddings").is_err());
    assert!(Endpoint::parse("http://localhost:notaport/x").is_err());
    assert!(Endpoint::parse("http://:8080/x").is_err());
}

#[test]
fn builds_openai_body() -> Result<()> {
    let body = build_body("modelo", &["a".to_string(), "b".to_string()])?;
    assert!(body.contains("\"model\":\"modelo\""));
    assert!(body.contains("\"input\":[\"a\",\"b\"]"));
    Ok(())
}

#[test]
fn parses_openai_and_legacy_responses() -> Result<()> {
    let openai = "{\"data\":[{\"embedding\":[0.5,0.5]},{\"embedding\":[1,0]}]}";
    assert_eq!(parse_vectors(openai)?.len(), 2);
    let legacy = "{\"embeddings\":[[0.5,0.5],[1,0]]}";
    assert_eq!(parse_vectors(legacy)?.len(), 2);
    assert!(parse_vectors("{\"object\":\"list\"}").is_err());
    Ok(())
}

#[test]
fn splits_http_response() -> Result<()> {
    let response = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}";
    let (status, body) = split_response(response)?;
    assert_eq!(status, 200);
    assert_eq!(body, "{}");
    assert!(split_response("lixo").is_err());
    Ok(())
}

#[test]
fn retry_delay_is_bounded_and_exponential() {
    assert_eq!(retry_delay(0), Duration::from_millis(100));
    assert_eq!(retry_delay(1), Duration::from_millis(200));
    assert!(retry_delay(30) <= Duration::from_millis(2_000));
}
