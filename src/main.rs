use http::Request;
use k8s_openapi::api::core::v1::Node;
use kube::{api::ListParams, Api, ResourceExt};
use reqwest::{Client, Url};
use serde::Serialize;
use serde_json::value::Value;
use std::collections::{BTreeMap, HashSet};
use std::env;
use std::time::Duration;

use std::hash::{Hash, Hasher};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Serialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct AppsignalMetric {
    name: String,
    metric_type: String,
    value: f64,
    tags: BTreeMap<String, String>,
}

impl AppsignalMetric {
    pub fn new(
        metric_name: &str,
        tags: BTreeMap<String, String>,
        value: &serde_json::Value,
    ) -> Option<AppsignalMetric> {
        value.as_f64().map(|value| Self {
            name: metric_name.to_string(),
            metric_type: "gauge".to_string(),
            value,
            tags,
        })
    }

    pub fn into_key(self) -> AppsignalMetricKey {
        AppsignalMetricKey(self)
    }
}

#[derive(Serialize, Clone, Debug)]
struct AppsignalMetricKey(AppsignalMetric);

impl Hash for AppsignalMetricKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.name.hash(state);
        self.0.tags.hash(state);
    }
}

impl PartialEq for AppsignalMetricKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.name == other.0.name && self.0.tags == other.0.tags
    }
}

impl Eq for AppsignalMetricKey {}

#[derive(Debug)]
struct KubernetesMetric {
    metric_type: String,
    name: String,
    cpu_usage_nano_cores: Option<u64>,
    cpu_usage_core_nano_seconds: Option<u64>,
    memory_available_bytes: Option<u64>,
    memory_usage_bytes: Option<u64>,
    memory_working_set_bytes: Option<u64>,
    memory_rss_bytes: Option<u64>,
    memory_page_faults: Option<u64>,
    memory_major_page_faults: Option<u64>,
    network_rx_bytes: Option<u64>,
    network_rx_errors: Option<u64>,
    network_tx_bytes: Option<u64>,
    network_tx_errors: Option<u64>,
    fs_available_bytes: Option<u64>,
    fs_capacity_bytes: Option<u64>,
    fs_used_bytes: Option<u64>,
    fs_inodes_free: Option<u64>,
    fs_inodes: Option<u64>,
    fs_inodes_used: Option<u64>,
    rlimit_maxpid: Option<u64>,
    rlimit_curproc: Option<u64>,
    ephemeral_storage_available_bytes: Option<u64>,
    ephemeral_storage_capacity_bytes: Option<u64>,
    ephemeral_storage_used_bytes: Option<u64>,
    ephemeral_storage_inodes_free: Option<u64>,
    ephemeral_storage_inodes: Option<u64>,
    ephemeral_storage_inodes_used: Option<u64>,
    process_count: Option<u64>,
    swap_usage_bytes: Option<u64>,
}

