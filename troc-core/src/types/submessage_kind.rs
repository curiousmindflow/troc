use std::fmt::Display;

use binrw::binrw;

// 9.4.5.1.1
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[binrw]
#[brw(repr(u8))]
pub enum SubmessageKind {
    #[default]
    RtpsHe = 0x00,
    Data = 0x15,
    Gap = 0x08,
    Heartbeat = 0x07,
    AckNack = 0x06,
    Pad = 0x01,
    InfoTs = 0x09,
    InfoReply = 0x0f,
    InfoReplyIp4 = 0x0d,
    InfoDst = 0x0e,
    InfoSrc = 0x0c,
    DataFrag = 0x16,
    NackFrag = 0x12,
    HeartbeatFrag = 0x13,
    Unknown = 0x80,
}

impl Display for SubmessageKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubmessageKind::RtpsHe => f.write_str("SubmessageKind::RtpsHe")?,
            SubmessageKind::Data => f.write_str("SubmessageKind::Data")?,
            SubmessageKind::Gap => f.write_str("SubmessageKind::Gap")?,
            SubmessageKind::Heartbeat => f.write_str("SubmessageKind::Heartbeat")?,
            SubmessageKind::AckNack => f.write_str("SubmessageKind::AckNack")?,
            SubmessageKind::Pad => f.write_str("SubmessageKind::Pad")?,
            SubmessageKind::InfoTs => f.write_str("SubmessageKind::InfoTs")?,
            SubmessageKind::InfoReplyIp4 => f.write_str("SubmessageKind::InfoReplyIp4")?,
            SubmessageKind::InfoReply => f.write_str("SubmessageKind::InfoReply")?,
            SubmessageKind::InfoDst => f.write_str("SubmessageKind::InfoDst")?,
            SubmessageKind::InfoSrc => f.write_str("SubmessageKind::InfoSrc")?,
            SubmessageKind::DataFrag => f.write_str("SubmessageKind::DataFrag")?,
            SubmessageKind::NackFrag => f.write_str("SubmessageKind::NackFrag")?,
            SubmessageKind::HeartbeatFrag => f.write_str("SubmessageKind::HeartbeatFrag")?,
            SubmessageKind::Unknown => f.write_str("SubmessageKind::Unknown")?,
        };
        Ok(())
    }
}

impl From<u8> for SubmessageKind {
    fn from(value: u8) -> Self {
        match value {
            0x00 => SubmessageKind::RtpsHe,
            0x01 => SubmessageKind::Pad,
            0x06 => SubmessageKind::AckNack,
            0x07 => SubmessageKind::Heartbeat,
            0x08 => SubmessageKind::Gap,
            0x09 => SubmessageKind::InfoTs,
            0x0c => SubmessageKind::InfoSrc,
            0x0d => SubmessageKind::InfoReplyIp4,
            0x0e => SubmessageKind::InfoDst,
            0x0f => SubmessageKind::InfoReply,
            0x12 => SubmessageKind::NackFrag,
            0x13 => SubmessageKind::HeartbeatFrag,
            0x15 => SubmessageKind::Data,
            0x16 => SubmessageKind::DataFrag,
            0xFF => SubmessageKind::Unknown,
            _ => unreachable!(),
        }
    }
}

impl From<SubmessageKind> for u8 {
    fn from(value: SubmessageKind) -> Self {
        match value {
            SubmessageKind::RtpsHe => 0x00,
            SubmessageKind::Pad => 0x01,
            SubmessageKind::AckNack => 0x06,
            SubmessageKind::Heartbeat => 0x07,
            SubmessageKind::Gap => 0x08,
            SubmessageKind::InfoTs => 0x09,
            SubmessageKind::InfoSrc => 0x0c,
            SubmessageKind::InfoReplyIp4 => 0x0d,
            SubmessageKind::InfoDst => 0x0e,
            SubmessageKind::InfoReply => 0x0f,
            SubmessageKind::NackFrag => 0x12,
            SubmessageKind::HeartbeatFrag => 0x13,
            SubmessageKind::Data => 0x15,
            SubmessageKind::DataFrag => 0x16,
            SubmessageKind::Unknown => 0xFF,
        }
    }
}
