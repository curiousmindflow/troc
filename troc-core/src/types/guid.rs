use std::fmt::{Debug, Display, Write};

use binrw::binrw;
use pretty_hex::PrettyHex;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::PRETTY_HEX_CONFIG;

use super::vendor_id::VendorId;

pub static GUID_UNKOWN: Guid = Guid {
    guid_prefix: GUIDPREFIX_UNKWOWN,
    entity_id: ENTITYID_UNKOWN,
};

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[binrw]
#[br(import(_len: usize))]
pub struct Guid {
    guid_prefix: GuidPrefix,
    entity_id: EntityId,
}

impl Guid {
    pub fn new(guid_prefix: GuidPrefix, entity_id: EntityId) -> Self {
        Self {
            guid_prefix,
            entity_id,
        }
    }

    pub fn generate(vendor_id: VendorId, entity_id: EntityId) -> Self {
        Self {
            guid_prefix: GuidPrefix::new(vendor_id),
            entity_id,
        }
    }

    pub fn get_guid_prefix(&self) -> GuidPrefix {
        self.guid_prefix
    }

    pub fn get_entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn as_bytes(&self) -> [u8; 16] {
        let guid_prefix = self.guid_prefix.0.as_slice();
        let entity_id_entity_key = self.entity_id.entity_key.as_slice();
        let entity_id_entity_kind = &[self.entity_id.entity_kind];
        let bytes: [u8; 16] = [guid_prefix, entity_id_entity_key, entity_id_entity_kind]
            .concat()
            .try_into()
            .unwrap();
        bytes
    }

    pub fn role(&self) -> &'static str {
        match self.get_entity_id() {
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR => "participant_reader",
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR => "publications_reader",
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR => "subscriptions_reader",
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER => "participant_writer",
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER => "publications_writer",
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER => "subscriptions_writer",
            _ => "applicative",
        }
    }
}

impl Display for Guid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}.{}", self.guid_prefix, self.entity_id))?;
        Ok(())
    }
}

impl Debug for Guid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}.{:?}", self.guid_prefix, self.entity_id))?;
        Ok(())
    }
}

pub static GUIDPREFIX_UNKWOWN: GuidPrefix = GuidPrefix([0; 12]);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[binrw]
pub struct GuidPrefix(pub [u8; 12]);

impl GuidPrefix {
    pub fn new(vendor_id: VendorId) -> Self {
        let mut guid_prefix = GuidPrefix(random());
        guid_prefix.0[0] = vendor_id.0[0];
        guid_prefix.0[1] = vendor_id.0[1];
        guid_prefix
    }

    pub const fn from(prefix: [u8; 12], vendor_id: VendorId) -> Self {
        let mut guid_prefix = GuidPrefix(prefix);
        guid_prefix.0[0] = vendor_id.0[0];
        guid_prefix.0[1] = vendor_id.0[1];
        guid_prefix
    }
}

impl Default for GuidPrefix {
    fn default() -> Self {
        GUIDPREFIX_UNKWOWN
    }
}

impl Display for GuidPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.0.hex_conf(PRETTY_HEX_CONFIG)))?;
        Ok(())
    }
}

impl Debug for GuidPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.0.hex_conf(PRETTY_HEX_CONFIG)))?;
        Ok(())
    }
}

pub const ENTITYID_UNKOWN: EntityId = EntityId {
    entity_key: [0x00, 0x00, 0x00],
    entity_kind: 0x00,
};

pub const ENTITYID_PARTICIPANT: EntityId = EntityId {
    entity_key: [0x00, 0x00, 0x01],
    entity_kind: 0xc1,
};

pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId = EntityId {
    entity_key: [0x00, 0x00, 0x03],
    entity_kind: 0xc2,
};

pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId = EntityId {
    entity_key: [0x00, 0x00, 0x03],
    entity_kind: 0xc7,
};

pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId = EntityId {
    entity_key: [0x00, 0x00, 0x04],
    entity_kind: 0xc2,
};

pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId = EntityId {
    entity_key: [0x00, 0x00, 0x04],
    entity_kind: 0xc7,
};

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER: EntityId = EntityId {
    entity_key: [0x00, 0x01, 0x00],
    entity_kind: 0xc2,
};

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR: EntityId = EntityId {
    entity_key: [0x00, 0x01, 0x00],
    entity_kind: 0xc7,
};

pub const ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER: EntityId = EntityId {
    entity_key: [0x00, 0x02, 0x00],
    entity_kind: 0xc2,
};

pub const ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_READER: EntityId = EntityId {
    entity_key: [0x00, 0x02, 0x00],
    entity_kind: 0xc7,
};

pub const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId = EntityId {
    entity_key: [0x00, 0x00, 0x02],
    entity_kind: 0xc2,
};

