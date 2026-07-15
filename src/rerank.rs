//! Optional remote reranker contract and environment-only configuration.
//!
//! C08a defines types only. Network adaptation is C08b; retrieval/runtime
//! wiring is C08c.

use std::ffi::OsString;
use std::fmt;
use std::time::Duration;

pub(crate) const PROVIDER_ENV: &str = "NOWDOCS_RERANK_PROVIDER";
pub(crate) const MODEL_ENV: &str = "NOWDOCS_RERANK_MODEL";
pub(crate) const TIMEOUT_ENV: &str = "NOWDOCS_RERANK_TIMEOUT_MS";
pub(crate) const COHERE_API_KEY_ENV: &str = "COHERE_API_KEY";

const DEFAULT_TIMEOUT_MS: u64 = 2000;
const MIN_TIMEOUT_MS: u64 = 100;
const MAX_TIMEOUT_MS: u64 = 10000;
const COHERE_RERANK_URL: &str = "https://api.cohere.com/v2/rerank";
const MAX_RERANK_DOCUMENTS: usize = 40;
const MAX_RERANK_DOCUMENT_BYTES: usize = 8192;
const MAX_RERANK_RESPONSE_BYTES: usize = 1024 * 1024;
const COHERE_MAX_TOKENS_PER_DOC: u32 = 4096;
const COHERE_USER_AGENT: &str = concat!("nowdocs/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RerankConfig {
    Disabled,
    Cohere(CohereConfig),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CohereConfig {
    pub(crate) model: String,
    pub(crate) api_key: CohereApiKey,
    pub(crate) timeout: Duration,
}

#[derive(PartialEq, Eq)]
pub(crate) struct CohereApiKey(String);

impl CohereApiKey {
    pub(crate) fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for CohereApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CohereApiKey").field(&"[REDACTED]").finish()
    }
}

pub(crate) struct CohereReranker {
    agent: ureq::Agent,
    endpoint: String,
    model: String,
    api_key: CohereApiKey,
}

impl fmt::Debug for CohereReranker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CohereReranker")
            .field("endpoint", &self.endpoint)
            .field("model", &self.model)
            .field("api_key", &self.api_key)
            .finish()
    }
}

impl CohereReranker {
    pub(crate) fn new(config: CohereConfig) -> Self {
        Self::build(config, COHERE_RERANK_URL.to_owned(), true)
    }

    #[cfg(test)]
    fn new_for_test(config: CohereConfig, port: u16) -> Self {
        Self::build(config, format!("http://127.0.0.1:{port}/v2/rerank"), false)
    }

    fn build(config: CohereConfig, endpoint: String, https_only: bool) -> Self {
        let agent_config = ureq::Agent::config_builder()
            .timeout_global(Some(config.timeout))
            .https_only(https_only)
            .max_redirects(0)
            .http_status_as_error(false)
            .user_agent(COHERE_USER_AGENT)
            .build();
        Self {
            agent: ureq::Agent::new_with_config(agent_config),
            endpoint,
            model: config.model,
            api_key: config.api_key,
        }
    }
}

#[derive(serde::Serialize)]
struct CohereRerankRequest<'a> {
    model: &'a str,
    query: &'a str,
    documents: Vec<&'a str>,
    top_n: usize,
    max_tokens_per_doc: u32,
}

#[derive(serde::Deserialize)]
struct CohereRerankResponse {
    results: Vec<CohereRerankResult>,
}

#[derive(serde::Deserialize)]
struct CohereRerankResult {
    index: usize,
}

fn validate_request_inputs(query: &str, documents: &[RerankDocument]) -> Result<(), RerankFailure> {
    let invalid_count = documents.is_empty() || documents.len() > MAX_RERANK_DOCUMENTS;
    if query.is_empty() || invalid_count {
        return Err(RerankFailure::new(RerankFailureClass::InvalidRequest));
    }
    let mut stable_ids = std::collections::HashSet::with_capacity(documents.len());
    for document in documents {
        if document.document.is_empty()
            || document.document.len() > MAX_RERANK_DOCUMENT_BYTES
            || !stable_ids.insert(document.stable_id)
        {
            return Err(RerankFailure::new(RerankFailureClass::InvalidRequest));
        }
    }
    Ok(())
}

