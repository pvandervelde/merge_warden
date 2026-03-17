// See docs/spec/interfaces/server-config.md for the full contract.

/// All errors that can occur during server startup or event processing.
///
/// See `docs/spec/interfaces/server-config.md` for detailed documentation of
/// each variant, including which environment variable produces which error.
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    /// A required environment variable was absent.
    ///
    /// Parameter: the variable name that was missing.
    #[error("Missing required environment variable '{0}'")]
    MissingEnvVar(String),

    /// An environment variable was present but its value was not valid.
    #[error("Invalid environment variable '{name}': {message}")]
    InvalidEnvVar { name: String, message: String },

    /// The TOML configuration file was found but could not be parsed.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// GitHub authentication or client initialisation failed.
    #[error("GitHub authentication error: {0}")]
    AuthError(String),

    /// TCP bind, listener creation, or `axum::serve` failure at startup.
    #[error("Server startup error: {0}")]
    StartupError(String),

    /// An error that occurred while processing an individual webhook event.
    ///
    /// Distinct from `AuthError` so alerting rules can distinguish
    /// "GitHub auth broken" from "bad payload on one event".
    #[error("Request processing error: {0}")]
    ProcessingError(String),

    /// The `tracing` subscriber could not be installed.
    #[error("Telemetry initialization failed: {0}")]
    TelemetryInitFailed(String),

    /// Propagated from the ingress layer.
    #[error("Ingress error: {source}")]
    IngressError {
        #[from]
        source: crate::ingress::IngressError,
    },
}
