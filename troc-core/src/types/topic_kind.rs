use std::fmt::Display;

#[derive(Debug, Clone, Copy, Default)]
pub enum TopicKind {
    #[default]
    NoKey,
    WithKey,
}

impl Display for TopicKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopicKind::NoKey => f.write_str("NoKey")?,
            TopicKind::WithKey => f.write_str("WithKey")?,
        };
        Ok(())
    }
}
