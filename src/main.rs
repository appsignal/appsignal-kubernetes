extern crate time;

use http::Request;
use k8s_openapi::api::core::v1::Node;
use kube::{api::ListParams, Api, ResourceExt};
use log::{debug, trace};
use protobuf::Message;
use reqwest::{Client, Url};
use std::env;
use std::time::Duration;

include!("../protocol/mod.rs");

use kubernetes::KubernetesMetrics;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

impl KubernetesMetrics {
    pub fn from_node_json(json: serde_json::Value) -> KubernetesMetrics {
        let mut metric = KubernetesMetrics::new();

        if let Some(node_name) = json["nodeName"].as_str() {
            metric.set_node_name(node_name.to_string());
        }

        metric.set_timestamp(now_timestamp());

        if let Some(cpu_usage_nano_cores) = json["cpu"]["usageNanoCores"].as_i64() {
            metric.set_cpu_usage_nano_cores(cpu_usage_nano_cores);
        }

        if let Some(cpu_usage_core_nano_seconds) = json["cpu"]["usageNanoSeconds"].as_i64() {
            metric.set_cpu_usage_core_nano_seconds(cpu_usage_core_nano_seconds);
        }

        if let Some(memory_available_bytes) = json["memory"]["availableBytes"].as_i64() {
            metric.set_memory_available_bytes(memory_available_bytes);
        }

        if let Some(memory_usage_bytes) = json["memory"]["usageBytes"].as_i64() {
            metric.set_memory_usage_bytes(memory_usage_bytes);
        }

        if let Some(memory_working_set_bytes) = json["memory"]["workingSetBytes"].as_i64() {
            metric.set_memory_working_set_bytes(memory_working_set_bytes);
        }

        if let Some(memory_rss_bytes) = json["memory"]["rssBytes"].as_i64() {
            metric.set_memory_rss_bytes(memory_rss_bytes);
        }

        if let Some(memory_page_faults) = json["memory"]["pageFaults"].as_i64() {
            metric.set_memory_page_faults(memory_page_faults as i32);
        }

        if let Some(memory_major_page_faults) = json["memory"]["majorPageFaults"].as_i64() {
            metric.set_memory_major_page_faults(memory_major_page_faults as i32);
        }

        if let Some(network_rx_bytes) = json["network"]["rxBytes"].as_i64() {
            metric.set_network_rx_bytes(network_rx_bytes);
        }

        if let Some(network_rx_errors) = json["network"]["rxErrors"].as_i64() {
            metric.set_network_rx_errors(network_rx_errors as i32);
        }

        if let Some(network_tx_bytes) = json["network"]["txBytes"].as_i64() {
            metric.set_network_tx_bytes(network_tx_bytes);
        }

        if let Some(network_tx_errors) = json["network"]["txErrors"].as_i64() {
            metric.set_network_tx_errors(network_tx_errors as i32);
        }

        if let Some(fs_available_bytes) = json["fs"]["availableBytes"].as_i64() {
            metric.set_fs_available_bytes(fs_available_bytes);
        }

        if let Some(fs_capacity_bytes) = json["fs"]["capacityBytes"].as_i64() {
            metric.set_fs_capacity_bytes(fs_capacity_bytes);
        }

        if let Some(fs_used_bytes) = json["fs"]["usedBytes"].as_i64() {
            metric.set_fs_used_bytes(fs_used_bytes);
        }

        if let Some(fs_inodes_free) = json["fs"]["inodesFree"].as_i64() {
            metric.set_fs_inodes_free(fs_inodes_free);
        }

        if let Some(fs_inodes) = json["fs"]["inodes"].as_i64() {
            metric.set_fs_inodes(fs_inodes);
        }

        if let Some(fs_inodes_used) = json["fs"]["inodesUsed"].as_i64() {
            metric.set_fs_inodes_used(fs_inodes_used);
        }

        if let Some(rlimit_maxpid) = json["rlimit"]["maxpid"].as_i64() {
            metric.set_rlimit_maxpid(rlimit_maxpid as i32);
        }

        if let Some(rlimit_curproc) = json["rlimit"]["curproc"].as_i64() {
            metric.set_rlimit_curproc(rlimit_curproc as i32);
        }

        if let Some(swap_usage_bytes) = json["swap"]["swapUsageBytes"].as_i64() {
            metric.set_swap_usage_bytes(swap_usage_bytes);
        }

        metric
    }

