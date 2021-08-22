use std::{fmt, str::FromStr};

use serde::Deserialize;

use crate::common::{Datetime, CRS};

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Query {
    pub limit: Option<isize>, // OAF Core 1.0
    pub offset: Option<isize>,
    pub bbox: Option<String>, // OAF Core 1.0
    pub bbox_crs: Option<String>,
    pub datetime: Option<Datetime>, // OAF Core 1.0
    pub crs: Option<String>,
    pub filter: Option<String>,
    pub filter_lang: Option<String>, // default = 'cql-text'
    pub filter_crs: Option<String>,
}

// #[derive(Deserialize, Debug, Clone)]
// pub enum FilterLang {
//     CqlText,
//     CqlJson,
// }

impl Query {
    pub fn as_string_with_offset(&mut self, offset: isize) -> String {
        self.offset = Some(offset);
        self.to_string()
    }

    pub fn make_envelope(&self) -> Option<String> {
        if let Some(bbox) = self.bbox.as_ref() {
            let srid = self
                .bbox_crs
                .as_ref()
                .and_then(|crs| CRS::from_str(crs).ok())
                .and_then(|crs| crs.ogc_to_epsg())
                .map_or("4326".to_string(), |crs| crs.code)
                .parse::<u32>()
                .expect("Failed to parse bbox crs EPSG code");

            let mut coords: Vec<&str> = bbox.split(',').collect();

            if coords.len() == 6 {
                coords.remove(5);
                coords.remove(2);
            }
            assert_eq!(coords.len(), 4);

            Some(format!(
                "ST_MakeEnvelope({coords}, {srid})",
                coords = coords.join(", "),
                srid = srid,
            ))
        } else {
            None
        }
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut query_str = vec![];
        if let Some(limit) = self.limit {
            query_str.push(format!("limit={}", limit));
        }
        if let Some(offset) = self.offset {
            query_str.push(format!("offset={}", offset));
        }
        if let Some(bbox) = &self.bbox {
            query_str.push(format!("bbox={}", bbox));
        }
        if let Some(bbox_crs) = &self.bbox_crs {
            query_str.push(format!("bboxCrs={}", bbox_crs));
        }
        if let Some(datetime) = &self.datetime {
            query_str.push(format!("datetime={}", datetime));
        }
        if let Some(crs) = &self.crs {
            query_str.push(format!("crs={}", crs));
        }
        if let Some(filter) = &self.filter {
            query_str.push(format!("filter={}", filter));
        }
        if let Some(filter_lang) = &self.filter_lang {
            query_str.push(format!("filter-lang={}", filter_lang));
        }
        if let Some(filter_crs) = &self.filter_crs {
            query_str.push(format!("filter-crs={}", filter_crs));
        }
        write!(f, "{}", query_str.join("&"))
    }
}