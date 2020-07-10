extern crate serde_regex;

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::RwLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// A general abstraction of an HTTP request for all handlers.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MockServerHttpRequest {
    pub path: String,
    pub method: String,
    pub headers: Option<BTreeMap<String, String>>,
    pub query_params: Option<BTreeMap<String, String>>,
    pub body: Option<String>,
}

impl MockServerHttpRequest {
    pub fn new(method: String, path: String) -> Self {
        Self {
            path,
            method,
            headers: None,
            query_params: None,
            body: None,
        }
    }

    pub fn with_headers(mut self, arg: BTreeMap<String, String>) -> Self {
        self.headers = Some(arg);
        self
    }

    pub fn with_query_params(mut self, arg: BTreeMap<String, String>) -> Self {
        self.query_params = Some(arg);
        self
    }

    pub fn with_body(mut self, arg: String) -> Self {
        self.body = Some(arg);
        self
    }
}

/// A general abstraction of an HTTP response for all handlers.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MockServerHttpResponse {
    pub status: u16,
    pub headers: Option<BTreeMap<String, String>>,
    pub body: Option<String>,
    pub duration: Option<Duration>,
}

impl MockServerHttpResponse {
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: None,
            body: None,
            duration: None,
        }
    }

    pub fn with_headers(mut self, arg: BTreeMap<String, String>) -> Self {
        self.headers = Some(arg);
        self
    }

    pub fn with_body(mut self, arg: String) -> Self {
        self.body = Some(arg);
        self
    }
}

/// A general abstraction of an HTTP request for all handlers.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Pattern {
    #[serde(with = "serde_regex")]
    pub regex: Regex,
}

impl Pattern {
    pub fn from_regex(regex: Regex) -> Pattern {
        Pattern { regex }
    }
}

impl Ord for Pattern {
    fn cmp(&self, other: &Self) -> Ordering {
        self.regex.as_str().cmp(other.regex.as_str())
    }
}

impl PartialOrd for Pattern {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        self.regex.as_str() == other.regex.as_str()
    }
}

impl Eq for Pattern {}

pub type MockMatcherFunction = fn(Rc<MockServerHttpRequest>) -> bool;

/// A general abstraction of an HTTP request for all handlers.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RequestRequirements {
    pub path: Option<String>,
    pub path_contains: Option<Vec<String>>,
    pub path_matches: Option<Vec<Pattern>>,
    pub method: Option<String>,
    pub headers: Option<BTreeMap<String, String>>,
    pub header_exists: Option<Vec<String>>,
    pub body: Option<String>,
    pub json_body: Option<Value>,
    pub json_body_includes: Option<Vec<Value>>,
    pub body_contains: Option<Vec<String>>,
    pub body_matches: Option<Vec<Pattern>>,
    pub query_param_exists: Option<Vec<String>>,
    pub query_param: Option<BTreeMap<String, String>>,

    #[serde(skip_serializing, skip_deserializing)]
    pub matchers: Option<Vec<MockMatcherFunction>>,
}

impl RequestRequirements {
    pub fn new() -> Self {
        Self {
            path: None,
            path_contains: None,
            path_matches: None,
            method: None,
            headers: None,
            header_exists: None,
            body: None,
            json_body: None,
            json_body_includes: None,
            body_contains: None,
            body_matches: None,
            query_param_exists: None,
            query_param: None,
            matchers: None,
        }
    }

    pub fn with_path(mut self, arg: String) -> Self {
        self.path = Some(arg);
        self
    }

    pub fn with_method(mut self, arg: String) -> Self {
        self.method = Some(arg);
        self
    }

    pub fn with_body(mut self, arg: String) -> Self {
        self.body = Some(arg);
        self
    }

    pub fn with_json_body(mut self, arg: Value) -> Self {
        self.json_body = Some(arg);
        self
    }

    pub fn with_path_contains(mut self, arg: Vec<String>) -> Self {
        self.path_contains = Some(arg);
        self
    }

    pub fn with_path_matches(mut self, arg: Vec<Pattern>) -> Self {
        self.path_matches = Some(arg);
        self
    }

    pub fn with_headers(mut self, arg: BTreeMap<String, String>) -> Self {
        self.headers = Some(arg);
        self
    }

    pub fn with_header_exists(mut self, arg: Vec<String>) -> Self {
        self.header_exists = Some(arg);
        self
    }

    pub fn with_json_body_includes(mut self, arg: Vec<Value>) -> Self {
        self.json_body_includes = Some(arg);
        self
    }

    pub fn with_body_contains(mut self, arg: Vec<String>) -> Self {
        self.body_contains = Some(arg);
        self
    }

    pub fn with_body_matches(mut self, arg: Vec<Pattern>) -> Self {
        self.body_matches = Some(arg);
        self
    }

