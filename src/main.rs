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
    pub fn from_node_json(json: serde_json::Value) -> Option<KubernetesMetrics> {
        match json["nodeName"].as_str() {
            Some(node_name) => {
                let mut metric = KubernetesMetrics::new();

                metric.set_node_name(node_name.to_string());

                metric.set_timestamp(now_timestamp());

                if let Some(cpu_usage_nano_cores) = json["cpu"]["usageNanoCores"].as_i64() {
                    metric.set_cpu_usage_nano_cores(cpu_usage_nano_cores);
                }

                if let Some(cpu_usage_core_nano_seconds) =
                    json["cpu"]["usageCoreNanoSeconds"].as_i64()
                {
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

                if let Some(swap_available_bytes) = json["swap"]["swapAvailableBytes"].as_i64() {
                    metric.set_swap_available_bytes(swap_available_bytes);
                }

                Some(metric)
            }
            _ => None,
        }
    }

    pub fn from_pod_json(
        node_name: Option<&str>,
        json: serde_json::Value,
    ) -> Option<KubernetesMetrics> {
        match (node_name, json["podRef"]["name"].as_str()) {
            (Some(node_name), Some(pod_name)) => {
                let mut metric = KubernetesMetrics::new();

                metric.set_node_name(node_name.to_string());
                metric.set_pod_name(pod_name.to_string());

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

                if let Some(cpu_usage_core_nano_seconds) =
                    json["cpu"]["usageCoreNanoSeconds"].as_i64()
                {
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

                if let Some(ephemeral_storage_used_bytes) =
                    json["ephemeral-storage"]["usedBytes"].as_i64()
                {
                    metric.set_ephemeral_storage_used_bytes(ephemeral_storage_used_bytes);
                }

                if let Some(ephemeral_storage_inodes_free) =
                    json["ephemeral-storage"]["inodesFree"].as_i64()
                {
                    metric.set_ephemeral_storage_inodes_free(ephemeral_storage_inodes_free);
                }

                if let Some(ephemeral_storage_inodes) = json["ephemeral-storage"]["inodes"].as_i64()
                {
                    metric.set_ephemeral_storage_inodes(ephemeral_storage_inodes);
                }

                if let Some(ephemeral_storage_inodes_used) =
                    json["ephemeral-storage"]["inodesUsed"].as_i64()
                {
                    metric.set_ephemeral_storage_inodes_used(ephemeral_storage_inodes_used);
                }

                if let Some(process_count) = json["process_stats"]["process_count"].as_i64() {
                    metric.set_process_count(process_count as i32);
                }

                if let Some(swap_usage_bytes) = json["swap"]["swapUsageBytes"].as_i64() {
                    metric.set_swap_usage_bytes(swap_usage_bytes);
                }

                Some(metric)
            }
            _ => None,
        }
    }

    pub fn from_volume_json(
        node_name: Option<&str>,
        json: serde_json::Value,
    ) -> Option<KubernetesMetrics> {
        match (node_name, json["name"].as_str()) {
            (Some(node_name), Some(volume_name)) => {
                let mut metric = KubernetesMetrics::new();

                metric.set_node_name(node_name.to_string());
                metric.set_volume_name(volume_name.to_string());

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

                Some(metric)
            }
            _ => None,
        }
    }

    pub fn delta(&self, previous: KubernetesMetrics) -> KubernetesMetrics {
        let mut new = self.clone();

        new.set_network_rx_bytes(new.get_network_rx_bytes() - previous.get_network_rx_bytes());

        new.set_network_rx_errors(new.get_network_rx_errors() - previous.get_network_rx_errors());

        new.set_network_tx_bytes(new.get_network_tx_bytes() - previous.get_network_tx_bytes());

        new.set_network_tx_errors(new.get_network_tx_errors() - previous.get_network_tx_errors());

        new
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let duration = Duration::new(60, 0);
    let mut interval = tokio::time::interval(duration);
    let mut previous = Vec::new();

    loop {
        interval.tick().await;

        match run(previous).await {
            Ok(results) => previous = results,
            Err(err) => panic!("Failed to extract metrics: {}", err),
        }
    }
}

async fn run(previous: Vec<KubernetesMetrics>) -> Result<Vec<KubernetesMetrics>, Error> {
    let kube_client = kube::Client::try_default().await?;
    let api: Api<Node> = Api::all(kube_client.clone());
    let nodes = api.list(&ListParams::default()).await?;
    let mut metrics = Vec::new();
    let mut payload = Vec::new();

    for node in nodes {
        let name = node.name_any();
        let url = format!("/api/v1/nodes/{}/proxy/stats/summary", name);

        let kube_request = Request::get(url).body(Default::default())?;
        let kube_response = kube_client
            .request::<serde_json::Value>(kube_request.clone())
            .await?;

        trace!("JSON: {:?}", kube_response);

        if let Some(node_metric) = KubernetesMetrics::from_node_json(kube_response["node"].clone())
        {
            if let Some(result) = previous.iter().find(|&p| {
                p.node_name == node_metric.node_name && p.volume_name == "" && p.pod_uuid == ""
            }) {
                payload.push(node_metric.delta(result.clone()));
            }

            metrics.push(node_metric.clone());

            trace!("Node: {:?}", node_metric);
        };

        if let Some(pods) = kube_response["pods"].as_array() {
            for pod in pods {
                if let Some(pod_metric) = KubernetesMetrics::from_pod_json(
                    kube_response["node"]["nodeName"].as_str(),
                    pod.clone(),
                ) {

                    if let Some(result) = previous.iter().find(|&p| {
                        p.pod_uuid == pod_metric.pod_uuid
                    }) {
                        payload.push(pod_metric.delta(result.clone()));
                    }

                    metrics.push(pod_metric.clone());

                    trace!("Pod: {:?}", pod_metric);
                };

                if let Some(volumes) = pod["volume"].as_array() {
                    for volume in volumes {
                        if let Some(volume_metric) = KubernetesMetrics::from_volume_json(
                            kube_response["node"]["nodeName"].as_str(),
                            volume.clone(),
                        ) {

                            if let Some(result) = previous.iter().find(|&p| {
                                p.volume_name == volume_metric.volume_name &&
                                p.node_name == volume_metric.node_name
                            }) {
                                payload.push(volume_metric.delta(result.clone()));
                            }

                            metrics.push(volume_metric.clone());

                            trace!("Volume: {:?}", volume_metric);
                        };
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

    for metric in &payload {
        let metric_bytes = metric.write_to_bytes().expect("Could not serialize metric");
        let appsignal_response = reqwest_client
            .post(url.clone())
            .body(metric_bytes)
            .send()
            .await?;

        debug!("Metric sent: {:?}", appsignal_response);
    }

    Ok(metrics)
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
    use std::fs::File;

    fn json() -> serde_json::Value {
        let file =
            File::open("test/fixtures/digitalocean.json").expect("Could not open example file");
        serde_json::from_reader(file).expect("Could not parse example file")
    }

    #[test]
    fn extract_node_metrics_with_empty_results() {
        assert_eq!(None, KubernetesMetrics::from_node_json(json!([])));
    }

    #[test]
    fn extract_node_metrics_with_results() {
        let metric = KubernetesMetrics::from_node_json(json()["node"].clone()).unwrap();

        assert_eq!("pool-k1f1it7zb-ekz6u", metric.node_name);

        assert!(metric.timestamp > 1736429031);
        assert!(metric.timestamp % 60 == 0);

        assert_eq!(44128133, metric.cpu_usage_nano_cores);
        assert_eq!(83361299610000, metric.cpu_usage_core_nano_seconds);

        assert_eq!(1039441920, metric.memory_available_bytes);
        assert_eq!(1565478912, metric.memory_usage_bytes);
        assert_eq!(1023315968, metric.memory_working_set_bytes);
        assert_eq!(450433024, metric.memory_rss_bytes);
        assert_eq!(97153339, metric.memory_page_faults);
        assert_eq!(3780, metric.memory_major_page_faults);

        assert_eq!(6011987255, metric.network_rx_bytes);
        assert_eq!(42, metric.network_rx_errors);
        assert_eq!(5541026205, metric.network_tx_bytes);
        assert_eq!(42, metric.network_tx_errors);

        assert_eq!(36804550656, metric.fs_available_bytes);
        assert_eq!(52666433536, metric.fs_capacity_bytes);
        assert_eq!(13682970624, metric.fs_used_bytes);
        assert_eq!(3148894, metric.fs_inodes_free);
        assert_eq!(3268608, metric.fs_inodes);
        assert_eq!(119714, metric.fs_inodes_used);

        assert_eq!(15432, metric.rlimit_maxpid);
        assert_eq!(363, metric.rlimit_curproc);

        assert_eq!(42, metric.swap_usage_bytes);
        assert_eq!(42, metric.swap_available_bytes);
    }

    #[test]
    fn extract_node_metrics_with_swap_data() {
        let metric = KubernetesMetrics::from_node_json(json!({
          "nodeName": "node",
          "swap": {
              "time": "2025-01-31T10:57:18Z",
              "swapAvailableBytes": 10465738752 as u64,
              "swapUsageBytes": 1024 as u64
          }
        }))
        .unwrap();

        assert_eq!("node", metric.node_name);
        assert_eq!(1024, metric.swap_usage_bytes);
        assert_eq!(10465738752, metric.swap_available_bytes);
    }

    #[test]
    fn extract_pod_metrics_with_empty_results() {
        assert_eq!(None, KubernetesMetrics::from_pod_json(None, json!([])));
    }

    #[test]
    fn extract_pod_metrics_with_results() {
        let metric =
            KubernetesMetrics::from_pod_json(Some("node"), json()["pods"][0].clone()).unwrap();

        assert_eq!("node", metric.node_name);
        assert_eq!("konnectivity-agent-8qf4d", metric.pod_name);
        assert_eq!("kube-system", metric.pod_namespace);
        assert_eq!("eba341db-5f3c-4cbf-9f2d-1ca9e926c7e4", metric.pod_uuid);

        assert!(metric.timestamp > 1736429031);
        assert!(metric.timestamp % 60 == 0);

        assert_eq!(409594, metric.cpu_usage_nano_cores);
        assert_eq!(631022780000, metric.cpu_usage_core_nano_seconds);

        assert_eq!(10760192, metric.memory_usage_bytes);
        assert_eq!(10702848, metric.memory_working_set_bytes);
        assert_eq!(9584640, metric.memory_rss_bytes);
        assert_eq!(2832, metric.memory_page_faults);
        assert_eq!(7, metric.memory_major_page_faults);

        assert_eq!(2732202444, metric.network_rx_bytes);
        assert_eq!(42, metric.network_rx_errors);
        assert_eq!(2814051624, metric.network_tx_bytes);
        assert_eq!(42, metric.network_tx_errors);

        assert_eq!(36804550656, metric.ephemeral_storage_available_bytes);
        assert_eq!(52666433536, metric.ephemeral_storage_capacity_bytes);
        assert_eq!(15360000, metric.ephemeral_storage_used_bytes);
        assert_eq!(3148894, metric.ephemeral_storage_inodes_free);
        assert_eq!(3268608, metric.ephemeral_storage_inodes);
        assert_eq!(17, metric.ephemeral_storage_inodes_used);

        assert_eq!(1, metric.process_count);

        assert_eq!(42, metric.swap_usage_bytes);
    }

    #[test]
    fn extract_volume_metrics_with_empty_results() {
        assert_eq!(None, KubernetesMetrics::from_volume_json(None, json!([])));
    }

    #[test]
    fn extract_volume_metrics_with_results() {
        let metric = KubernetesMetrics::from_volume_json(
            Some("node"),
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
        )
        .unwrap();

        assert_eq!("node", metric.node_name);
        assert_eq!("kube-api-access-qz4b4", metric.volume_name);

        assert!(metric.timestamp > 1736429031);
        assert!(metric.timestamp % 60 == 0);

        assert_eq!(8318251008, metric.fs_available_bytes);
    }

    #[test]
    fn delta_subtracts_network_data() {
        let metric = KubernetesMetrics::from_node_json(json()["node"].clone())
            .clone()
            .unwrap();

        let new = metric.delta(metric.clone());

        assert_eq!(0, new.network_rx_bytes);
        assert_eq!(0, new.network_rx_errors);
        assert_eq!(0, new.network_tx_bytes);
        assert_eq!(0, new.network_tx_errors);
    }
}
