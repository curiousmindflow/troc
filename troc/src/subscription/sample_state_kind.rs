#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum SampleStateKind {
    #[default]
    NotRead,
    Read,
}
