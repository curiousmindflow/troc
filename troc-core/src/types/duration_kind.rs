use std::time::Duration;

#[derive(Debug, Default)]
pub enum DurationKind {
    #[default]
    Infinite,
    Finite(Duration),
}
