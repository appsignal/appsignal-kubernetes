use std::env;
use std::time::Duration;
use reqwest::Client;
use serde_json::json;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let endpoint = env::var("ENDPOINT").unwrap_or("unknown".to_string());
    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

    let json = json!({
        "name": "cpu_nano_cores",
        "metricType": "gauge",
        "value": 184417755,
        "tags": {"hostname": "minikube"}
    }).to_string();

    let res = client
        .post(&endpoint)
        .body(json.as_str().to_owned())
        .send()
        .await?;

    println!("Done: {:?}", res);

    Ok(())
}
