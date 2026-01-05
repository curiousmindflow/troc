mod change_from_writer;
mod historycache;
mod reader;

pub use change_from_writer::{ChangeFromWriter, ChangeFromWriterMap};
pub use historycache::ReaderHistoryCache;
pub use reader::{Reader, ReaderBuilder, ReaderConfiguration};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum SampleStateKind {
    #[default]
    NotRead,
    Read,
    Any,
}
