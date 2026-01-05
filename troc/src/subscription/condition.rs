#[derive(Debug, Default, Clone, Copy)]
pub enum ReadCondition {
    #[default]
    NotRead,
    Read,
    All,
}
