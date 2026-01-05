#![allow(dead_code)]

use super::filter_signature::FilterSignature;

#[derive(Debug, Default, Clone)]
pub struct FilterSignatureSequence(pub(crate) Vec<FilterSignature>);
