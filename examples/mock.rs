//! Mock HTTP **responses** with [`suitecase::mock`]: the code under test calls a small
//! transport that records `GET` + path and returns a body string, as if it were an HTTP client.
//!
//! Run: `cargo run --example mock`  
//! Test: `cargo test --example mock`

use serde_json::Value;
use suitecase::mock::{Mock, TestingT, eq};

struct NoopT;

impl TestingT for NoopT {
    fn errorf(&self, _msg: &str) {}

    fn fail_now(&self) {}
}

pub struct MockHttp {
    pub mock: Mock,
}

impl MockHttp {
    pub fn get(&self, path: &str) -> String {
        let out = self
            .mock
            .method_called("GET", suitecase::mock_args!(path.to_string()));
        out.string(0)
    }
}

fn repo_stargazers_from_response(body: &str) -> u64 {
    let v: Value = serde_json::from_str(body).expect("response JSON");
    v["stargazers_count"].as_u64().expect("stargazers_count")
}

pub fn fetch_repo_stargazers(client: &MockHttp, owner: &str, repo: &str) -> u64 {
    let path = format!("/repos/{owner}/{repo}");
    let body = client.get(&path);
    repo_stargazers_from_response(&body)
}

fn run_demo() {
    let api = MockHttp { mock: Mock::new() };
    let path = "/repos/octocat/Hello-World".to_string();
    let response = r#"{"stargazers_count":5000,"name":"Hello-World"}"#;

    api.mock
        .on("GET", vec![eq(path.clone())])
        .returning(move || vec![Box::new(response.to_string())])
        .finish();

    let stars = fetch_repo_stargazers(&api, "octocat", "Hello-World");
    assert_eq!(stars, 5000);
    assert!(api.mock.assert_expectations(&NoopT));
}

fn main() {
    run_demo();
    println!("mock HTTP example: ok");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_matches_main() {
        run_demo();
    }

    #[test]
    fn parses_github_style_json() {
        let body = r#"{"stargazers_count":42}"#;
        assert_eq!(repo_stargazers_from_response(body), 42);
    }
}
