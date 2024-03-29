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
    let mut out = Vec::new();

    for node in nodes {
        let name = node.name_any();
        let url = format!("/api/v1/nodes/{}/proxy/stats/summary", name);

        let kube_request = Request::get(url).body(Default::default())?;
        let kube_response = kube_client
            .request::<serde_json::Value>(kube_request)
            .await?;

        extract_metrics(kube_response, &name, &mut out);
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

    Ok(())
}

fn extract_metrics(results: Value, node_name: &str, out: &mut Vec<AppsignalMetric>) {
    for (metric_name, metric_value) in [
        (
            "node_cpu_usage_nano_cores",
            &results["node"]["cpu"]["usageNanoCores"],
        ),
        (
            "node_cpu_usage_core_nano_seconds",
            &results["node"]["cpu"]["usageCoreNanoSeconds"],
        ),
        (
            "node_memory_usage_bytes",
            &results["node"]["memory"]["usageBytes"],
        ),
        (
            "node_memory_working_set_bytes",
            &results["node"]["memory"]["workingSetBytes"],
        ),
        (
            "node_memory_rss_bytes",
            &results["node"]["memory"]["rssBytes"],
        ),
        (
            "node_memory_page_faults",
            &results["node"]["memory"]["pageFaults"],
        ),
        (
            "node_memory_major_page_faults",
            &results["node"]["memory"]["majorPageFaults"],
        ),
        (
            "node_network_rx_bytes",
            &results["node"]["network"]["rxBytes"],
        ),
        (
            "node_network_rx_errors",
            &results["node"]["network"]["rxErrors"],
        ),
        (
            "node_network_tx_bytes",
            &results["node"]["network"]["txBytes"],
        ),
        (
            "node_network_tx_errors",
            &results["node"]["network"]["txErrors"],
        ),
        (
            "node_fs_available_bytes",
            &results["node"]["fs"]["availableBytes"],
        ),
        (
            "node_s_capacity_bytes",
            &results["node"]["fs"]["capacityBytes"],
        ),
        ("node_fs_used_bytes", &results["node"]["fs"]["usedBytes"]),
        ("node_fs_inodes_free", &results["node"]["fs"]["inodesFree"]),
        ("node_fs_inodes", &results["node"]["fs"]["inodes"]),
        ("node_fs_inodes_used", &results["node"]["fs"]["inodesUsed"]),
        ("node_rlimit_maxpid", &results["node"]["rlimit"]["maxpid"]),
        ("node_rlimit_curproc", &results["node"]["rlimit"]["curproc"]),
        (
            "node_swap_available_bytes",
            &results["node"]["swap"]["swapAvailableBytes"],
        ),
        (
            "node_swap_usage_bytes",
            &results["node"]["swap"]["swapUsageBytes"],
        ),
    ] {
        out.push(AppsignalMetric::new(metric_name, &node_name, metric_value));
    }
}
