use http::Request;
use k8s_openapi::api::core::v1::Node;
use kube::{api::ListParams, Api, ResourceExt};
use reqwest::Client;
use serde::Serialize;
use serde_json::value::Value;
use std::collections::HashMap;
use std::env;
use std::time::Duration;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppsignalMetric {
    name: String,
    metric_type: String,
    value: f32,
    tags: HashMap<String, String>,
}

impl AppsignalMetric {
    pub fn new(metric_name: &str, node_name: &str, value: &serde_json::Value) -> AppsignalMetric {
        // Create tags
        let mut tags = HashMap::with_capacity(1);
        tags.insert("node".to_owned(), node_name.to_owned());

        // See if we can use value
        let value = match value {
            Value::Number(value) => match value.as_f64() {
                Some(value) => value as f32,
                None => 0.0,
            },
            _ => 0.0,
        };

        AppsignalMetric {
            name: metric_name.to_string(),
            metric_type: "gauge".to_string(),
            value,
            tags,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    loop {
        run().await.ok();
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

async fn run() -> Result<(), Error> {
    let kube_client = kube::Client::try_default().await?;
    let api: Api<Node> = Api::all(kube_client.clone());
    let nodes = api.list(&ListParams::default()).await?;

    for node in nodes {
        let name = node.name_any();
        let url = format!("/api/v1/nodes/{}/proxy/stats/summary", name);

        let kube_request = Request::get(url).body(Default::default())?;
        let kube_response = kube_client
            .request::<serde_json::Value>(kube_request)
            .await?;

        let mut out = Vec::new();

        for (metric_name, metric_value) in [
            // use .get instead of direct access
            (
                "node_cpu_usage_nano_cores",
                &kube_response["node"]["cpu"]["usageNanoCores"],
            ),
            (
                "node_cpu_usage_core_nano_seconds",
                &kube_response["node"]["cpu"]["usageCoreNanoSeconds"],
            ),
            (
                "node_memory_usage_bytes",
                &kube_response["node"]["memory"]["usageBytes"],
            ),
            (
                "node_memory_working_set_bytes",
                &kube_response["node"]["memory"]["workingSetBytes"],
            ),
            (
                "node_memory_rss_bytes",
                &kube_response["node"]["memory"]["rssBytes"],
            ),
            (
                "node_memory_page_faults",
                &kube_response["node"]["memory"]["pageFaults"],
            ),
            (
                "node_memory_major_page_faults",
                &kube_response["node"]["memory"]["majorPageFaults"],
            ),
            (
                "node_network_rx_bytes",
                &kube_response["node"]["network"]["rxBytes"],
            ),
            (
                "node_network_rx_errors",
                &kube_response["node"]["network"]["rxErrors"],
            ),
            (
                "node_network_tx_bytes",
                &kube_response["node"]["network"]["txBytes"],
            ),
            (
                "node_network_tx_errors",
                &kube_response["node"]["network"]["txErrors"],
            ),
            (
                "node_fs_available_bytes",
                &kube_response["node"]["fs"]["availableBytes"],
            ),
            (
                "node_s_capacity_bytes",
                &kube_response["node"]["fs"]["capacityBytes"],
            ),
            (
                "node_fs_used_bytes",
                &kube_response["node"]["fs"]["usedBytes"],
            ),
            (
                "node_fs_inodes_free",
                &kube_response["node"]["fs"]["inodesFree"],
            ),
            ("node_fs_inodes", &kube_response["node"]["fs"]["inodes"]),
            (
                "node_fs_inodes_used",
                &kube_response["node"]["fs"]["inodesUsed"],
            ),
            (
                "node_rlimit_maxpid",
                &kube_response["node"]["rlimit"]["maxpid"],
            ),
            (
                "node_rlimit_curproc",
                &kube_response["node"]["rlimit"]["curproc"],
            ),
            (
                "node_swap_available_bytes",
                &kube_response["node"]["swap"]["swapAvailableBytes"],
            ),
            (
                "node_swap_usage_bytes",
                &kube_response["node"]["swap"]["swapUsageBytes"],
            ),
        ] {
            out.push(AppsignalMetric::new(metric_name, &name, metric_value));
        }

        let json = serde_json::to_string(&out).expect("Could not serialize JSON");

        let endpoint = env::var("ENDPOINT").unwrap_or("unknown".to_string());
        let reqwest_client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        let appsignal_response = reqwest_client
            .post(&endpoint)
            .body(json.to_owned())
            .send()
            .await?;

        println!("Done: {:?}", appsignal_response);
    }

    Ok(())
}