pub const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId = EntityId {
    entity_key: [0x00, 0x00, 0x02],
    entity_kind: 0xc7,
};

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[binrw]
#[br(import(_len: usize))]
pub struct EntityId {
    pub entity_key: [u8; 3],
    pub entity_kind: u8,
}

impl EntityId {
    pub fn new(entity_key: [u8; 3], entity_kind: u8) -> Self {
        Self {
            entity_key,
            entity_kind,
        }
    }

    pub fn unknown(entity_key: [u8; 3]) -> Self {
        Self {
            entity_key,
            entity_kind: 0x00,
        }
    }

    pub fn participant(entity_key: [u8; 3]) -> Self {
        Self {
            entity_key,
            entity_kind: 0x00,
        }
    }

    pub fn writer_with_key(entity_key: [u8; 3]) -> Self {
        Self {
            entity_key,
            entity_kind: 0x02,
        }
    }

    pub fn writer_no_key(entity_key: [u8; 3]) -> Self {
        Self {
            entity_key,
            entity_kind: 0x03,
        }
    }

    pub fn reader_no_key(entity_key: [u8; 3]) -> Self {
        Self {
            entity_key,
            entity_kind: 0x04,
        }
    }

    pub fn reader_with_key(entity_key: [u8; 3]) -> Self {
        Self {
            entity_key,
            entity_kind: 0x07,
        }
    }

    pub fn writer_group(entity_key: [u8; 3]) -> Self {
        Self {
            entity_key,
            entity_kind: 0x08,
        }
    }

    pub fn reader_group(entity_key: [u8; 3]) -> Self {
        Self {
            entity_key,
            entity_kind: 0x09,
        }
    }

    pub fn unknown_builtin(entity_key: [u8; 3]) -> Self {
        EntityId::unknown(entity_key).to_builtin()
    }

    pub fn participant_builtin(entity_key: [u8; 3]) -> Self {
        EntityId::participant(entity_key).to_builtin()
    }

    pub fn writer_with_key_builtin(entity_key: [u8; 3]) -> Self {
        EntityId::writer_with_key(entity_key).to_builtin()
    }

    pub fn writer_no_key_builtin(entity_key: [u8; 3]) -> Self {
        EntityId::writer_no_key(entity_key).to_builtin()
    }

    pub fn reader_no_key_builtin(entity_key: [u8; 3]) -> Self {
        EntityId::reader_no_key(entity_key).to_builtin()
    }

    pub fn reader_with_key_builtin(entity_key: [u8; 3]) -> Self {
        EntityId::reader_with_key(entity_key).to_builtin()
    }

    pub fn writer_group_builtin(entity_key: [u8; 3]) -> Self {
        EntityId::writer_group(entity_key).to_builtin()
    }

    pub fn reader_group_builtin(entity_key: [u8; 3]) -> Self {
        EntityId::reader_group(entity_key).to_builtin()
    }

    fn to_builtin(mut self) -> Self {
        self.entity_kind += 0xc0;
        self
    }
}

impl Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_repr = match *self {
            ENTITYID_UNKOWN => "ENTITYID_UNKOWN",
            ENTITYID_PARTICIPANT => "ENTITYID_PARTICIPANT",
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER => {
                "ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER"
            }
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR => {
                "ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR"
            }
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER => {
                "ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER"
            }
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR => {
                "ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR"
            }
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER => {
                "ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER"
            }
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR => {
                "ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR"
            }
            ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER => {
                "ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER"
            }
            ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_READER => {
                "ENTITYID_P2P_BUILTIN_PARTICIPANT_MESSAGE_READER"
            }
            ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER => "ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER",
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR => "ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR",
            id => {
                f.write_fmt(format_args!(
                    "{}.{}",
                    self.entity_key.hex_conf(PRETTY_HEX_CONFIG),
                    id.entity_kind
                ))?;
                return Ok(());
            }
        };
        f.write_str(str_repr)?;
        Ok(())
    }
}

impl Debug for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{}.{}",
            self.entity_key.hex_conf(PRETTY_HEX_CONFIG),
            self.entity_kind
        ))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
pub struct EntityKey(pub [u8; 3]);

impl EntityKey {
    pub const MAX: u32 = 2u32.pow(3 * 8);
    pub const MIN: u32 = 0;

    pub fn to_u32(self) -> u32 {
        let mut value_4_bytes = [0u8; 4];
        value_4_bytes[1..].copy_from_slice(&self.0);
        u32::from_ne_bytes(value_4_bytes)
    }

    pub fn from_u32(value: u32) -> Self {
        let value = value.to_ne_bytes();
        let mut value_3_bytes = [0u8; 3];
        value_3_bytes.copy_from_slice(&value[1..]);
        Self(value_3_bytes)
    }
}