impl KubernetesMetric {
    pub fn from_node_json(json: serde_json::Value) -> KubernetesMetric {
        KubernetesMetric {
            metric_type: "node".to_string(),
            name: json["nodeName"].to_string(),
            cpu_usage_nano_cores: json["cpu"]["usageNanoCores"].as_u64(),
            cpu_usage_core_nano_seconds: json["cpu"]["usageCoreNanoSeconds"].as_u64(),
            memory_available_bytes: json["memory"]["availableBytes"].as_u64(),
            memory_usage_bytes: json["memory"]["usageBytes"].as_u64(),
            memory_working_set_bytes: json["memory"]["workingSetBytes"].as_u64(),
            memory_rss_bytes: json["memory"]["rssBytes"].as_u64(),
            memory_page_faults: json["memory"]["pageFaults"].as_u64(),
            memory_major_page_faults: json["memory"]["majorPageFaults"].as_u64(),
            network_rx_bytes: json["network"]["rxBytes"].as_u64(),
            network_rx_errors: json["network"]["rxErrors"].as_u64(),
            network_tx_bytes: json["network"]["txBytes"].as_u64(),
            network_tx_errors: json["network"]["txErrors"].as_u64(),
            fs_available_bytes: json["fs"]["availableBytes"].as_u64(),
            fs_capacity_bytes: json["fs"]["capacityBytes"].as_u64(),
            fs_used_bytes: json["fs"]["usedBytes"].as_u64(),
            fs_inodes_free: json["fs"]["inodesFree"].as_u64(),
            fs_inodes: json["fs"]["inodes"].as_u64(),
            fs_inodes_used: json["fs"]["inodesUsed"].as_u64(),
            rlimit_maxpid: json["rlimit"]["maxpid"].as_u64(),
            rlimit_curproc: json["rlimit"]["curproc"].as_u64(),
            ephemeral_storage_available_bytes: None,
            ephemeral_storage_capacity_bytes: None,
            ephemeral_storage_used_bytes: None,
            ephemeral_storage_inodes_free: None,
            ephemeral_storage_inodes: None,
            ephemeral_storage_inodes_used: None,
            process_count: None,
            swap_usage_bytes: json["swap"]["swapUsageBytes"].as_u64(),
        }
    }

    pub fn from_pod_json(json: serde_json::Value) -> KubernetesMetric {
        KubernetesMetric {
            metric_type: "pod".to_string(),
            name: json["podRef"]["name"].to_string(),
            cpu_usage_nano_cores: json["cpu"]["usageNanoCores"].as_u64(),
            cpu_usage_core_nano_seconds: json["cpu"]["usageCoreNanoSeconds"].as_u64(),
            memory_available_bytes: None,
            memory_usage_bytes: json["memory"]["usageBytes"].as_u64(),
            memory_working_set_bytes: json["memory"]["workingSetBytes"].as_u64(),
            memory_rss_bytes: json["memory"]["rssBytes"].as_u64(),
            memory_page_faults: json["memory"]["pageFaults"].as_u64(),
            memory_major_page_faults: json["memory"]["majorPageFaults"].as_u64(),
            network_rx_bytes: None,
            network_rx_errors: None,
            network_tx_bytes: None,
            network_tx_errors: None,
            fs_available_bytes: None,
            fs_capacity_bytes: None,
            fs_used_bytes: None,
            fs_inodes_free: None,
            fs_inodes: None,
            fs_inodes_used: None,
            rlimit_maxpid: None,
            rlimit_curproc: None,
            ephemeral_storage_available_bytes: json["ephemeral-storage"]["availableBytes"].as_u64(),
            ephemeral_storage_capacity_bytes: json["ephemeral-storage"]["capacityBytes"].as_u64(),
            ephemeral_storage_used_bytes: json["ephemeral-storage"]["usedBytes"].as_u64(),
            ephemeral_storage_inodes_free: json["ephemeral-storage"]["InodesFree"].as_u64(),
            ephemeral_storage_inodes: json["ephemeral-storage"]["Inodes"].as_u64(),
            ephemeral_storage_inodes_used: json["ephemeral-storage"]["InodesUsed"].as_u64(),
            process_count: json["process_stats"]["process_count"].as_u64(),
            swap_usage_bytes: json["swap"]["swapUsageBytes"].as_u64(),
        }
    }
}

#[tokio::main]
async fn main() {
    let duration = Duration::new(60, 0);
    let mut interval = tokio::time::interval(duration);
    let metrics_url = must_metrics_url_from_env();

    loop {
        interval.tick().await;
        if let Err(error) = run(&metrics_url).await {
            eprintln!("Failed to extract metrics: {}", &error);
        };
    }
}

fn must_metrics_url_from_env() -> Url {
    let endpoint =
        env::var("APPSIGNAL_ENDPOINT").unwrap_or("https://appsignal-endpoint.net".to_owned());
    let api_key = env::var("APPSIGNAL_API_KEY").expect("APPSIGNAL_API_KEY not set");
    let base = Url::parse(&endpoint).expect("Could not parse endpoint");
    let path = format!("metrics/json?api_key={}", api_key);
    base.join(&path).expect("Could not build request URL")
}