fn classify_status(status: u16) -> Option<RerankFailureClass> {
    match status {
        200..=299 => None,
        401 | 403 | 498 => Some(RerankFailureClass::Authentication),
        429 => Some(RerankFailureClass::RateLimit),
        499 => Some(RerankFailureClass::Network),
        400..=499 => Some(RerankFailureClass::InvalidRequest),
        500..=599 => Some(RerankFailureClass::ServerError),
        _ => Some(RerankFailureClass::InvalidResponse),
    }
}

fn map_transport_error(error: ureq::Error) -> RerankFailure {
    let class = match error {
        ureq::Error::Timeout(_) => RerankFailureClass::Timeout,
        ureq::Error::StatusCode(status) => {
            classify_status(status).unwrap_or(RerankFailureClass::InvalidResponse)
        }
        ureq::Error::Http(_)
        | ureq::Error::BadUri(_)
        | ureq::Error::BodyExceedsLimit(_)
        | ureq::Error::RequireHttpsOnly(_) => RerankFailureClass::InvalidRequest,
        _ => RerankFailureClass::Network,
    };
    RerankFailure::new(class)
}

fn ordered_stable_ids(
    response: CohereRerankResponse,
    documents: &[RerankDocument],
) -> Result<Vec<u32>, RerankFailure> {
    if response.results.len() != documents.len() {
        return Err(RerankFailure::new(RerankFailureClass::InvalidResponse));
    }
    let mut seen = vec![false; documents.len()];
    let mut ordered = Vec::with_capacity(documents.len());
    for result in response.results {
        if result.index >= documents.len() || seen[result.index] {
            return Err(RerankFailure::new(RerankFailureClass::InvalidResponse));
        }
        seen[result.index] = true;
        ordered.push(documents[result.index].stable_id);
    }
    Ok(ordered)
}

