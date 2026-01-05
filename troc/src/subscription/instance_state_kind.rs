#[derive(Debug, Default, Clone, Copy)]
pub enum InstanceStateKind {
    #[default]
    Alive,
    NotAliveDisposed,
    NotAliveNoWriters,
}
