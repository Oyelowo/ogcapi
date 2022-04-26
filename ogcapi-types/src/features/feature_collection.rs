use serde::{Deserialize, Serialize};

use crate::common::Links;

use super::Feature;

/// A set of Features from a dataset
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCollection {
    #[serde(default = "feature_collection")]
    pub r#type: String,
    pub features: Vec<Feature>,
    #[serde(default)]
    pub links: Links,
    pub time_stamp: Option<String>,
    pub number_matched: Option<u64>,
    pub number_returned: Option<usize>,
}

fn feature_collection() -> String {
    "FeatureCollection".to_string()
}