async fn run(metrics_url: &Url) -> Result<(), Error> {
    let kube_client = kube::Client::try_default().await?;
    let api: Api<Node> = Api::all(kube_client.clone());
    let nodes = api.list(&ListParams::default()).await?;
    let mut out = HashSet::new();

    for node in nodes {
        let url = format!("/api/v1/nodes/{}/proxy/stats/summary", node.name_any());

        let kube_request = Request::get(url).body(Default::default())?;
        let kube_response = kube_client
            .request::<serde_json::Value>(kube_request)
            .await?;

        extract_metrics(&kube_response, &mut out);
    }

    let metrics_count = out.len();

    let json = serde_json::to_string(&out).expect("Could not serialize JSON");

    let reqwest_client = Client::builder().timeout(Duration::from_secs(30)).build()?;

    let appsignal_response = reqwest_client
        .post(metrics_url.clone())
        .body(json.to_owned())
        .send()
        .await?;

    println!(
        "Sent {} metrics to AppSignal: status code {:?}",
        metrics_count,
        appsignal_response.status()
    );

    Ok(())
}

fn extract_metrics(kube_response: &Value, out: &mut HashSet<AppsignalMetricKey>) {
    println!(
        "Node: {:?}",
        KubernetesMetric::from_node_json(kube_response["node"].clone())
    );

    extract_node_metrics(&kube_response["node"], out);

    if let Some(pods) = kube_response["pods"].as_array() {
        for pod in pods {
            println!("Pod: {:?}", KubernetesMetric::from_pod_json(pod.clone()));

            let pod_name = extract_pod_metrics(pod, out);

            if let (Some(pod_name), Some(volumes)) = (pod_name, pod["volume"].as_array()) {
                for volume in volumes {
                    extract_volume_metrics(volume, &pod_name, out);
                }
            }
        }
    };
}

fn extract_volume_metrics(results: &Value, pod_name: &str, out: &mut HashSet<AppsignalMetricKey>) {
    let volume_name = if let Some(name) = results["name"].as_str() {
        name
    } else {
        eprintln!("Could not extract volume name");
        return;
    };

    for (metric_name, metric_value) in [
        ("volume_available_bytes", &results["availableBytes"]),
        ("volume_capacity_bytes", &results["capacityBytes"]),
        ("volume_used_bytes", &results["usedBytes"]),
        ("volume_inodes_free", &results["inodesFree"]),
        ("volume_inodes", &results["inodes"]),
        ("volume_inodes_used", &results["inodesUsed"]),
    ] {
        let mut tags = BTreeMap::new();
        tags.insert("pod".to_owned(), pod_name.to_owned());
        tags.insert("volume".to_owned(), volume_name.to_owned());

        if let Some(metric) = AppsignalMetric::new(metric_name, tags, metric_value) {
            out.insert(metric.into_key());
        }
    }
}

fn extract_pod_metrics(
    pod_results: &Value,
    out: &mut HashSet<AppsignalMetricKey>,
) -> Option<String> {
    let pod_name = if let Some(name) = pod_results["podRef"]["name"].as_str() {
        name
    } else {
        eprintln!("Could not extract pod name");
        return None;
    };

    for (metric_name, metric_value) in [
        (
            "pod_cpu_usage_nano_cores",
            &pod_results["cpu"]["usageNanoCores"],
        ),
        (
            "pod_cpu_usage_core_nano_seconds",
            &pod_results["cpu"]["usageCoreNanoSeconds"],
        ),
        (
            "pod_memory_working_set_bytes",
            &pod_results["memory"]["workingSetBytes"],
        ),
        (
            "pod_swap_available_bytes",
            &pod_results["swap"]["swapAvailableBytes"],
        ),
        (
            "pod_swap_usage_bytes",
            &pod_results["swap"]["swapUsageBytes"],
        ),
    ] {
        let mut tags = BTreeMap::new();
        tags.insert("pod".to_owned(), pod_name.to_owned());

        if let Some(metric) = AppsignalMetric::new(metric_name, tags, metric_value) {
            out.insert(metric.into_key());
        }
    }

    Some(pod_name.to_owned())
}

