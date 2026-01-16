#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct ContentFilterProperty {
    pub(crate) content_filtered_topic_name: [char; 256],
    pub(crate) related_topic_name: [char; 256],
    pub(crate) filter_class_name: [char; 256],
    pub(crate) filter_expression: String,
    pub(crate) expression_parameteres: Vec<String>,
}