impl Reranker for CohereReranker {
    fn rerank(&self, query: &str, documents: &[RerankDocument]) -> Result<Vec<u32>, RerankFailure> {
        validate_request_inputs(query, documents)?;
        let request = CohereRerankRequest {
            model: &self.model,
            query,
            documents: documents
                .iter()
                .map(|document| document.document.as_str())
                .collect(),
            top_n: documents.len(),
            max_tokens_per_doc: COHERE_MAX_TOKENS_PER_DOC,
        };
        let body = serde_json::to_vec(&request)
            .map_err(|_| RerankFailure::new(RerankFailureClass::InvalidRequest))?;
        let authorization = format!("Bearer {}", self.api_key.expose_secret());
        let mut response = self
            .agent
            .post(self.endpoint.as_str())
            .header("Authorization", authorization.as_str())
            .header("Accept", "application/json")
            .content_type("application/json")
            .send(body.as_slice())
            .map_err(map_transport_error)?;
        if let Some(class) = classify_status(response.status().as_u16()) {
            return Err(RerankFailure::new(class));
        }
        let bytes = response
            .body_mut()
            .with_config()
            .limit((MAX_RERANK_RESPONSE_BYTES + 1) as u64)
            .read_to_vec()
            .map_err(|_| RerankFailure::new(RerankFailureClass::InvalidResponse))?;
        if bytes.len() > MAX_RERANK_RESPONSE_BYTES {
            return Err(RerankFailure::new(RerankFailureClass::InvalidResponse));
        }
        let parsed: CohereRerankResponse = serde_json::from_slice(&bytes)
            .map_err(|_| RerankFailure::new(RerankFailureClass::InvalidResponse))?;
        ordered_stable_ids(parsed, documents)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RerankConfigError {
    ProviderRequired { setting: &'static str },
    UnsupportedProvider,
    MissingModel,
    MissingApiKey,
    InvalidTimeout,
    NonUnicode { setting: &'static str },
}

impl fmt::Display for RerankConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProviderRequired { setting } => {
                write!(f, "{setting} requires NOWDOCS_RERANK_PROVIDER=cohere")
            }
            Self::UnsupportedProvider => {
                write!(f, "unsupported rerank provider; expected cohere")
            }
            Self::MissingModel => write!(
                f,
                "NOWDOCS_RERANK_MODEL is required when Cohere reranking is enabled"
            ),
            Self::MissingApiKey => write!(
                f,
                "COHERE_API_KEY is required when Cohere reranking is enabled"
            ),
            Self::InvalidTimeout => write!(
                f,
                "NOWDOCS_RERANK_TIMEOUT_MS must be an integer from 100 through 10000"
            ),
            Self::NonUnicode { setting } => {
                write!(f, "{setting} must contain valid UTF-8")
            }
        }
    }
}

impl std::error::Error for RerankConfigError {}

#[allow(dead_code)] // used by integration tests and callers outside the lib
pub(crate) fn load_rerank_config() -> Result<RerankConfig, RerankConfigError> {
    parse_rerank_config_with(read_process_env)
}

fn read_process_env(name: &str) -> Option<OsString> {
    std::env::var_os(name)
}

pub(crate) fn parse_rerank_config_with<F>(
    mut read_env: F,
) -> Result<RerankConfig, RerankConfigError>
where
    F: FnMut(&str) -> Option<OsString>,
{
    let provider_raw = read_env(PROVIDER_ENV);
    let model_raw = read_env(MODEL_ENV);
    let timeout_raw = read_env(TIMEOUT_ENV);

    let Some(provider_raw) = provider_raw else {
        if model_raw.is_some() {
            return Err(RerankConfigError::ProviderRequired { setting: MODEL_ENV });
        }
        if timeout_raw.is_some() {
            return Err(RerankConfigError::ProviderRequired {
                setting: TIMEOUT_ENV,
            });
        }
        return Ok(RerankConfig::Disabled);
    };

    let provider = into_utf8(provider_raw, PROVIDER_ENV)?;
    if provider.trim() != "cohere" {
        return Err(RerankConfigError::UnsupportedProvider);
    }

    let model = model_raw
        .ok_or(RerankConfigError::MissingModel)
        .and_then(|value| into_utf8(value, MODEL_ENV))?;
    let model = model.trim();
    if model.is_empty() {
        return Err(RerankConfigError::MissingModel);
    }

    let timeout = match timeout_raw {
        None => Duration::from_millis(DEFAULT_TIMEOUT_MS),
        Some(value) => {
            let value = into_utf8(value, TIMEOUT_ENV)?;
            let milliseconds = value
                .trim()
                .parse::<u64>()
                .map_err(|_| RerankConfigError::InvalidTimeout)?;
            if !(MIN_TIMEOUT_MS..=MAX_TIMEOUT_MS).contains(&milliseconds) {
                return Err(RerankConfigError::InvalidTimeout);
            }
            Duration::from_millis(milliseconds)
        }
    };

    let api_key = read_env(COHERE_API_KEY_ENV)
        .ok_or(RerankConfigError::MissingApiKey)
        .and_then(|value| into_utf8(value, COHERE_API_KEY_ENV))?;
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err(RerankConfigError::MissingApiKey);
    }

    Ok(RerankConfig::Cohere(CohereConfig {
        model: model.to_owned(),
        api_key: CohereApiKey(api_key.to_owned()),
        timeout,
    }))
}

fn into_utf8(value: OsString, setting: &'static str) -> Result<String, RerankConfigError> {
    value
        .into_string()
        .map_err(|_| RerankConfigError::NonUnicode { setting })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RerankDocument {
    pub(crate) stable_id: u32,
    pub(crate) document: String,
}

pub trait Reranker: Send + Sync {
    fn rerank(&self, query: &str, documents: &[RerankDocument]) -> Result<Vec<u32>, RerankFailure>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RerankFailureClass {
    Network,
    Timeout,
    Authentication,
    InvalidRequest,
    RateLimit,
    ServerError,
    InvalidResponse,
}

impl RerankFailureClass {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Network => "network",
            Self::Timeout => "timeout",
            Self::Authentication => "authentication",
            Self::InvalidRequest => "invalid_request",
            Self::RateLimit => "rate_limit",
            Self::ServerError => "server_error",
            Self::InvalidResponse => "invalid_response",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RerankFailure {
    class: RerankFailureClass,
}

impl RerankFailure {
    pub const fn new(class: RerankFailureClass) -> Self {
        Self { class }
    }

    #[allow(dead_code)] // used by integration tests
    pub const fn class(&self) -> RerankFailureClass {
        self.class
    }
}

impl fmt::Display for RerankFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.class.label())
    }
}