    pub fn from_pod_json(node_name: String, json: serde_json::Value) -> KubernetesMetrics {
        let mut metric = KubernetesMetrics::new();

        metric.set_node_name(node_name);

        if let Some(name) = json["podRef"]["name"].as_str() {
            metric.set_pod_name(name.to_string());
        }

        if let Some(namespace) = json["podRef"]["namespace"].as_str() {
            metric.set_pod_namespace(namespace.to_string());
        }

        if let Some(uid) = json["podRef"]["uid"].as_str() {
            metric.set_pod_uuid(uid.to_string());
        }

        metric.set_timestamp(now_timestamp());

        if let Some(cpu_usage_nano_cores) = json["cpu"]["usageNanoCores"].as_i64() {
            metric.set_cpu_usage_nano_cores(cpu_usage_nano_cores);
        }

        if let Some(cpu_usage_core_nano_seconds) = json["cpu"]["usageCoreNanoSeconds"].as_i64() {
            metric.set_cpu_usage_core_nano_seconds(cpu_usage_core_nano_seconds);
        }

        if let Some(memory_usage_bytes) = json["memory"]["usageBytes"].as_i64() {
            metric.set_memory_usage_bytes(memory_usage_bytes);
        }

        if let Some(memory_working_set_bytes) = json["memory"]["workingSetBytes"].as_i64() {
            metric.set_memory_working_set_bytes(memory_working_set_bytes);
        }

        if let Some(memory_rss_bytes) = json["memory"]["rssBytes"].as_i64() {
            metric.set_memory_rss_bytes(memory_rss_bytes);
        }

        if let Some(memory_page_faults) = json["memory"]["pageFaults"].as_i64() {
            metric.set_memory_page_faults(memory_page_faults as i32);
        }

        if let Some(memory_major_page_faults) = json["memory"]["majorPageFaults"].as_i64() {
            metric.set_memory_major_page_faults(memory_major_page_faults as i32);
        }

        if let Some(ephemeral_storage_available_bytes) =
            json["ephemeral-storage"]["availableBytes"].as_i64()
        {
            metric.set_ephemeral_storage_available_bytes(ephemeral_storage_available_bytes);
        }

        if let Some(ephemeral_storage_capacity_bytes) =
            json["ephemeral-storage"]["capacityBytes"].as_i64()
        {
            metric.set_ephemeral_storage_capacity_bytes(ephemeral_storage_capacity_bytes);
        }

        if let Some(ephemeral_storage_used_bytes) = json["ephemeral-storage"]["usedBytes"].as_i64()
        {
            metric.set_ephemeral_storage_used_bytes(ephemeral_storage_used_bytes);
        }

        if let Some(ephemeral_storage_inodes_free) =
            json["ephemeral-storage"]["InodesFree"].as_i64()
        {
            metric.set_ephemeral_storage_inodes_free(ephemeral_storage_inodes_free);
        }

        if let Some(ephemeral_storage_inodes) = json["ephemeral-storage"]["Inodes"].as_i64() {
            metric.set_ephemeral_storage_inodes(ephemeral_storage_inodes);
        }

        if let Some(ephemeral_storage_inodes_used) =
            json["ephemeral-storage"]["InodesUsed"].as_i64()
        {
            metric.set_ephemeral_storage_inodes_used(ephemeral_storage_inodes_used);
        }

        if let Some(process_count) = json["process_stats"]["process_count"].as_i64() {
            metric.set_process_count(process_count as i32);
        }

        if let Some(swap_usage_bytes) = json["swap"]["swapUsageBytes"].as_i64() {
            metric.set_swap_usage_bytes(swap_usage_bytes);
        }
        metric
    }

    pub fn from_volume_json(node_name: String, json: serde_json::Value) -> KubernetesMetrics {
        let mut metric = KubernetesMetrics::new();

        metric.set_node_name(node_name);

        if let Some(name) = json["name"].as_str() {
            metric.set_volume_name(name.to_string());
        }

        metric.set_timestamp(now_timestamp());

        if let Some(fs_available_bytes) = json["availableBytes"].as_i64() {
            metric.set_fs_available_bytes(fs_available_bytes);
        }

        if let Some(fs_capacity_bytes) = json["capacityBytes"].as_i64() {
            metric.set_fs_capacity_bytes(fs_capacity_bytes);
        }

        if let Some(fs_used_bytes) = json["usedBytes"].as_i64() {
            metric.set_fs_used_bytes(fs_used_bytes);
        }

        if let Some(fs_inodes_free) = json["inodesFree"].as_i64() {
            metric.set_fs_inodes_free(fs_inodes_free);
        }

        if let Some(fs_inodes) = json["inodes"].as_i64() {
            metric.set_fs_inodes(fs_inodes);
        }

        if let Some(fs_inodes_used) = json["inodesUsed"].as_i64() {
            metric.set_fs_inodes_used(fs_inodes_used);
        }

        metric
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
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
    let mut metrics = Vec::new();

    for node in nodes {
        let name = node.name_any();
        let url = format!("/api/v1/nodes/{}/proxy/stats/summary", name);

        let kube_request = Request::get(url).body(Default::default())?;
        let kube_response = kube_client
            .request::<serde_json::Value>(kube_request.clone())
            .await?;

        trace!("JSON: {:?}", kube_response);

        let node_metric = KubernetesMetrics::from_node_json(kube_response["node"].clone());
        metrics.push(node_metric.clone());

        trace!("Node: {:?}", node_metric);

        if let Some(pods) = kube_response["pods"].as_array() {
            for pod in pods {
                let pod_metric = KubernetesMetrics::from_pod_json(
                    kube_response["node"]["nodeName"].to_string(),
                    pod.clone(),
                );

                metrics.push(pod_metric.clone());

                trace!("Pod: {:?}", pod_metric);

                if let Some(volumes) = pod["volume"].as_array() {
                    for volume in volumes {
                        let volume_metric = KubernetesMetrics::from_volume_json(
                            kube_response["node"]["nodeName"].to_string(),
                            volume.clone(),
                        );

                        metrics.push(volume_metric.clone());

                        trace!("Volume: {:?}", volume_metric);
                    }
                }
            }
        };
    }

    let endpoint =
        env::var("APPSIGNAL_ENDPOINT").unwrap_or("https://appsignal-endpoint.net".to_owned());
    let api_key = env::var("APPSIGNAL_API_KEY").expect("APPSIGNAL_API_KEY not set");
    let base = Url::parse(&endpoint).expect("Could not parse endpoint");
    let path = format!("metrics/kubernetes?api_key={}", api_key);
    let url = base.join(&path).expect("Could not build request URL");

    let reqwest_client = Client::builder().timeout(Duration::from_secs(30)).build()?;

    for metric in metrics {
        let metric_bytes = metric.write_to_bytes().expect("Could not serialize metric");
        let appsignal_response = reqwest_client
            .post(url.clone())
            .body(metric_bytes)
            .send()
            .await?;

        debug!("Metric sent: {:?}", appsignal_response);
    }

    Ok(())
}

fn now_timestamp() -> i64 {
    let timestamp = time::now_utc().to_timespec().sec;
    timestamp - timestamp % 60
}

#[cfg(test)]
mod tests {
    use crate::KubernetesMetrics;
    use serde_json::json;
    use std::assert_eq;

