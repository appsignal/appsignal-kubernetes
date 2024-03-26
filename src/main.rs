use http::Request;
use k8s_openapi::api::core::v1::Node;
use kube::{api::ListParams, Api, ResourceExt};
use reqwest::Client;
use serde_json::json;
use std::env;
use std::time::Duration;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
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

        let json = json!([
            {
                "name": "node_cpu_usage_nano_cores",
                "metricType": "gauge",
                "value": kube_response["node"]["cpu"]["usageNanoCores"],
                "tags": {"node": name}
            },
            {
                "name": "node_cpu_usage_core_nano_seconds",
                "metricType": "gauge",
                "value": kube_response["node"]["cpu"]["usageCoreNanoSeconds"],
                "tags": {"node": name}
            },
            {
                "name": "node_memory_usage_bytes",
                "metricType": "gauge",
                "value": kube_response["node"]["memory"]["usageBytes"],
                "tags": {"node": name}
            },
            {
                "name": "node_memory_working_set_bytes",
                "metricType": "gauge",
                "value": kube_response["node"]["memory"]["workingSetBytes"],
                "tags": {"node": name}
            },
            {
                "name": "node_memory_rss_bytes",
                "metricType": "gauge",
                "value": kube_response["node"]["memory"]["rssBytes"],
                "tags": {"node": name}
            },
            {
                "name": "node_memory_page_faults",
                "metricType": "gauge",
                "value": kube_response["node"]["memory"]["pageFaults"],
                "tags": {"node": name}
            },
            {
                "name": "node_memory_major_page_faults",
                "metricType": "gauge",
                "value": kube_response["node"]["memory"]["majorPageFaults"],
                "tags": {"node": name}
            },
            {
                "name": "node_network_rx_bytes",
                "metricType": "gauge",
                "value": kube_response["node"]["network"]["rxBytes"],
                "tags": {"node": name}
            },
            {
                "name": "node_network_rx_errors",
                "metricType": "gauge",
                "value": kube_response["node"]["network"]["rxErrors"],
                "tags": {"node": name}
            },
            {
                "name": "node_network_tx_bytes",
                "metricType": "gauge",
                "value": kube_response["node"]["network"]["txBytes"],
                "tags": {"node": name}
            },
            {
                "name": "node_network_tx_errors",
                "metricType": "gauge",
                "value": kube_response["node"]["network"]["txErrors"],
                "tags": {"node": name}
            },
            {
                "name": "node_fs_available_bytes",
                "metricType": "gauge",
                "value": kube_response["node"]["fs"]["availableBytes"],
                "tags": {"node": name}
            },
            {
                "name": "node_s_capacity_bytes",
                "metricType": "gauge",
                "value": kube_response["node"]["fs"]["capacityBytes"],
                "tags": {"node": name}
            },
            {
                "name": "node_fs_used_bytes",
                "metricType": "gauge",
                "value": kube_response["node"]["fs"]["usedBytes"],
                "tags": {"node": name}
            },
            {
                "name": "node_fs_inodes_free",
                "metricType": "gauge",
                "value": kube_response["node"]["fs"]["inodesFree"],
                "tags": {"node": name}
            },
            {
                "name": "node_fs_inodes",
                "metricType": "gauge",
                "value": kube_response["node"]["fs"]["inodes"],
                "tags": {"node": name}
            },
            {
                "name": "node_fs_inodes_used",
                "metricType": "gauge",
                "value": kube_response["node"]["fs"]["inodesUsed"],
                "tags": {"node": name}
            },
            {
                "name": "node_rlimit_maxpid",
                "metricType": "gauge",
                "value": kube_response["node"]["rlimit"]["maxpid"],
                "tags": {"node": name}
            },
            {
                "name": "node_rlimit_curproc",
                "metricType": "gauge",
                "value": kube_response["node"]["rlimit"]["curproc"],
                "tags": {"node": name}
            },
            {
                "name": "node_swap_available_bytes",
                "metricType": "gauge",
                "value": kube_response["node"]["swap"]["swapAvailableBytes"],
                "tags": {"node": name}
            },
            {
                "name": "node_swap_usage_bytes",
                "metricType": "gauge",
                "value": kube_response["node"]["swap"]["swapUsageBytes"],
                "tags": {"node": name}
            }
        ])
        .to_string();

        let endpoint = env::var("ENDPOINT").unwrap_or("unknown".to_string());
        let reqwest_client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        let appsignal_response = reqwest_client
            .post(&endpoint)
            .body(json.as_str().to_owned())
            .send()
            .await?;

        println!("Done: {:?}", appsignal_response);
    }

    Ok(())
}
