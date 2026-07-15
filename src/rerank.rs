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
pub(crate) struct RerankDocument {
    pub(crate) stable_id: u32,
    pub(crate) document: String,
}

pub(crate) trait Reranker: Send + Sync {
    fn rerank(&self, query: &str, documents: &[RerankDocument]) -> Result<Vec<u32>, RerankFailure>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RerankFailureClass {
    Network,
    Timeout,
    Authentication,
    InvalidRequest,
    RateLimit,
    ServerError,
    InvalidResponse,
}

impl RerankFailureClass {
    pub(crate) const fn label(self) -> &'static str {
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
pub(crate) struct RerankFailure {
    class: RerankFailureClass,
}

impl RerankFailure {
    pub(crate) const fn new(class: RerankFailureClass) -> Self {
        Self { class }
    }

    pub(crate) const fn class(&self) -> RerankFailureClass {
        self.class
    }
}

impl fmt::Display for RerankFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.class.label())
    }
}

impl std::error::Error for RerankFailure {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::time::Duration;

    fn parse(entries: &[(&str, &str)]) -> Result<RerankConfig, RerankConfigError> {
        let values: HashMap<String, OsString> = entries
            .iter()
            .map(|(key, value)| ((*key).to_owned(), OsString::from(*value)))
            .collect();
        parse_rerank_config_with(|name| values.get(name).cloned())
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
}
