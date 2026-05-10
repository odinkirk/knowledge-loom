use thiserror::Error;

/// Errors that can occur during embedding operations
#[derive(Error, Debug)]
pub enum EmbedError {
    /// Error occurred while downloading the model
    #[error("Failed to download model: {0}")]
    ModelDownloadError(String),

    /// Error occurred while loading the model
    #[error("Failed to load model: {0}")]
    ModelLoadError(String),

    /// Error occurred during embedding generation
    #[error("Failed to generate embedding: {0}")]
    EmbeddingError(String),

    /// Network error occurred (for external providers)
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Configuration error (invalid settings, missing environment variables, etc.)
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Dimension mismatch between providers
    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    /// Timeout occurred during operation
    #[error("Operation timed out after {timeout_secs} seconds")]
    Timeout { timeout_secs: u64 },

    /// HTTP error occurred (for external providers)
    #[error("HTTP error: {status} - {message}")]
    HttpError { status: u16, message: String },

    /// Invalid response format from external provider
    #[error("Invalid response format: {0}")]
    InvalidResponseFormat(String),

    /// Authentication error (for external providers)
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    /// Model integrity check failed
    #[error("Model integrity check failed: {0}")]
    ModelIntegrityError(String),

    /// Model file not found
    #[error("Model file not found: {0}")]
    ModelNotFoundError(String),

    /// Unsupported model type
    #[error("Unsupported model type: {0}")]
    UnsupportedModelError(String),

    /// IO error occurred
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Generic error with custom message
    #[error("{0}")]
    Custom(String),
}

impl EmbedError {
    /// Create a model download error
    pub fn model_download_error(msg: impl Into<String>) -> Self {
        Self::ModelDownloadError(msg.into())
    }

    /// Create a model load error
    pub fn model_load_error(msg: impl Into<String>) -> Self {
        Self::ModelLoadError(msg.into())
    }

    /// Create an embedding generation error
    pub fn embedding_error(msg: impl Into<String>) -> Self {
        Self::EmbeddingError(msg.into())
    }

    /// Create a network error
    pub fn network_error(msg: impl Into<String>) -> Self {
        Self::NetworkError(msg.into())
    }

    /// Create a configuration error
    pub fn configuration_error(msg: impl Into<String>) -> Self {
        Self::ConfigurationError(msg.into())
    }

    /// Create a dimension mismatch error
    #[allow(dead_code)]
    #[must_use]
    pub fn dimension_mismatch(expected: usize, actual: usize) -> Self {
        Self::DimensionMismatch { expected, actual }
    }

    /// Create a timeout error
    pub fn timeout(timeout_secs: u64) -> Self {
        Self::Timeout { timeout_secs }
    }

    /// Create an HTTP error
    pub fn http_error(status: u16, message: impl Into<String>) -> Self {
        Self::HttpError {
            status,
            message: message.into(),
        }
    }

    /// Create an invalid response format error
    pub fn invalid_response_format(msg: impl Into<String>) -> Self {
        Self::InvalidResponseFormat(msg.into())
    }

    /// Create an authentication error
    pub fn authentication_error(msg: impl Into<String>) -> Self {
        Self::AuthenticationError(msg.into())
    }

    /// Create a model integrity error
    pub fn model_integrity_error(msg: impl Into<String>) -> Self {
        Self::ModelIntegrityError(msg.into())
    }

    /// Create a model not found error
    pub fn model_not_found_error(msg: impl Into<String>) -> Self {
        Self::ModelNotFoundError(msg.into())
    }

    /// Create an unsupported model error
    pub fn unsupported_model_error(msg: impl Into<String>) -> Self {
        Self::UnsupportedModelError(msg.into())
    }

    /// Create a custom error
    pub fn custom(msg: impl Into<String>) -> Self {
        Self::Custom(msg.into())
    }
}

/// Result type for embedding operations
pub type Result<T> = std::result::Result<T, EmbedError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = EmbedError::model_download_error("test");
        assert!(matches!(err, EmbedError::ModelDownloadError(_)));

        let err = EmbedError::model_load_error("test");
        assert!(matches!(err, EmbedError::ModelLoadError(_)));

        let err = EmbedError::embedding_error("test");
        assert!(matches!(err, EmbedError::EmbeddingError(_)));

        let err = EmbedError::network_error("test");
        assert!(matches!(err, EmbedError::NetworkError(_)));

        let err = EmbedError::configuration_error("test");
        assert!(matches!(err, EmbedError::ConfigurationError(_)));

        let err = EmbedError::dimension_mismatch(384, 512);
        assert!(matches!(
            err,
            EmbedError::DimensionMismatch {
                expected: 384,
                actual: 512
            }
        ));

        let err = EmbedError::timeout(5);
        assert!(matches!(err, EmbedError::Timeout { timeout_secs: 5 }));

        let err = EmbedError::http_error(404, "Not Found");
        assert!(matches!(err, EmbedError::HttpError { status: 404, .. }));

        let err = EmbedError::invalid_response_format("test");
        assert!(matches!(err, EmbedError::InvalidResponseFormat(_)));

        let err = EmbedError::authentication_error("test");
        assert!(matches!(err, EmbedError::AuthenticationError(_)));

        let err = EmbedError::model_integrity_error("test");
        assert!(matches!(err, EmbedError::ModelIntegrityError(_)));

        let err = EmbedError::model_not_found_error("test");
        assert!(matches!(err, EmbedError::ModelNotFoundError(_)));

        let err = EmbedError::unsupported_model_error("test");
        assert!(matches!(err, EmbedError::UnsupportedModelError(_)));

        let err = EmbedError::custom("test");
        assert!(matches!(err, EmbedError::Custom(_)));
    }

    #[test]
    fn test_error_display() {
        let err = EmbedError::model_download_error("test message");
        assert_eq!(err.to_string(), "Failed to download model: test message");

        let err = EmbedError::dimension_mismatch(384, 512);
        assert_eq!(err.to_string(), "Dimension mismatch: expected 384, got 512");

        let err = EmbedError::timeout(5);
        assert_eq!(err.to_string(), "Operation timed out after 5 seconds");

        let err = EmbedError::http_error(404, "Not Found");
        assert_eq!(err.to_string(), "HTTP error: 404 - Not Found");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let embed_err = EmbedError::from(io_err);
        assert!(matches!(embed_err, EmbedError::IoError(_)));
    }

    #[test]
    fn test_error_from_json() {
        // Create a JSON error by trying to parse invalid JSON
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let embed_err = EmbedError::from(json_err);
        assert!(matches!(embed_err, EmbedError::JsonError(_)));
    }
}
