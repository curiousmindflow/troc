#![allow(dead_code)]
use std::{error::Error, fmt::Display};

use chrono::{DateTime, Local, NaiveDateTime, NaiveTime};
use regex::Regex;
use serde::{Deserialize, Serialize};
use troc::{DDSType, Guid, KeyCalculationError, Keyed, cdr};
impl std::error::Error for SizeParseError {}

use tracing_subscriber::EnvFilter;

pub static O: u64 = 1;
pub static K: u64 = 1024 * O;
pub static M: u64 = 1024 * K;
pub static G: u64 = 1024 * M;

/// The size of the struct is 28 + payload.len()
/// 8 octets for time_ns
/// 4 octets for time_ns
/// 8 octets for the size of payload + payload.len()
/// 16 octests for writer_guid
#[derive(Debug, Default, Serialize, Deserialize, Clone, DDSType)]
pub struct Message {
    time_s: i64,
    time_ns: u32,
    payload: Vec<u8>,
    writer_guid: String,
    id: u64,
    #[key]
    key: u64,
}

impl Message {
    pub fn new(writer_guid: Guid, data: &[u8], id: u64, key: u64) -> Self {
        Self {
            time_s: Local::now().timestamp(),
            time_ns: Local::now().timestamp_subsec_nanos(),
            payload: data.to_vec(),
            writer_guid: writer_guid.to_string(),
            id,
            key,
        }
    }

    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }

    pub fn set_key(&mut self, key: u64) {
        self.key = key;
    }

    pub fn serialize(data: &Message) -> Vec<u8> {
        cdr::serialize::<_, _, cdr::CdrBe>(data, cdr::Infinite).unwrap()
    }

    pub fn deserialize(data: &[u8]) -> Self {
        cdr::deserialize(data).unwrap()
    }

    pub fn payload_size(&self) -> u64 {
        self.payload.len() as u64
    }

    pub fn datetime(&self) -> NaiveDateTime {
        DateTime::from_timestamp(self.time_s, self.time_ns)
            .unwrap()
            .naive_local()
    }

    pub fn time(&self) -> NaiveTime {
        self.datetime().time()
    }

    pub fn writer_guid(&self) -> String {
        self.writer_guid.clone()
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn key(&self) -> u64 {
        self.key
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, DDSType)]
pub struct Shape {
    #[key]
    pub color: String,
    pub x: i32,
    pub y: i32,
    pub shapesize: i32,
}

pub fn parse_size(input: &str) -> Result<u64, SizeParseError> {
    let regex = Regex::new(r"(?<value>\d+)(?<unit>[o,O,k,K,m,M,g,G]{0,1})").unwrap();

    let captures = regex.captures(input).ok_or(SizeParseError::new(&format!(
        "The format of the size is incorrect: {}",
        input
    )))?;

    let value_str = captures.name("value").unwrap().as_str();
    // TODO: find out why when input = "1" 'unit' group is a match... captures.name("unit") return Some(_)
    let unit_str = captures.name("unit").map_or('o', |m| {
        let str = m.as_str();
        if str.is_empty() {
            'o'
        } else {
            str.chars().next().unwrap()
        }
    });

    let value = value_str
        .parse::<u64>()
        .map_err(SizeParseError::from_error)?;

    let unit = match unit_str {
        'o' | 'O' => O,
        'k' | 'K' => K,
        'm' | 'M' => M,
        'g' | 'G' => G,
        _ => unreachable!(),
    };

    let result = value * unit;
    Ok(result)
}

#[derive(Debug)]
pub struct SizeParseError {
    input: String,
}

impl SizeParseError {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
        }
    }

    pub fn from_error(e: impl Error) -> Self {
        Self {
            input: e.to_string(),
        }
    }
}

impl Display for SizeParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "Parsing of size failed, the input was incorrect: {}",
            self.input
        ))
    }
}

pub struct OtlParam {
    pub service_name: &'static str,
}

pub fn set_up_log(_otl: Option<OtlParam>) {
    let mut base_filter = EnvFilter::new("warn,neli=error");

    if let Ok(env_filter) = std::env::var("RUST_LOG") {
        for directive in env_filter.split(',') {
            if let Ok(parsed) = directive.parse() {
                base_filter = base_filter.add_directive(parsed);
            }
        }
    }

    tracing_subscriber::fmt()
        .with_env_filter(base_filter)
        .init();

    // let stdout_layer = tracing_subscriber::fmt::layer()
    //     .compact()
    //     .with_line_number(true)
    //     .with_target(true);

    // let registry = tracing_subscriber::registry()
    //     .with(filter_layer)
    //     .with(stdout_layer);

    // if let Some(otl) = otl {
    //     global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());

    //     let tracer = opentelemetry_jaeger::new_pipeline()
    //         .with_service_name(otl.service_name)
    //         .install_simple()
    //         .unwrap();

    //     let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    //     registry.with(opentelemetry).try_init().unwrap()
    // } else {
    //     registry.try_init().unwrap();
    // }
}