impl std::error::Error for RerankFailure {}

/// Build a configured reranker from environment variables. Returns `Ok(None)`
/// when the rerank feature is not configured. Returns `Err` for a partial or
/// invalid opt-in — the caller must surface the error before entering the read
/// loop.
pub(crate) fn configured_reranker() -> Result<Option<Box<dyn Reranker>>, RerankConfigError> {
    configured_reranker_with(read_process_env)
}

pub(crate) fn configured_reranker_with<F>(
    read_env: F,
) -> Result<Option<Box<dyn Reranker>>, RerankConfigError>
where
    F: FnMut(&str) -> Option<OsString>,
{
    match parse_rerank_config_with(read_env)? {
        RerankConfig::Disabled => Ok(None),
        RerankConfig::Cohere(config) => Ok(Some(Box::new(CohereReranker::new(config)))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    fn parse(entries: &[(&str, &str)]) -> Result<RerankConfig, RerankConfigError> {
        let values: HashMap<String, OsString> = entries
            .iter()
            .map(|(key, value)| ((*key).to_owned(), OsString::from(*value)))
            .collect();
        parse_rerank_config_with(|name| values.get(name).cloned())
    }

    fn config(timeout: Duration) -> CohereConfig {
        CohereConfig {
            model: "rerank-v4.0-fast".to_owned(),
            api_key: CohereApiKey("test-api-key".to_owned()),
            timeout,
        }
    }

    fn documents() -> Vec<RerankDocument> {
        vec![
            RerankDocument {
                stable_id: 11,
                document: "first document".to_owned(),
            },
            RerankDocument {
                stable_id: 22,
                document: "second document".to_owned(),
            },
            RerankDocument {
                stable_id: 33,
                document: "third document".to_owned(),
            },
        ]
    }

    struct MockResponse {
        status: u16,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
        delay: Option<Duration>,
    }

    fn mock_server(
        response: MockResponse,
    ) -> (u16, mpsc::Receiver<Vec<u8>>, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let (sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let request = read_request(&mut stream);
            let _ = sender.send(request);
            if let Some(delay) = response.delay {
                thread::sleep(delay);
            }
            let mut raw = format!(
                "HTTP/1.1 {} Test\r\nContent-Length: {}\r\nConnection: close\r\n",
                response.status,
                response.body.len()
            );
            for (name, value) in response.headers {
                raw.push_str(&format!("{name}: {value}\r\n"));
            }
            raw.push_str("\r\n");
            let _ = stream.write_all(raw.as_bytes());
            let _ = stream.write_all(&response.body);
        });
        (port, receiver, handle)
    }

    fn read_request(stream: &mut TcpStream) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut buffer = [0_u8; 4096];
        let header_end = loop {
            let count = stream.read(&mut buffer).unwrap();
            if count == 0 {
                return bytes;
            }
            bytes.extend_from_slice(&buffer[..count]);
            if let Some(end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                break end + 4;
            }
        };
        let headers = String::from_utf8_lossy(&bytes[..header_end]);
        let length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().ok())
                    .flatten()
            })
            .unwrap_or(0);
        while bytes.len() < header_end + length {
            let count = stream.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            bytes.extend_from_slice(&buffer[..count]);
        }
        bytes
    }

    fn response(body: &[u8]) -> MockResponse {
        MockResponse {
            status: 200,
            headers: vec![("Content-Type".to_owned(), "application/json".to_owned())],
            body: body.to_vec(),
            delay: None,
        }
    }

    #[test]
    fn cohere_adapter_success_sends_exact_v2_request_and_maps_provider_order() {
        let body = br#"{"results":[{"index":2,"relevance_score":0.9},{"index":0,"relevance_score":0.8},{"index":1,"relevance_score":0.7}]}"#;
        let (port, requests, handle) = mock_server(response(body));
        let reranker = CohereReranker::new_for_test(config(Duration::from_secs(2)), port);
        let documents = documents();
        let ordered = reranker.rerank("how?", &documents).unwrap();
        assert_eq!(ordered, vec![33, 11, 22]);
        let request = String::from_utf8(requests.recv().unwrap()).unwrap();
        assert!(request.starts_with("POST /v2/rerank HTTP/1.1\r\n"));
        let request_lower = request.to_ascii_lowercase();
        assert!(request_lower.contains("authorization: bearer test-api-key\r\n"));
        assert!(request_lower.contains("accept: application/json\r\n"));
        assert!(request_lower.contains("content-type: application/json\r\n"));
        assert!(request_lower.contains(&format!(
            "user-agent: {}\r\n",
            COHERE_USER_AGENT.to_ascii_lowercase()
        )));
        let json_start = request.find("\r\n\r\n").unwrap() + 4;
        let value: serde_json::Value = serde_json::from_str(&request[json_start..]).unwrap();
        let mut keys: Vec<_> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            ["documents", "max_tokens_per_doc", "model", "query", "top_n"]
        );
        assert_eq!(
            value.get("model").and_then(|v| v.as_str()),
            Some("rerank-v4.0-fast")
        );
        assert_eq!(value.get("query").and_then(|v| v.as_str()), Some("how?"));
        assert_eq!(value.get("top_n").and_then(|v| v.as_u64()), Some(3));
        assert_eq!(
            value.get("max_tokens_per_doc").and_then(|v| v.as_u64()),
            Some(4096)
        );
        assert_eq!(
            value.get("documents").unwrap(),
            &serde_json::json!(["first document", "second document", "third document"])
        );
        assert!(!request[json_start..].contains("test-api-key"));
        let debug = format!("{reranker:?}");
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("test-api-key"));
        handle.join().unwrap();
    }

    #[test]
    fn cohere_adapter_invalid_inputs_fail_before_network() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let port = listener.local_addr().unwrap().port();
        let reranker = CohereReranker::new_for_test(config(Duration::from_secs(2)), port);
        let cases = vec![
            ("", documents()),
            ("query", vec![]),
            (
                "query",
                (0..41)
                    .map(|id| RerankDocument {
                        stable_id: id,
                        document: "x".to_owned(),
                    })
                    .collect(),
            ),
            (
                "query",
                vec![RerankDocument {
                    stable_id: 1,
                    document: String::new(),
                }],
            ),
            (
                "query",
                vec![RerankDocument {
                    stable_id: 1,
                    document: "x".repeat(8193),
                }],
            ),
            (
                "query",
                vec![
                    RerankDocument {
                        stable_id: 1,
                        document: "a".to_owned(),
                    },
                    RerankDocument {
                        stable_id: 1,
                        document: "b".to_owned(),
                    },
                ],
            ),
        ];
        for (query, docs) in cases {
            assert_eq!(
                reranker.rerank(query, &docs).unwrap_err().class(),
                RerankFailureClass::InvalidRequest
            );
        }
        assert!(
            matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
        );
    }

    #[test]
    fn cohere_adapter_response_requires_complete_unique_in_range_indexes() {
        let docs = documents();
        for results in [
            vec![],
            vec![0, 1],
            vec![0, 1, 2, 2],
            vec![0, 0, 1],
            vec![0, 1, 3],
        ] {
            let response = CohereRerankResponse {
                results: results
                    .into_iter()
                    .map(|index| CohereRerankResult { index })
                    .collect(),
            };
            assert_eq!(
                ordered_stable_ids(response, &docs).unwrap_err().class(),
                RerankFailureClass::InvalidResponse
            );
        }
    }

    #[test]
    fn cohere_adapter_status_classification_is_complete() {
        for (status, expected) in [
            (200, None),
            (204, None),
            (302, Some(RerankFailureClass::InvalidResponse)),
            (400, Some(RerankFailureClass::InvalidRequest)),
            (401, Some(RerankFailureClass::Authentication)),
            (403, Some(RerankFailureClass::Authentication)),
            (404, Some(RerankFailureClass::InvalidRequest)),
            (422, Some(RerankFailureClass::InvalidRequest)),
            (429, Some(RerankFailureClass::RateLimit)),
            (498, Some(RerankFailureClass::Authentication)),
            (499, Some(RerankFailureClass::Network)),
            (500, Some(RerankFailureClass::ServerError)),
            (501, Some(RerankFailureClass::ServerError)),
            (503, Some(RerankFailureClass::ServerError)),
            (504, Some(RerankFailureClass::ServerError)),
        ] {
            assert_eq!(classify_status(status), expected);
        }
    }

    #[test]
    fn cohere_adapter_http_error_uses_redacted_class_only() {
        let (port, _, handle) = mock_server(MockResponse {
            status: 401,
            headers: vec![],
            body: b"sensitive-marker query document test-api-key".to_vec(),
            delay: None,
        });
        let reranker = CohereReranker::new_for_test(config(Duration::from_secs(2)), port);
        let error = reranker.rerank("query", &documents()).unwrap_err();
        assert_eq!(error.class(), RerankFailureClass::Authentication);
        let rendered = format!("{error:?} {error}");
        for secret in ["sensitive-marker", "query", "document", "test-api-key"] {
            assert!(!rendered.contains(secret));
        }
        handle.join().unwrap();
    }

    #[test]
    fn cohere_adapter_redirect_is_not_followed() {
        let second = TcpListener::bind("127.0.0.1:0").unwrap();
        second.set_nonblocking(true).unwrap();
        let second_port = second.local_addr().unwrap().port();
        let (port, _, handle) = mock_server(MockResponse {
            status: 302,
            headers: vec![(
                "Location".to_owned(),
                format!("http://127.0.0.1:{second_port}/v2/rerank"),
            )],
            body: Vec::new(),
            delay: None,
        });
        let reranker = CohereReranker::new_for_test(config(Duration::from_secs(2)), port);
        assert_eq!(
            reranker.rerank("query", &documents()).unwrap_err().class(),
            RerankFailureClass::InvalidResponse
        );
        assert!(
            matches!(second.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
        );
        handle.join().unwrap();
    }

    #[test]
    fn cohere_adapter_rejects_body_larger_than_one_mibibyte() {
        let (port, _, handle) = mock_server(response(&vec![b'x'; MAX_RERANK_RESPONSE_BYTES + 1]));
        let reranker = CohereReranker::new_for_test(config(Duration::from_secs(2)), port);
        assert_eq!(
            reranker.rerank("query", &documents()).unwrap_err().class(),
            RerankFailureClass::InvalidResponse
        );
        handle.join().unwrap();
    }

    #[test]
    fn cohere_adapter_timeout_uses_configured_global_deadline() {
        let (port, _, handle) = mock_server(MockResponse {
            status: 200,
            headers: vec![],
            body: br#"{"results":[]}"#.to_vec(),
            delay: Some(Duration::from_millis(250)),
        });
        let reranker = CohereReranker::new_for_test(config(Duration::from_millis(100)), port);
        assert_eq!(
            reranker.rerank("query", &documents()).unwrap_err().class(),
            RerankFailureClass::Timeout
        );
        handle.join().unwrap();
    }

    #[test]
    fn cohere_adapter_connection_failure_is_network() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            drop(stream);
        });
        let reranker = CohereReranker::new_for_test(config(Duration::from_secs(2)), port);
        assert_eq!(
            reranker.rerank("query", &documents()).unwrap_err().class(),
            RerankFailureClass::Network
        );
        handle.join().unwrap();
    }

    #[test]
    fn cohere_adapter_malformed_or_missing_results_is_invalid_response() {
        for body in [br#"{"#.as_slice(), br#"{}"#.as_slice()] {
            let (port, _, handle) = mock_server(response(body));
            let reranker = CohereReranker::new_for_test(config(Duration::from_secs(2)), port);
            assert_eq!(
                reranker.rerank("query", &documents()).unwrap_err().class(),
                RerankFailureClass::InvalidResponse
            );
            handle.join().unwrap();
        }
    }

    #[test]
    fn disabled_config_does_not_read_an_unrelated_cohere_key() {
        let config = parse_rerank_config_with(|name| {
            assert_ne!(name, COHERE_API_KEY_ENV, "disabled mode read the key");
            None
        })
        .unwrap();
        assert_eq!(config, RerankConfig::Disabled);

        assert_eq!(
            parse(&[(COHERE_API_KEY_ENV, "customer-secret")]).unwrap(),
            RerankConfig::Disabled
        );
    }

    #[test]
    fn orphaned_nowdocs_settings_require_explicit_provider() {
        assert_eq!(
            parse(&[(MODEL_ENV, "rerank-v4.0-fast")]),
            Err(RerankConfigError::ProviderRequired { setting: MODEL_ENV })
        );
        assert_eq!(
            parse(&[(TIMEOUT_ENV, "2000")]),
            Err(RerankConfigError::ProviderRequired {
                setting: TIMEOUT_ENV
            })
        );
        assert_eq!(
            parse(&[(MODEL_ENV, "rerank-v4.0-fast"), (TIMEOUT_ENV, "2000")]),
            Err(RerankConfigError::ProviderRequired { setting: MODEL_ENV })
        );
    }

    #[test]
    fn cohere_opt_in_requires_exact_provider_model_and_key() {
        assert_eq!(
            parse(&[(PROVIDER_ENV, "COHERE")]),
            Err(RerankConfigError::UnsupportedProvider)
        );
        assert_eq!(
            parse(&[(PROVIDER_ENV, "voyage")]),
            Err(RerankConfigError::UnsupportedProvider)
        );
        assert_eq!(
            parse(&[(PROVIDER_ENV, "   ")]),
            Err(RerankConfigError::UnsupportedProvider)
        );
        assert_eq!(
            parse(&[(PROVIDER_ENV, "cohere")]),
            Err(RerankConfigError::MissingModel)
        );
        assert_eq!(
            parse(&[(PROVIDER_ENV, "cohere"), (MODEL_ENV, "rerank-v4.0-fast")]),
            Err(RerankConfigError::MissingApiKey)
        );
        assert_eq!(
            parse(&[
                (PROVIDER_ENV, "cohere"),
                (MODEL_ENV, "   "),
                (COHERE_API_KEY_ENV, "key"),
            ]),
            Err(RerankConfigError::MissingModel)
        );
        assert_eq!(
            parse(&[
                (PROVIDER_ENV, "cohere"),
                (MODEL_ENV, "model"),
                (COHERE_API_KEY_ENV, "   "),
            ]),
            Err(RerankConfigError::MissingApiKey)
        );
    }

    #[test]
    fn complete_cohere_config_trims_values_and_uses_default_timeout() {
        let config = parse(&[
            (PROVIDER_ENV, "  cohere  "),
            (MODEL_ENV, "  rerank-v4.0-fast  "),
            (COHERE_API_KEY_ENV, "  customer-secret  "),
        ])
        .unwrap();

        let RerankConfig::Cohere(config) = config else {
            panic!("expected Cohere configuration");
        };
        assert_eq!(config.model, "rerank-v4.0-fast");
        assert_eq!(config.api_key.expose_secret(), "customer-secret");
        assert_eq!(config.timeout, Duration::from_millis(2000));
    }

    #[test]
    fn timeout_accepts_only_the_inclusive_contract_range() {
        for accepted in ["100", "10000"] {
            let config = parse(&[
                (PROVIDER_ENV, "cohere"),
                (MODEL_ENV, "model"),
                (COHERE_API_KEY_ENV, "key"),
                (TIMEOUT_ENV, accepted),
            ])
            .unwrap();
            let RerankConfig::Cohere(config) = config else {
                panic!("expected Cohere configuration");
            };
            assert_eq!(config.timeout.as_millis().to_string(), accepted);
        }

        for rejected in ["99", "10001", "two-seconds", "-1"] {
            assert_eq!(
                parse(&[
                    (PROVIDER_ENV, "cohere"),
                    (MODEL_ENV, "model"),
                    (COHERE_API_KEY_ENV, "key"),
                    (TIMEOUT_ENV, rejected),
                ]),
                Err(RerankConfigError::InvalidTimeout)
            );
        }
    }

    #[test]
    fn configuration_debug_and_errors_never_expose_values() {
        let config = parse(&[
            (PROVIDER_ENV, "cohere"),
            (MODEL_ENV, "model"),
            (COHERE_API_KEY_ENV, "customer-secret"),
        ])
        .unwrap();
        let debug = format!("{config:?}");
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("customer-secret"));

        let error = parse(&[(PROVIDER_ENV, "customer-secret")]).unwrap_err();
        assert!(!format!("{error:?}").contains("customer-secret"));
        assert!(!error.to_string().contains("customer-secret"));
    }

    #[test]
    fn runtime_failure_is_class_only_and_redacted() {
        let expected = [
            (RerankFailureClass::Network, "network"),
            (RerankFailureClass::Timeout, "timeout"),
            (RerankFailureClass::Authentication, "authentication"),
            (RerankFailureClass::InvalidRequest, "invalid_request"),
            (RerankFailureClass::RateLimit, "rate_limit"),
            (RerankFailureClass::ServerError, "server_error"),
            (RerankFailureClass::InvalidResponse, "invalid_response"),
        ];

        for (class, label) in expected {
            let failure = RerankFailure::new(class);
            assert_eq!(failure.class(), class);
            assert_eq!(failure.to_string(), label);
            assert!(!format!("{failure:?}").contains("query"));
        }
    }

    #[test]
    fn reranker_contract_is_object_safe_send_sync_and_returns_stable_ids() {
        struct Reverse;
        impl Reranker for Reverse {
            fn rerank(
                &self,
                _query: &str,
                documents: &[RerankDocument],
            ) -> Result<Vec<u32>, RerankFailure> {
                Ok(documents.iter().rev().map(|doc| doc.stable_id).collect())
            }
        }

        fn invoke(reranker: &(dyn Reranker + Send + Sync)) -> Vec<u32> {
            reranker
                .rerank(
                    "query",
                    &[
                        RerankDocument {
                            stable_id: 7,
                            document: "first".to_owned(),
                        },
                        RerankDocument {
                            stable_id: 9,
                            document: "second".to_owned(),
                        },
                    ],
                )
                .unwrap()
        }

        assert_eq!(invoke(&Reverse), vec![9, 7]);
    }

    #[cfg(unix)]
    #[test]
    fn non_unicode_values_fail_without_echoing_bytes() {
        use std::os::unix::ffi::OsStringExt;

        for invalid_setting in [PROVIDER_ENV, MODEL_ENV, TIMEOUT_ENV, COHERE_API_KEY_ENV] {
            let error = parse_rerank_config_with(|name| match name {
                _ if name == invalid_setting => Some(OsString::from_vec(vec![0xff])),
                PROVIDER_ENV => Some(OsString::from("cohere")),
                MODEL_ENV => Some(OsString::from("model")),
                TIMEOUT_ENV => None,
                COHERE_API_KEY_ENV => Some(OsString::from("key")),
                _ => None,
            })
            .unwrap_err();
            assert_eq!(
                error,
                RerankConfigError::NonUnicode {
                    setting: invalid_setting
                }
            );
            assert!(!error.to_string().contains("255"));
        }
    }

    #[test]
    fn configured_reranker_factory_performs_no_network() {
        // Injected environment: Disabled → Ok(None) with zero network.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let result = super::configured_reranker_with(|_| None).unwrap();
        assert!(result.is_none());
        assert!(
            matches!(listener.accept(), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock),
            "factory must not connect for Disabled config"
        );

        // Cohere → Ok(Some) with zero network (no request until rerank()).
        let result = super::configured_reranker_with(|name| match name {
            PROVIDER_ENV => Some(OsString::from("cohere")),
            MODEL_ENV => Some(OsString::from("rerank-v4.0-fast")),
            COHERE_API_KEY_ENV => Some(OsString::from("test-key")),
            _ => None,
        })
        .unwrap();
        assert!(result.is_some());
        assert!(
            matches!(listener.accept(), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock),
            "factory must not connect for Cohere config"
        );
    }

    #[test]
    fn configured_reranker_fails_serve_and_smoke_before_search() {
        // Invalid opt-in → Err (partial config fails startup).
        for (entries, expected) in [
            (
                vec![(PROVIDER_ENV, "cohere")],
                RerankConfigError::MissingModel,
            ),
            (
                vec![(PROVIDER_ENV, "cohere"), (MODEL_ENV, "model")],
                RerankConfigError::MissingApiKey,
            ),
        ] {
            let result = super::configured_reranker_with(|name| {
                entries
                    .iter()
                    .find(|(k, _)| *k == name)
                    .map(|(_, v)| OsString::from(*v))
            });
            match result {
                Ok(_) => panic!("expected error {expected:?}, got Ok"),
                Err(e) => assert_eq!(e, expected),
            }
        }
    }
}
