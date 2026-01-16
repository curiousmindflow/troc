use std::fmt::Display;

use binrw::binrw;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u16)]
#[binrw]
#[br(import(_len: usize))]
#[br(map = protov_read_map)]
#[bw(map = protov_write_map)]
pub enum ProtocolVersion {
    #[default]
    #[brw(magic = 0x02_04u16)]
    V2_4,
    #[brw(magic = 0x02_02u16)]
    V2_2,
    #[brw(magic = 0x02_01u16)]
    V2_1,
    #[brw(magic = 0x02_00u16)]
    V2_0,
    #[brw(magic = 0x01_01u16)]
    V1_1,
    #[brw(magic = 0x01_00u16)]
    V1_0,
    #[brw(magic = 0x00_00u16)]
    Unknown,
}

fn protov_read_map(value: [u8; 2]) -> ProtocolVersion {
    match value {
        [0x02, 0x04] => ProtocolVersion::V2_4,
        [0x02, 0x02] => ProtocolVersion::V2_2,
        [0x02, 0x01] => ProtocolVersion::V2_1,
        [0x02, 0x00] => ProtocolVersion::V2_0,
        [0x01, 0x01] => ProtocolVersion::V1_1,
        [0x01, 0x00] => ProtocolVersion::V1_0,
        _ => ProtocolVersion::Unknown,
    }
}

fn protov_write_map(value: &ProtocolVersion) -> [u8; 2] {
    match value {
        ProtocolVersion::V2_4 => [0x02, 0x04],
        ProtocolVersion::V2_2 => [0x02, 0x02],
        ProtocolVersion::V2_1 => [0x02, 0x01],
        ProtocolVersion::V2_0 => [0x02, 0x00],
        ProtocolVersion::V1_1 => [0x01, 0x01],
        ProtocolVersion::V1_0 => [0x01, 0x00],
        ProtocolVersion::Unknown => [0x00, 0x00],
    }
}

impl Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolVersion::Unknown => f.write_str("ProtocolVersion::Unknown")?,
            ProtocolVersion::V1_0 => f.write_str("ProtocolVersion::ProtocolVersion1_0")?,
            ProtocolVersion::V1_1 => f.write_str("ProtocolVersion::ProtocolVersion1_1")?,
            ProtocolVersion::V2_0 => f.write_str("ProtocolVersion::ProtocolVersion2_0")?,
            ProtocolVersion::V2_1 => f.write_str("ProtocolVersion::ProtocolVersion2_1")?,
            ProtocolVersion::V2_2 => f.write_str("ProtocolVersion::ProtocolVersion2_2")?,
            ProtocolVersion::V2_4 => f.write_str("ProtocolVersion::ProtocolVersion2_4")?,
        };
        Ok(())
    }
}
