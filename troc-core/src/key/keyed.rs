use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Error)]
pub struct KeyCalculationError {
    msg: String,
    #[source]
    source: anyhow::Error,
}

impl KeyCalculationError {
    pub fn new(msg: &str, source: anyhow::Error) -> Self {
        Self {
            msg: msg.to_string(),
            source,
        }
    }
}

impl Display for KeyCalculationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)?;
        f.write_str(&format!("Source: {}", self.source))?;
        Ok(())
    }
}

pub trait Keyed: Send + Sync {
    /// Calculate and get the key hash of this [`PlugeableMessage`]
    fn key(&self) -> Result<[u8; 16], KeyCalculationError>;
}
