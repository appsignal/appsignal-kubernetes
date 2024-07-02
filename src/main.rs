use http::Request;
use k8s_openapi::api::core::v1::Node;
use kube::{api::ListParams, Api, ResourceExt};
use reqwest::{Client, Url};
use serde::Serialize;
use serde_json::value::Value;
use std::collections::HashMap;
use std::env;
use std::time::Duration;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Serialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
struct AppsignalMetric {
    name: String,
    metric_type: String,
    value: f32,
    tags: HashMap<String, String>,
}

impl AppsignalMetric {
    pub fn new(
        metric_name: &str,
        tags: HashMap<String, String>,
        value: &serde_json::Value,
    ) -> AppsignalMetric {
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

#[derive(Debug)]
struct KubernetesMetric {
    metric_type: String,
    name: String,
    cpu_usage_nano_cores: u64,
    cpu_usage_core_nano_seconds: u64,
    memory_available_bytes: u64,
    memory_usage_bytes: u64,
    memory_working_set_bytes: u64,
    memory_rss_bytes: u64,
    memory_page_faults: u64,
    memory_major_page_faults: u64,
    // network_rx_bytes: u64,
    // network_rx_errors: u64,
    // network_tx_bytes: u64,
    // network_tx_errors: u64,
    // fs_available_bytes: u64,
    // fs_capacity_bytes: u64,
    // fs_inodes_free: u64,
    // fs_inodes: u64,
    // fs_inodes_used: u64,
    // rlimit_maxpid: u64,
    // rlimit_curproc: u64,
    swap_usage_bytes: u64,
}

impl KubernetesMetric {
    pub fn from_node_json(json: serde_json::Value) -> KubernetesMetric {
        KubernetesMetric {
            metric_type: "node".to_string(),
            name: json["nodeName"].to_string(),
            cpu_usage_nano_cores: json["cpu"]["usageNanoCores"].as_u64().unwrap(),
            cpu_usage_core_nano_seconds: json["cpu"]["usageCoreNanoSeconds"].as_u64().unwrap(),
            memory_available_bytes: json["memory"]["availableBytes"].as_u64().unwrap(),
            memory_usage_bytes: json["memory"]["usageBytes"].as_u64().unwrap(),
            memory_working_set_bytes: json["memory"]["workingSetBytes"].as_u64().unwrap(),
            memory_rss_bytes: json["memory"]["rssBytes"].as_u64().unwrap(),
            memory_page_faults: json["memory"]["pageFaults"].as_u64().unwrap(),
            memory_major_page_faults: json["memory"]["majorPageFaults"].as_u64().unwrap(),
            swap_usage_bytes: json["swap"]["swapUsageBytes"].as_u64().unwrap(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let duration = Duration::new(60, 0);
    let mut interval = tokio::time::interval(duration);

    loop {
        interval.tick().await;
        run().await.expect("Failed to extract metrics.")
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
            .request::<serde_json::Value>(kube_request.clone())
            .await?;

        println!("Node: {:?}",
            KubernetesMetric::from_node_json(kube_response["node"].clone())
        );

        if let Some(pods) = kube_response["pods"].as_array() {
            for pod in pods {
                extract_pod_metrics(
                    pod,
                    pod["podRef"]["name"]
                        .as_str()
                        .expect("Could not extract pod name"),
                    &mut out,
                );
            }
        };

        extract_node_metrics(kube_response, &name, &mut out);
    }

    let json = serde_json::to_string(&out).expect("Could not serialize JSON");

    let endpoint =
        env::var("APPSIGNAL_ENDPOINT").unwrap_or("https://appsignal-endpoint.net".to_owned());
    let api_key = env::var("APPSIGNAL_API_KEY").expect("APPSIGNAL_API_KEY not set");
    let base = Url::parse(&endpoint).expect("Could not parse endpoint");
    let path = format!("metrics/json?api_key={}", api_key);
    let url = base.join(&path).expect("Could not build request URL");

    let reqwest_client = Client::builder().timeout(Duration::from_secs(30)).build()?;

    let appsignal_response = reqwest_client
        .post(url)
        .body(json.to_owned())
        .send()
        .await?;

    println!("Done: {:?}", appsignal_response);

    Ok(())
}

fn extract_pod_metrics(results: &Value, pod_name: &str, out: &mut Vec<AppsignalMetric>) {
    for (metric_name, metric_value) in [
        (
            "pod_cpu_usage_nano_cores",
            &results["cpu"]["usageNanoCores"],
        ),
        (
            "pod_cpu_usage_core_nano_seconds",
            &results["cpu"]["usageCoreNanoSeconds"],
        ),
        (
            "pod_memory_working_set_bytes",
            &results["memory"]["workingSetBytes"],
        ),
        (
            "pod_swap_available_bytes",
            &results["swap"]["swapAvailableBytes"],
        ),
        ("pod_swap_usage_bytes", &results["swap"]["swapUsageBytes"]),
    ] {
        let mut tags = HashMap::with_capacity(1);
        tags.insert("pod".to_owned(), pod_name.to_owned());
        out.push(AppsignalMetric::new(metric_name, tags, metric_value));
    }
}

fn extract_node_metrics(results: Value, node_name: &str, out: &mut Vec<AppsignalMetric>) {
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
            "node_memory_available_bytes",
            &results["node"]["memory"]["availableBytes"],
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
        let mut tags = HashMap::with_capacity(1);
        tags.insert("node".to_owned(), node_name.to_owned());
        out.push(AppsignalMetric::new(metric_name, tags, metric_value));
    }
}

#[cfg(test)]
mod tests {
    use crate::extract_node_metrics;
    use crate::extract_pod_metrics;
    use crate::AppsignalMetric;
    use crate::HashMap;
    use serde_json::json;

    #[test]
    fn extract_node_metrics_with_empty_results() {
        let mut out = Vec::new();
        extract_node_metrics(json!([]), "node", &mut out);
        assert_eq!(
            AppsignalMetric::new(
                "node_cpu_usage_nano_cores",
                HashMap::from([("node".to_string(), "node".to_string())]),
                &json!(0.0)
            ),
            out[0]
        );
    }

    #[test]
    fn extract_node_metrics_with_results() {
        let mut out = Vec::new();
        extract_node_metrics(
            json!({
             "node": {
              "cpu": {
               "time": "2024-03-29T12:21:36Z",
               "usageNanoCores": 232839439,
               "usageCoreNanoSeconds": 1118592000000 as u64
              },
            }
            }),
            "node",
            &mut out,
        );

        assert_eq!(
            AppsignalMetric::new(
                "node_cpu_usage_nano_cores",
                HashMap::from([("node".to_string(), "node".to_string())]),
                &json!(232839439)
            ),
            out[0]
        );
    }

    #[test]
    fn extract_pod_metrics_with_empty_results() {
        let mut out = Vec::new();
        extract_pod_metrics(&json!([]), "pod", &mut out);
        assert_eq!(
            AppsignalMetric::new(
                "pod_cpu_usage_nano_cores",
                HashMap::from([("pod".to_string(), "pod".to_string())]),
                &json!(0.0)
            ),
            out[0]
        );
    }

    #[test]
    fn extract_pod_metrics_with_results() {
        let mut out = Vec::new();
        extract_pod_metrics(
            &json!({
              "cpu": {
               "time": "2024-03-29T12:21:36Z",
               "usageNanoCores": 232839439,
               "usageCoreNanoSeconds": 1118592000000 as u64
              },
            }),
            "pod",
            &mut out,
        );

        assert_eq!(
            AppsignalMetric::new(
                "pod_cpu_usage_nano_cores",
                HashMap::from([("pod".to_string(), "pod".to_string())]),
                &json!(232839439)
            ),
            out[0]
        );
    }
}
