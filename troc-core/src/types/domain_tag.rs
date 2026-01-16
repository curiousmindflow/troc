use std::{fmt::Display, str::FromStr};

use binrw::{BinResult, Endian, binrw};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialOrd, Ord)]
#[binrw]
#[br(import(len: usize))]
pub struct DomainTag {
    #[br(parse_with = DomainTag::custom_parser, args((len,)))]
    #[bw(write_with = DomainTag::custom_writer)]
    value: String,
}

impl DomainTag {
    pub fn new(tag: &str) -> Self {
        DomainTag {
            value: tag.trim().to_string(),
        }
    }

    #[binrw::parser(reader, endian)]
    fn custom_parser(args: (usize,)) -> BinResult<String> {
        let mut buf = vec![0u8; args.0];
        reader.read_exact(&mut buf)?;

        let string_slice: [u8; 4] = buf[..4].try_into().unwrap();

        let string_len = match endian {
            Endian::Little => u32::from_le_bytes(string_slice),
            Endian::Big => u32::from_be_bytes(string_slice),
        };

        let str_slice = &buf[4..4 + string_len as usize - 1];
        let domain_tag_value = String::from_utf8_lossy(str_slice).to_string();
        Ok(domain_tag_value)
    }

    #[allow(clippy::ptr_arg)]
    #[binrw::writer(writer, endian)]
    fn custom_writer(value: &String) -> BinResult<()> {
        let mut value_bytes = value.to_owned().into_bytes();
        value_bytes.append(&mut vec![0]);

        let mut value = match endian {
            Endian::Little => (value_bytes.len() as u32).to_le_bytes().to_vec(),
            Endian::Big => (value_bytes.len() as u32).to_be_bytes().to_vec(),
        };

        value.append(&mut value_bytes);

        let _ = writer.write(&value);
        Ok(())
    }
}

impl FromStr for DomainTag {
    type Err = std::fmt::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(DomainTag::new(s))
    }
}

impl AsRef<str> for DomainTag {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

impl Display for DomainTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}:{:?}", &self.value, &self.value.as_bytes()))
    }
}

impl PartialEq for DomainTag {
    fn eq(&self, other: &Self) -> bool {
        self.value.as_bytes() == other.value.as_bytes()
    }
}

impl Eq for DomainTag {}