    pub fn with_query_param_exists(mut self, arg: Vec<String>) -> Self {
        self.query_param_exists = Some(arg);
        self
    }

    pub fn with_query_param(mut self, arg: BTreeMap<String, String>) -> Self {
        self.query_param = Some(arg);
        self
    }
}

/// A Request that is made to set a new mock.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MockDefinition {
    pub request: RequestRequirements,
    pub response: MockServerHttpResponse,
}

impl MockDefinition {
    pub fn new(req: RequestRequirements, mock: MockServerHttpResponse) -> Self {
        Self {
            request: req,
            response: mock,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MockIdentification {
    pub mock_id: usize,
}

impl MockIdentification {
    pub fn new(mock_id: usize) -> Self {
        Self { mock_id }
    }
}

/// The shared state accessible to all handlers
pub struct MockServerState {
    pub mocks: RwLock<BTreeMap<usize, ActiveMock>>,
    id_counter: AtomicUsize,
}

impl MockServerState {
    pub fn create_new_id(&self) -> usize {
        self.id_counter.fetch_add(1, Relaxed)
    }

    pub fn new() -> Self {
        MockServerState {
            mocks: RwLock::new(BTreeMap::new()),
            id_counter: AtomicUsize::new(0),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ActiveMock {
    pub id: usize,
    pub call_counter: usize,
    pub definition: MockDefinition,
}

impl ActiveMock {
    pub fn new(id: usize, mock_definition: MockDefinition) -> Self {
        ActiveMock {
            id,
            definition: mock_definition,
            call_counter: 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
}

impl ErrorResponse {
    pub fn new<T>(message: &T) -> ErrorResponse
    where
        T: ToString,
    {
        ErrorResponse {
            message: message.to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::server::data::{Pattern, RequestRequirements};
    use regex::Regex;
    use serde_json::json;
    use std::collections::BTreeMap;

    /// This test makes sure that adding the matching rules to a mock fills the struct as expected.
    #[test]
    fn fill_mock_requirements() {
        // Arrange
        let with_path = "with_path";
        let with_path_contains = vec!["with_path_contains".into()];
        let with_path_matches = vec![Pattern::from_regex(
            Regex::new(r#"with_path_matches"#).unwrap(),
        )];
        let mut with_headers = BTreeMap::new();
        with_headers.insert("test".into(), "value".into());
        let with_method = "GET";
        let with_body = "with_body";
        let with_body_contains = vec!["body_contains".into()];
        let with_body_matches = vec![Pattern::from_regex(
            Regex::new(r#"with_body_matches"#).unwrap(),
        )];
        let with_json_body = json!(12.5);
        let with_json_body_includes = vec![json!(12.5)];
        let with_query_param_exists = vec!["with_query_param_exists".into()];
        let mut with_query_param = BTreeMap::new();
        with_query_param.insert("with_query_param".into(), "value".into());
        let with_header_exists = vec!["with_header_exists".into()];

        // Act
        let rr = RequestRequirements::new()
            .with_path(with_path.clone().into())
            .with_path_contains(with_path_contains.clone())
            .with_path_matches(with_path_matches.clone())
            .with_headers(with_headers.clone())
            .with_method(with_method.clone().into())
            .with_body(with_body.clone().into())
            .with_body_contains(with_body_contains.clone())
            .with_body_matches(with_body_matches.clone())
            .with_json_body(with_json_body.clone())
            .with_json_body_includes(with_json_body_includes.clone())
            .with_query_param_exists(with_query_param_exists.clone())
            .with_query_param(with_query_param.clone())
            .with_header_exists(with_header_exists.clone());

        // Assert
        assert_eq!(rr.path.as_ref().unwrap(), with_path.clone());
        assert_eq!(
            rr.path_contains.as_ref().unwrap(),
            &with_path_contains.clone()
        );
        assert_eq!(
            rr.path_matches.as_ref().unwrap(),
            &with_path_matches.clone()
        );
        assert_eq!(rr.headers.as_ref().unwrap(), &with_headers.clone());
        assert_eq!(rr.body.as_ref().unwrap(), with_body.clone());
        assert_eq!(
            rr.body_contains.as_ref().unwrap(),
            &with_body_contains.clone()
        );
        assert_eq!(
            rr.body_matches.as_ref().unwrap(),
            &with_body_matches.clone()
        );
        assert_eq!(rr.json_body.as_ref().unwrap(), &with_json_body.clone());
        assert_eq!(
            rr.json_body_includes.as_ref().unwrap(),
            &with_json_body_includes.clone()
        );
        assert_eq!(
            rr.query_param_exists.as_ref().unwrap(),
            &with_query_param_exists.clone()
        );
        assert_eq!(rr.query_param.as_ref().unwrap(), &with_query_param.clone());
        assert_eq!(
            rr.header_exists.as_ref().unwrap(),
            &with_header_exists.clone()
        );
    }
}