    #[test]
    fn extract_node_metrics_with_empty_results() {
        let metric = KubernetesMetrics::from_node_json(json!([]));

        assert_eq!("", metric.node_name);
        assert!(metric.timestamp > 1736429031);
        assert!(metric.timestamp % 60 == 0);
    }

    #[test]
    fn extract_node_metrics_with_results() {
        let metric = KubernetesMetrics::from_node_json(json!({
          "nodeName": "node",
          "cpu": {
           "time": "2024-03-29T12:21:36Z",
           "usageNanoCores": 232839439,
           "usageCoreNanoSeconds": 1118592000000 as u64
          },
        }));

        assert_eq!("node", metric.node_name);
        assert_eq!(232839439, metric.cpu_usage_nano_cores);
    }

    #[test]
    fn extract_pod_metrics_with_empty_results() {
        let metric = KubernetesMetrics::from_pod_json("node".to_string(), json!([]));

        assert_eq!("node", metric.node_name);
        assert_eq!("", metric.pod_name);
        assert!(metric.timestamp > 1736429031);
        assert!(metric.timestamp % 60 == 0);
    }

    #[test]
    fn extract_pod_metrics_with_results() {
        let metric = KubernetesMetrics::from_pod_json(
            "node".to_string(),
            json!({
              "podRef": {
                "name": "kube-proxy-db7k4",
                "namespace": "kube-system",
                "uid": "3f3c1bf6-0fe9-4bc9-8cfb-965f36c485d8"
              },
              "cpu": {
                "time": "2024-03-29T12:21:36Z",
                "usageNanoCores": 232839439,
                "usageCoreNanoSeconds": 1118592000000 as u64
              },
            }),
        );

        assert_eq!("node", metric.node_name);
        assert_eq!("kube-proxy-db7k4", metric.pod_name);
        assert_eq!("kube-system", metric.pod_namespace);
        assert_eq!("3f3c1bf6-0fe9-4bc9-8cfb-965f36c485d8", metric.pod_uuid);
        assert_eq!(232839439, metric.cpu_usage_nano_cores);
        assert_eq!(1118592000000, metric.cpu_usage_core_nano_seconds);
    }

    #[test]
    fn extract_volume_metrics_with_empty_results() {
        let metric = KubernetesMetrics::from_volume_json("node".to_string(), json!([]));

        assert_eq!("node", metric.node_name);
        assert_eq!("", metric.volume_name);
        assert!(metric.timestamp > 1736429031);
        assert!(metric.timestamp % 60 == 0);
    }

    #[test]
    fn extract_volume_metrics_with_results() {
        let metric = KubernetesMetrics::from_volume_json(
            "node".to_string(),
            json!({
                "time": "2024-10-08T13:42:48Z",
                "availableBytes": 8318251008 as u64,
                "capacityBytes": 8318263296 as u64,
                "usedBytes": 12288,
                "inodesFree": 1015404,
                "inodes": 1015413,
                "inodesUsed": 9,
                "name": "kube-api-access-qz4b4"
            }),
        );

        assert_eq!("node", metric.node_name);
        assert_eq!("kube-api-access-qz4b4", metric.volume_name);
        assert_eq!(8318251008, metric.fs_available_bytes);
    }
}
