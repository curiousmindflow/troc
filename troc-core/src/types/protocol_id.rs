use binrw::binrw;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[binrw]
pub struct ProtocolId(pub(crate) [u8; 4]);

impl Default for ProtocolId {
    fn default() -> Self {
        ProtocolId([b'R', b'T', b'P', b'S'])
    }
}
