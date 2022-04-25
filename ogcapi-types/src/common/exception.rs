use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Exception based on [`RFC 7807`](https://datatracker.ietf.org/doc/html/rfc7807)
#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
pub struct Exception {
    /// A URI reference that identifies the problem type.
    pub r#type: String,
    /// A short, human-readable summary of the problem type.
    pub title: Option<String>,
    /// The HTTP status code generated by the origin server for this occurrence
    /// of the problem.
    pub status: Option<u16>,
    /// A human-readable explanation specific to this occurrence of the problem.
    pub detail: Option<String>,
    /// A URI reference that identifies the specific occurrence of the problem.
    /// It may or may not yield further information if dereferenced.
    pub instance: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "Map::is_empty")]
    pub additional_properties: Map<String, Value>,
}

impl Exception {
    pub fn new(r#type: impl ToString) -> Self {
        Exception {
            r#type: r#type.to_string(),
            ..Default::default()
        }
    }

    pub fn new_from_status(status: u16) -> Self {
        let exception = Exception::new(format!(
            "https://httpwg.org/specs/rfc7231.html#status.{}",
            status
        ));
        exception.status(status)
    }

    pub fn title(mut self, title: impl ToString) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn status(mut self, status: u16) -> Self {
        self.status = Some(status);
        self
    }

    pub fn detail(mut self, detail: impl ToString) -> Self {
        self.detail = Some(detail.to_string());
        self
    }

    pub fn instance(mut self, instance: impl ToString) -> Self {
        self.instance = Some(instance.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::Exception;

    #[test]
    fn exception() {
        let e = Exception::new_from_status(500);
        println!("{:#?}", e);
        println!("{}", serde_json::to_string_pretty(&e).unwrap());
    }
}