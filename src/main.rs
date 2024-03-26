use std::env;
use std::time::Duration;
use reqwest::Client;
use serde_json::json;
use http::Request;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let kube_client = kube::Client::try_default().await?;
    let name = "minikube";
    let url = format!("/api/v1/nodes/{}/proxy/stats/summary", name);

    let kube_request = Request::get(url).body(Default::default())?;
    let kube_response = kube_client.request::<serde_json::Value>(kube_request).await?;

    let json = json!([
        {
            "name": "cpu_nano_cores",
            "metricType": "gauge",
            "value": kube_response["node"]["cpu"]["usageNanoCores"],
            "tags": {"hostname": name}
        }
    ]).to_string();

    let endpoint = env::var("ENDPOINT").unwrap_or("unknown".to_string());
    let reqwest_client = Client::builder().timeout(Duration::from_secs(30)).build()?;

    let appsignal_response = reqwest_client
        .post(&endpoint)
        .body(json.as_str().to_owned())
        .send()
        .await?;

    println!("Done: {:?}", appsignal_response);

    Ok(())
}