fn extract_node_metrics(node_results: &Value, out: &mut HashSet<AppsignalMetricKey>) {
    let node_name = if let Some(name) = node_results["nodeName"].as_str() {
        name
    } else {
        eprintln!("Could not extract node name");
        return;
    };

    for (metric_name, metric_value) in [
        (
            "node_cpu_usage_nano_cores",
            &node_results["cpu"]["usageNanoCores"],
        ),
        (
            "node_cpu_usage_core_nano_seconds",
            &node_results["cpu"]["usageCoreNanoSeconds"],
        ),
        (
            "node_memory_available_bytes",
            &node_results["memory"]["availableBytes"],
        ),
        (
            "node_memory_usage_bytes",
            &node_results["memory"]["usageBytes"],
        ),
        (
            "node_memory_working_set_bytes",
            &node_results["memory"]["workingSetBytes"],
        ),
        ("node_memory_rss_bytes", &node_results["memory"]["rssBytes"]),
        (
            "node_memory_page_faults",
            &node_results["memory"]["pageFaults"],
        ),
        (
            "node_memory_major_page_faults",
            &node_results["memory"]["majorPageFaults"],
        ),
        ("node_network_rx_bytes", &node_results["network"]["rxBytes"]),
        (
            "node_network_rx_errors",
            &node_results["network"]["rxErrors"],
        ),
        ("node_network_tx_bytes", &node_results["network"]["txBytes"]),
        (
            "node_network_tx_errors",
            &node_results["network"]["txErrors"],
        ),
        (
            "node_fs_available_bytes",
            &node_results["fs"]["availableBytes"],
        ),
        (
            "node_fs_capacity_bytes",
            &node_results["fs"]["capacityBytes"],
        ),
        ("node_fs_used_bytes", &node_results["fs"]["usedBytes"]),
        ("node_fs_inodes_free", &node_results["fs"]["inodesFree"]),
        ("node_fs_inodes", &node_results["fs"]["inodes"]),
        ("node_fs_inodes_used", &node_results["fs"]["inodesUsed"]),
        ("node_rlimit_maxpid", &node_results["rlimit"]["maxpid"]),
        ("node_rlimit_curproc", &node_results["rlimit"]["curproc"]),
        (
            "node_swap_available_bytes",
            &node_results["swap"]["swapAvailableBytes"],
        ),
        (
            "node_swap_usage_bytes",
            &node_results["swap"]["swapUsageBytes"],
        ),
    ] {
        let mut tags = BTreeMap::new();
        tags.insert("node".to_owned(), node_name.to_owned());

        if let Some(metric) = AppsignalMetric::new(metric_name, tags, metric_value) {
            out.insert(metric.into_key());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use serde_json::json;

    fn assert_contains_metric(out: &HashSet<AppsignalMetricKey>, metric: AppsignalMetric) {
        if let Some(out_metric) = out.get(&metric.clone().into_key()) {
            assert_eq!(out_metric.0, metric);
        } else {
            panic!("Metric not found: {:?}", &metric);
        }
    }

    #[test]
    fn extract_metrics_without_response() {
        let mut out = HashSet::new();
        extract_metrics(&json!({}), &mut out);
        assert_eq!(out.len(), 0);
    }

    #[test]
    fn extract_metrics_with_response() {
        let mut out = HashSet::new();
        extract_metrics(
            &json!({
              "node": {
                "nodeName": "some_node",
                "cpu": {
                 "time": "2024-03-29T12:21:36Z",
                 "usageNanoCores": 111111111,
                },
              },
              "pods": [
                {
                  "podRef": {
                      "name": "some_pod"
                  },
                  "cpu": {
                   "time": "2024-03-29T12:21:36Z",
                   "usageNanoCores": 222222222,
                  },
                },
                {
                  "podRef": {
                      "name": "other_pod"
                  },
                  "cpu": {
                   "time": "2024-03-29T12:21:36Z",
                   "usageNanoCores": 333333333,
                  },
                  "volume": [
                    {
                      "name": "some_volume",
                      "availableBytes": 444444444,
                    },
                    {
                      "name": "other_volume",
                      "availableBytes": 555555555,
                    },
                  ],
                },
              ],
            }),
            &mut out,
        );
        assert_eq!(out.len(), 5);
        assert_contains_metric(
            &out,
            AppsignalMetric::new(
                "node_cpu_usage_nano_cores",
                BTreeMap::from([("node".to_string(), "some_node".to_string())]),
                &json!(111111111),
            )
            .expect("Could not create metric"),
        );
        assert_contains_metric(
            &out,
            AppsignalMetric::new(
                "pod_cpu_usage_nano_cores",
                BTreeMap::from([("pod".to_string(), "some_pod".to_string())]),
                &json!(222222222),
            )
            .expect("Could not create metric"),
        );
        assert_contains_metric(
            &out,
            AppsignalMetric::new(
                "pod_cpu_usage_nano_cores",
                BTreeMap::from([("pod".to_string(), "other_pod".to_string())]),
                &json!(333333333),
            )
            .expect("Could not create metric"),
        );
        assert_contains_metric(
            &out,
            AppsignalMetric::new(
                "volume_available_bytes",
                BTreeMap::from([
                    ("pod".to_string(), "other_pod".to_string()),
                    ("volume".to_string(), "some_volume".to_string()),
                ]),
                &json!(444444444),
            )
            .expect("Could not create metric"),
        );
        assert_contains_metric(
            &out,
            AppsignalMetric::new(
                "volume_available_bytes",
                BTreeMap::from([
                    ("pod".to_string(), "other_pod".to_string()),
                    ("volume".to_string(), "other_volume".to_string()),
                ]),
                &json!(555555555),
            )
            .expect("Could not create metric"),
        );
    }

    #[test]
    fn extract_node_metrics_without_results() {
        let mut out = HashSet::new();
        extract_node_metrics(&json!({}), &mut out);
        assert_eq!(out.len(), 0);
    }

    #[test]
    fn extract_node_metrics_missing_name() {
        let mut out: HashSet<AppsignalMetricKey, _> = HashSet::new();
        extract_node_metrics(
            &json!({
              "cpu": {
               "time": "2024-03-29T12:21:36Z",
               "usageNanoCores": 232839439,
               "usageCoreNanoSeconds": 1118592000000_u64
              },
            }),
            &mut out,
        );

        assert_eq!(out.len(), 0);
    }

    #[test]
    fn extract_node_metrics_with_results() {
        let mut out = HashSet::new();
        extract_node_metrics(
            &json!({
              "nodeName": "some_node",
              "cpu": {
               "time": "2024-03-29T12:21:36Z",
               "usageNanoCores": 232839439,
               "usageCoreNanoSeconds": 1118592000000_u64
              },
            }),
            &mut out,
        );

        assert_contains_metric(
            &out,
            AppsignalMetric::new(
                "node_cpu_usage_nano_cores",
                BTreeMap::from([("node".to_string(), "some_node".to_string())]),
                &json!(232839439),
            )
            .expect("Could not create metric"),
        );
        assert_contains_metric(
            &out,
            AppsignalMetric::new(
                "node_cpu_usage_core_nano_seconds",
                BTreeMap::from([("node".to_string(), "some_node".to_string())]),
                &json!(1118592000000_u64),
            )
            .expect("Could not create metric"),
        );

        assert_eq!(out.len(), 2);
    }

    #[test]
    fn extract_pod_metrics_without_results() {
        let mut out = HashSet::new();
        extract_pod_metrics(&json!({}), &mut out);
        assert_eq!(out.len(), 0);
    }

    #[test]
    fn extract_pod_metrics_missing_name() {
        let mut out = HashSet::new();
        extract_pod_metrics(
            &json!({
              "cpu": {
               "time": "2024-03-29T12:21:36Z",
               "usageNanoCores": 232839439,
               "usageCoreNanoSeconds": 1118592000000_u64
              },
            }),
            &mut out,
        );

        assert_eq!(out.len(), 0);
    }

    #[test]
    fn extract_pod_metrics_with_results() {
        let mut out = HashSet::new();
        extract_pod_metrics(
            &json!({
              "podRef": {
                  "name": "some_pod"
              },
              "cpu": {
               "time": "2024-03-29T12:21:36Z",
               "usageNanoCores": 232839439,
               "usageCoreNanoSeconds": 1118592000000_u64
              },
            }),
            &mut out,
        );

        assert_contains_metric(
            &out,
            AppsignalMetric::new(
                "pod_cpu_usage_nano_cores",
                BTreeMap::from([("pod".to_string(), "some_pod".to_string())]),
                &json!(232839439),
            )
            .expect("Could not create metric"),
        );
        assert_contains_metric(
            &out,
            AppsignalMetric::new(
                "pod_cpu_usage_core_nano_seconds",
                BTreeMap::from([("pod".to_string(), "some_pod".to_string())]),
                &json!(1118592000000_u64),
            )
            .expect("Could not create metric"),
        );

        assert_eq!(out.len(), 2);
    }

    #[test]
    fn extract_volume_metrics_without_results() {
        let mut out: HashSet<AppsignalMetricKey, _> = HashSet::new();
        extract_volume_metrics(&json!({}), "some_pod", &mut out);

        assert_eq!(out.len(), 0);
    }

    #[test]
    fn extract_volume_metrics_missing_name() {
        let mut out = HashSet::new();
        extract_volume_metrics(
            &json!({
              "availableBytes": 232839439,
              "capacityBytes": 1118592000000_u64,
            }),
            "some_pod",
            &mut out,
        );

        assert_eq!(out.len(), 0);
    }

    #[test]
    fn extract_volume_metrics_with_results() {
        let mut out = HashSet::new();
        extract_volume_metrics(
            &json!({
              "name": "some_volume",
              "availableBytes": 232839439,
              "capacityBytes": 1118592000000_u64,
            }),
            "some_pod",
            &mut out,
        );

        assert_contains_metric(
            &out,
            AppsignalMetric::new(
                "volume_available_bytes",
                BTreeMap::from([
                    ("pod".to_string(), "some_pod".to_string()),
                    ("volume".to_string(), "some_volume".to_string()),
                ]),
                &json!(232839439),
            )
            .expect("Could not create metric"),
        );

        assert_contains_metric(
            &out,
            AppsignalMetric::new(
                "volume_capacity_bytes",
                BTreeMap::from([
                    ("pod".to_string(), "some_pod".to_string()),
                    ("volume".to_string(), "some_volume".to_string()),
                ]),
                &json!(1118592000000_u64),
            )
            .expect("Could not create metric"),
        );

        assert_eq!(out.len(), 2);
    }

    #[test]
    fn serialize_metrics() {
        let mut out = HashSet::new();
        out.insert(
            AppsignalMetric::new(
                "some_metric",
                BTreeMap::from([("some_key".to_string(), "some_value".to_string())]),
                &json!(123456789),
            )
            .expect("Could not create metric")
            .into_key(),
        );

        let json = serde_json::to_string(&out).expect("Could not serialize JSON");

        assert_eq!(json, "[{\"name\":\"some_metric\",\"metricType\":\"gauge\",\"value\":123456789.0,\"tags\":{\"some_key\":\"some_value\"}}]");
    }
}
