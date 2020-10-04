use crate::data::{HttpMockRequest, RequestRequirements};
use crate::server::matchers::util::{distance_for, distance_for_opt};
use crate::server::matchers::{diff_str, Matcher, Mismatch, SimpleDiffResult, Tokenizer};

pub(crate) struct PathMatcher {}

impl PathMatcher {
    pub fn new(weight: f32) -> Self {
        Self {}
    }
}

impl Matcher for PathMatcher {
    fn matches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> bool {
        mock.path.as_ref().map_or(true, |path| path.eq(&req.path))
    }

    fn mismatches(&self, req: &HttpMockRequest, mock: &RequestRequirements) -> Vec<Mismatch> {
        match self.matches(req, mock) {
            true => Vec::new(),
            false => vec![Mismatch {
                title: "Request path does not match".to_string(),
                message: None,
                reason: Some(SimpleDiffResult {
                    expected: mock.path.as_ref().unwrap().to_owned(),
                    actual: req.path.to_owned(),
                    operation_name: "TODO".to_string(),
                    best_match: false,
                }),
                detailed_diff: None,
                score: distance_for_opt(&req.path, &mock.path.as_ref()),
            }],
        }
    }
}
