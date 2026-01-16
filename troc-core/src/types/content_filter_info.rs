#![allow(dead_code)]

use super::{filter_result::FilterResult, filter_signature_sequence::FilterSignatureSequence};

#[derive(Debug, Default, Clone)]
pub struct ContentFilterInfo {
    filter_result: FilterResult,
    filter_signatures: FilterSignatureSequence,
}
