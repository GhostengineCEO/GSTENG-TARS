use gsteng::ai::router::{get_response, LlmSource};

#[tokio::test]
async fn router_selects_local() {
    let resp = get_response(LlmSource::Local, "hello").await;
    assert_eq!(resp, "local-test");
}

#[tokio::test]
async fn router_selects_cloud() {
    let resp = get_response(LlmSource::Cloud, "hello").await;
    assert_eq!(resp, "cloud-test");
}
