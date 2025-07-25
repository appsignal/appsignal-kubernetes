mod ownership;

extern crate time;

use appsignal_transmitter::reqwest::{Client, Response, Url};
use http::Request;
use k8s_openapi::api::core::v1::{Node, Pod};
use kube::api::ListParams;
use kube::{Api, ResourceExt};
use log::{info, trace, warn};
use protobuf::Message;
use std::env;
use std::time::Duration;

mod protocol {
    #![allow(renamed_and_removed_lints)]
    include!("../protocol/mod.rs");
}

use protocol::kubernetes::{
    Container, ContainerStatus, KubernetesMetrics, KubernetesMetricsBatch, OwnerReference, PodPhase,
};

use crate::ownership::{OwnershipResolver, ResourceIdentifier};

// The threshold for metrics batch size, in bytes.
// The body of a request to AppSignal will only exceed this threshold
// if a single metric somehow exceeds it.
const BATCH_SIZE_THRESHOLD: u32 = 400_000;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

impl KubernetesMetrics {
    pub fn from_node_json(json: serde_json::Value) -> Option<KubernetesMetrics> {
        match json["nodeName"].as_str() {
            Some(node_name) => {
                let mut metric = KubernetesMetrics::new();

                metric.set_node_name(node_name.to_string());

                metric.set_timestamp(now_timestamp());

                if let Some(cpu_usage_nano_cores) = Self::extract_i64(&json, "/cpu/usageNanoCores")
                {
                    metric.set_cpu_usage_nano_cores(cpu_usage_nano_cores);
                }

                if let Some(cpu_usage_core_nano_seconds) =
                    Self::extract_i64(&json, "/cpu/usageCoreNanoSeconds")
                {
                    metric.set_cpu_usage_core_nano_seconds(cpu_usage_core_nano_seconds);
                }

                if let Some(memory_available_bytes) =
                    Self::extract_i64(&json, "/memory/availableBytes")
                {
                    metric.set_memory_available_bytes(memory_available_bytes);
                }

                if let Some(memory_usage_bytes) = Self::extract_i64(&json, "/memory/usageBytes") {
                    metric.set_memory_usage_bytes(memory_usage_bytes);
                }

                if let Some(memory_working_set_bytes) =
                    Self::extract_i64(&json, "/memory/workingSetBytes")
                {
                    metric.set_memory_working_set_bytes(memory_working_set_bytes);
                }

                if let Some(memory_rss_bytes) = Self::extract_i64(&json, "/memory/rssBytes") {
                    metric.set_memory_rss_bytes(memory_rss_bytes);
                }

                if let Some(memory_page_faults) = Self::extract_i64(&json, "/memory/pageFaults") {
                    metric.set_memory_page_faults(memory_page_faults as i32);
                }

                if let Some(memory_major_page_faults) =
                    Self::extract_i64(&json, "/memory/majorPageFaults")
                {
                    metric.set_memory_major_page_faults(memory_major_page_faults as i32);
                }

                if let (
                    Some(memory_available_bytes),
                    Some(memory_usage_bytes),
                    Some(memory_rss_bytes),
                ) = (
                    Self::extract_f64(&json, "/memory/availableBytes"),
                    Self::extract_f64(&json, "/memory/usageBytes"),
                    Self::extract_f64(&json, "/memory/rssBytes"),
                ) {
                    metric.set_memory_usage(Self::percentage_from(
                        memory_usage_bytes - memory_rss_bytes,
                        memory_usage_bytes + memory_available_bytes - memory_rss_bytes,
                    ));
                }

                if let Some(network_rx_bytes) = Self::extract_i64(&json, "/network/rxBytes") {
                    metric.set_network_rx_bytes(network_rx_bytes);
                }

                if let Some(network_rx_errors) = Self::extract_i64(&json, "/network/rxErrors") {
                    metric.set_network_rx_errors(network_rx_errors as i32);
                }

                if let Some(network_tx_bytes) = Self::extract_i64(&json, "/network/txBytes") {
                    metric.set_network_tx_bytes(network_tx_bytes);
                }

                if let Some(network_tx_errors) = Self::extract_i64(&json, "/network/txErrors") {
                    metric.set_network_tx_errors(network_tx_errors as i32);
                }

                if let Some(fs_available_bytes) = Self::extract_i64(&json, "/fs/availableBytes") {
                    metric.set_fs_available_bytes(fs_available_bytes);
                }

                if let Some(fs_capacity_bytes) = Self::extract_i64(&json, "/fs/capacityBytes") {
                    metric.set_fs_capacity_bytes(fs_capacity_bytes);
                }

                if let Some(fs_used_bytes) = Self::extract_i64(&json, "/fs/usedBytes") {
                    metric.set_fs_used_bytes(fs_used_bytes);
                }

                if let (Some(fs_capacity_bytes), Some(fs_used_bytes)) = (
                    Self::extract_f64(&json, "/fs/capacityBytes"),
                    Self::extract_f64(&json, "/fs/usedBytes"),
                ) {
                    metric.set_disk_usage(Self::percentage_from(fs_used_bytes, fs_capacity_bytes));
                }

                if let Some(fs_inodes_free) = Self::extract_i64(&json, "/fs/inodesFree") {
                    metric.set_fs_inodes_free(fs_inodes_free);
                }

                if let Some(fs_inodes) = Self::extract_i64(&json, "/fs/inodes") {
                    metric.set_fs_inodes(fs_inodes);
                }

                if let Some(fs_inodes_used) = Self::extract_i64(&json, "/fs/inodesUsed") {
                    metric.set_fs_inodes_used(fs_inodes_used);
                }

                if let Some(rlimit_maxpid) = Self::extract_i64(&json, "/rlimit/maxpid") {
                    metric.set_rlimit_maxpid(rlimit_maxpid as i32);
                }

                if let Some(rlimit_curproc) = Self::extract_i64(&json, "/rlimit/curproc") {
                    metric.set_rlimit_curproc(rlimit_curproc as i32);
                }

                if let Some(swap_usage_bytes) = Self::extract_i64(&json, "/swap/swapUsageBytes") {
                    metric.set_swap_usage_bytes(swap_usage_bytes);
                }

                if let Some(swap_available_bytes) =
                    Self::extract_i64(&json, "/swap/swapAvailableBytes")
                {
                    metric.set_swap_available_bytes(swap_available_bytes);
                }

                if let (Some(swap_available_bytes), Some(swap_usage_bytes)) = (
                    Self::extract_f64(&json, "/swap/swapAvailableBytes"),
                    Self::extract_f64(&json, "/swap/swapUsageBytes"),
                ) {
                    metric.set_swap_usage(Self::percentage_from(
                        swap_usage_bytes,
                        swap_usage_bytes + swap_available_bytes,
                    ));
                }

                Some(metric)
            }
            _ => None,
        }
    }

    pub fn from_pod_api(pod: &Pod) -> Option<KubernetesMetrics> {
        let pod_name = pod.metadata.name.as_ref()?;
        let pod_namespace = pod.metadata.namespace.as_ref()?;
        let pod_uuid = pod.metadata.uid.as_ref()?;
        let node_name = pod.spec.as_ref()?.node_name.as_ref()?;

        let mut metric = KubernetesMetrics::new();

        metric.set_node_name(node_name.to_string());
        metric.set_pod_name(pod_name.to_string());
        metric.set_pod_namespace(pod_namespace.to_string());
        metric.set_pod_uuid(pod_uuid.to_string());
        metric.set_timestamp(now_timestamp());

        // Extract phase
        if let Some(status) = &pod.status {
            if let Some(phase) = &status.phase {
                let pod_phase = match phase.as_str() {
                    "Pending" => PodPhase::POD_PHASE_PENDING,
                    "Running" => PodPhase::POD_PHASE_RUNNING,
                    "Succeeded" => PodPhase::POD_PHASE_SUCCEEDED,
                    "Failed" => PodPhase::POD_PHASE_FAILED,
                    _ => PodPhase::POD_PHASE_UNKNOWN,
                };
                metric.set_pod_phase(pod_phase);
            }
        }

        Some(metric)
    }

    pub fn enrich_with_stats(&mut self, json: serde_json::Value) {
        if let Some(cpu_usage_nano_cores) = Self::extract_i64(&json, "/cpu/usageNanoCores") {
            self.set_cpu_usage_nano_cores(cpu_usage_nano_cores);
        }

        if let Some(cpu_usage_core_nano_seconds) =
            Self::extract_i64(&json, "/cpu/usageCoreNanoSeconds")
        {
            self.set_cpu_usage_core_nano_seconds(cpu_usage_core_nano_seconds);
        }

        if let Some(memory_usage_bytes) = Self::extract_i64(&json, "/memory/usageBytes") {
            self.set_memory_usage_bytes(memory_usage_bytes);
        }

        if let Some(memory_working_set_bytes) = Self::extract_i64(&json, "/memory/workingSetBytes")
        {
            self.set_memory_working_set_bytes(memory_working_set_bytes);
        }

        if let Some(memory_rss_bytes) = Self::extract_i64(&json, "/memory/rssBytes") {
            self.set_memory_rss_bytes(memory_rss_bytes);
        }

        if let Some(memory_page_faults) = Self::extract_i64(&json, "/memory/pageFaults") {
            self.set_memory_page_faults(memory_page_faults as i32);
        }

        if let Some(memory_major_page_faults) = Self::extract_i64(&json, "/memory/majorPageFaults")
        {
            self.set_memory_major_page_faults(memory_major_page_faults as i32);
        }

        if let Some(network_rx_bytes) = Self::extract_i64(&json, "/network/rxBytes") {
            self.set_network_rx_bytes(network_rx_bytes);
        }

        if let Some(network_rx_errors) = Self::extract_i64(&json, "/network/rxErrors") {
            self.set_network_rx_errors(network_rx_errors as i32);
        }

        if let Some(network_tx_bytes) = Self::extract_i64(&json, "/network/txBytes") {
            self.set_network_tx_bytes(network_tx_bytes);
        }

        if let Some(network_tx_errors) = Self::extract_i64(&json, "/network/txErrors") {
            self.set_network_tx_errors(network_tx_errors as i32);
        }

        if let Some(ephemeral_storage_available_bytes) =
            Self::extract_i64(&json, "/ephemeral-storage/availableBytes")
        {
            self.set_ephemeral_storage_available_bytes(ephemeral_storage_available_bytes);
        }

        if let Some(ephemeral_storage_capacity_bytes) =
            Self::extract_i64(&json, "/ephemeral-storage/capacityBytes")
        {
            self.set_ephemeral_storage_capacity_bytes(ephemeral_storage_capacity_bytes);
        }

        if let Some(ephemeral_storage_used_bytes) =
            Self::extract_i64(&json, "/ephemeral-storage/usedBytes")
        {
            self.set_ephemeral_storage_used_bytes(ephemeral_storage_used_bytes);
        }

        if let Some(ephemeral_storage_inodes_free) =
            Self::extract_i64(&json, "/ephemeral-storage/inodesFree")
        {
            self.set_ephemeral_storage_inodes_free(ephemeral_storage_inodes_free);
        }

        if let Some(ephemeral_storage_inodes) =
            Self::extract_i64(&json, "/ephemeral-storage/inodes")
        {
            self.set_ephemeral_storage_inodes(ephemeral_storage_inodes);
        }

        if let Some(ephemeral_storage_inodes_used) =
            Self::extract_i64(&json, "/ephemeral-storage/inodesUsed")
        {
            self.set_ephemeral_storage_inodes_used(ephemeral_storage_inodes_used);
        }

        if let Some(process_count) = Self::extract_i64(&json, "/process_stats/process_count") {
            self.set_process_count(process_count as i32);
        }

        if let Some(swap_usage_bytes) = Self::extract_i64(&json, "/swap/swapUsageBytes") {
            self.set_swap_usage_bytes(swap_usage_bytes);
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

    pub fn is_node(&self) -> bool {
        !self.is_pod() && !self.is_volume()
    }

    pub fn is_pod(&self) -> bool {
        !self.pod_uuid.is_empty()
    }

    pub fn is_volume(&self) -> bool {
        !self.volume_name.is_empty()
    }

    pub fn delta(&self, previous: KubernetesMetrics) -> KubernetesMetrics {
        let mut new = self.clone();

        new.set_network_rx_bytes(new.get_network_rx_bytes() - previous.get_network_rx_bytes());

        new.set_network_rx_errors(new.get_network_rx_errors() - previous.get_network_rx_errors());

        new.set_network_tx_bytes(new.get_network_tx_bytes() - previous.get_network_tx_bytes());

        new.set_network_tx_errors(new.get_network_tx_errors() - previous.get_network_tx_errors());

        new
    }

    pub fn delta_from(&self, previous: Vec<KubernetesMetrics>) -> Option<KubernetesMetrics> {
        previous
            .iter()
            .find(|&p| {
                (self.is_pod() && p.pod_uuid == self.pod_uuid)
                    || (self.is_node() && p.node_name == self.node_name)
                    || (self.is_volume() && p.volume_name == self.volume_name)
            })
            .map(|previous| self.delta(previous.clone()))
    }

    pub fn extract_containers(&mut self, pods: &kube::api::ObjectList<Pod>) {
        if let Some(pod_data) = pods.iter().find(|pod| match &pod.metadata.name {
            Some(name) => name == &self.pod_name,
            _ => false,
        }) {
            if let Some(status) = &pod_data.status {
                if let Some(container_statuses) = &status.container_statuses {
                    let mut proto_containers = protobuf::RepeatedField::new();

                    for container_status in container_statuses {
                        let mut proto_container = Container::new();

                        // Set container name
                        proto_container.set_name(container_status.name.clone());

                        // Extract container status, reason, and exit code based on container state
                        if let Some(ref state) = container_status.state {
                            if let Some(ref _running) = state.running {
                                proto_container
                                    .set_status(ContainerStatus::CONTAINER_STATUS_RUNNING);
                                // Running state doesn't have reason or exit_code
                            } else if let Some(ref waiting) = state.waiting {
                                proto_container
                                    .set_status(ContainerStatus::CONTAINER_STATUS_WAITING);
                                if let Some(ref reason) = waiting.reason {
                                    proto_container.set_reason(reason.clone());
                                }
                                // Waiting state doesn't have exit_code
                            } else if let Some(ref terminated) = state.terminated {
                                proto_container
                                    .set_status(ContainerStatus::CONTAINER_STATUS_TERMINATED);
                                if let Some(ref reason) = terminated.reason {
                                    proto_container.set_reason(reason.clone());
                                }
                                proto_container.set_exit_code(terminated.exit_code);
                            }
                        } else {
                            // If no state is present, default to unknown
                            proto_container.set_status(ContainerStatus::CONTAINER_STATUS_UNKNOWN);
                        }

                        proto_containers.push(proto_container);
                    }

                    self.set_containers(proto_containers);
                }
            }
        };
    }

    pub fn extract_pod_restart_count_and_uptime(&mut self, pods: &kube::api::ObjectList<Pod>) {
        if let Some(pod_data) = pods.iter().find(|pod| match &pod.metadata.name {
            Some(name) => name == &self.pod_name,
            _ => false,
        }) {
            if let Some(status) = &pod_data.status {
                // Calculate restart count from container statuses
                let mut total_restart_count = 0;
                if let Some(container_statuses) = &status.container_statuses {
                    for container_status in container_statuses {
                        total_restart_count += container_status.restart_count;
                    }
                }
                self.set_pod_restart_count(total_restart_count);

                // Calculate uptime from pod start time
                if let Some(start_time) = &status.start_time {
                    let now = chrono::Utc::now();
                    if let Ok(start_time_parsed) =
                        chrono::DateTime::parse_from_rfc3339(&start_time.0.to_rfc3339())
                    {
                        let uptime_duration = now
                            .signed_duration_since(start_time_parsed.with_timezone(&chrono::Utc));
                        let uptime_seconds = uptime_duration.num_seconds().max(0);
                        self.set_pod_uptime_seconds(uptime_seconds);
                    }
                }
            }
        };
    }

    pub fn extract_pod_labels(&mut self, pods: &kube::api::ObjectList<Pod>) {
        if let Some(pod_data) = pods.iter().find(|pod| match &pod.metadata.name {
            Some(name) => name == &self.pod_name,
            _ => false,
        }) {
            if let Some(labels) = &pod_data.metadata.labels {
                let mut labels_map = std::collections::HashMap::new();
                for (key, value) in labels {
                    labels_map.insert(key.clone(), value.clone());
                }
                self.set_labels(labels_map);
            }
        };
    }

    pub fn extract_node_labels(&mut self, nodes: &kube::api::ObjectList<Node>) {
        if let Some(node_data) = nodes.iter().find(|node| match &node.metadata.name {
            Some(name) => name == &self.node_name,
            _ => false,
        }) {
            if let Some(labels) = &node_data.metadata.labels {
                let mut labels_map = std::collections::HashMap::new();
                for (key, value) in labels {
                    labels_map.insert(key.clone(), value.clone());
                }
                self.set_labels(labels_map);
            }
        };
    }

    pub async fn extract_owner_references(
        &mut self,
        resolver: &mut OwnershipResolver,
        pods: &kube::api::ObjectList<Pod>,
    ) -> Result<(), Error> {
        if let Some(pod) = pods.iter().find(|pod| match &pod.metadata.name {
            Some(name) => name == &self.pod_name,
            _ => false,
        }) {
            let resource = ResourceIdentifier::from_pod(pod);
            let owner_references = resolver.resolve_top_level_owners(&resource).await?;
            for owner_reference in owner_references {
                let mut protocol_owner_reference = OwnerReference::new();
                protocol_owner_reference.set_name(owner_reference.name);
                protocol_owner_reference.set_kind(owner_reference.gvk.kind);
                if let Some(namespace) = owner_reference.namespace {
                    protocol_owner_reference.set_namespace(namespace);
                }

                self.mut_owner_references().push(protocol_owner_reference);
            }
        };

        Ok(())
    }

    fn extract_i64(data: &serde_json::Value, path: &str) -> Option<i64> {
        Self::extract(data, path)?.as_i64()
    }

    fn extract_f64(data: &serde_json::Value, path: &str) -> Option<f64> {
        Self::extract(data, path)?.as_f64()
    }

    fn extract(data: &serde_json::Value, path: &str) -> Option<serde_json::Value> {
        let value = data.pointer(path);
        let number = value?.as_i64()?;

        if number.is_negative() {
            warn!("Unexpected negative value for {}: {}", path, number);

            None
        } else {
            value.cloned()
        }
    }

    fn percentage_from(value: f64, total: f64) -> i32 {
        (value / total * 100.0).clamp(0.0, 100.0).round() as i32
    }
}

#[derive(Debug)]
struct Config {
    endpoint: String,
    api_key: String,
}

impl Config {
    fn from_env() -> Config {
        Config {
            api_key: env::var("APPSIGNAL_API_KEY").expect("APPSIGNAL_API_KEY not set"),
            endpoint: env::var("APPSIGNAL_ENDPOINT")
                .unwrap_or("https://appsignal-endpoint.net".to_owned()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    info!(
        "Starting Appsignal for Kubernetes with configuration: {:?}",
        Config::from_env()
    );

    let client = kube::Client::try_default().await?;
    let mut resolver = OwnershipResolver::new(client.clone());
    let duration = Duration::new(60, 0);
    let mut interval = tokio::time::interval(duration);
    let mut previous = Vec::new();

    loop {
        interval.tick().await;

        match run(&client, &mut resolver, previous).await {
            Ok(results) => previous = results,
            Err(err) => {
                warn!("Failed to extract and report metrics: {}", err);
                previous = Vec::new();
            }
        }

        resolver.reset();
    }
}

async fn run(
    client: &kube::Client,
    resolver: &mut OwnershipResolver,
    previous: Vec<KubernetesMetrics>,
) -> Result<Vec<KubernetesMetrics>, Error> {
    info!("Extracting metrics from Kubernetes cluster");

    let nodes: Api<Node> = Api::all(client.clone());
    let nodes_list = nodes.list(&ListParams::default()).await?;

    let pods: Api<Pod> = Api::all(client.clone());
    let pods_list = pods.list(&ListParams::default()).await?;

    let mut metrics = Vec::new();
    let mut payload = Vec::new();

    // Collect all stats/summary data from all nodes
    let mut stats_data = std::collections::HashMap::new();

    for node in &nodes_list {
        let name = node.name_any();
        let url = format!("/api/v1/nodes/{}/proxy/stats/summary", name);

        let kube_request = Request::get(url).body(Default::default())?;
        let kube_response = client
            .request::<serde_json::Value>(kube_request.clone())
            .await?;

        trace!("JSON: {:?}", kube_response);

        // Process node metrics
        if let Some(mut node_metric) =
            KubernetesMetrics::from_node_json(kube_response["node"].clone())
        {
            node_metric.extract_node_labels(&nodes_list);

            if let Some(metric) = node_metric.delta_from(previous.clone()) {
                payload.push(metric);
            }

            metrics.push(node_metric.clone());

            trace!("Node: {:?}", node_metric);
        };

        // Store stats data for later pod enrichment
        if let Some(pods) = kube_response["pods"].as_array() {
            for pod in pods {
                if let Some(pod_ref) = pod.get("podRef") {
                    if let Some(pod_uid) = pod_ref.get("uid").and_then(|uid| uid.as_str()) {
                        stats_data.insert(pod_uid.to_string(), pod.clone());
                    }
                }
            }
        }

        // Process volume metrics
        if let Some(pods) = kube_response["pods"].as_array() {
            for pod in pods {
                if let Some(volumes) = pod["volume"].as_array() {
                    for volume in volumes {
                        if let Some(volume_metric) = KubernetesMetrics::from_volume_json(
                            kube_response["node"]["nodeName"].as_str(),
                            volume.clone(),
                        ) {
                            if let Some(metric) = volume_metric.delta_from(previous.clone()) {
                                payload.push(metric);
                            }

                            metrics.push(volume_metric.clone());

                            trace!("Volume: {:?}", volume_metric);
                        };
                    }
                }
            }
        }
    }

    // Process all pods from Kubernetes API
    for pod in &pods_list {
        if let Some(mut pod_metric) = KubernetesMetrics::from_pod_api(pod) {
            // Extract additional metadata from Kubernetes API
            pod_metric.extract_containers(&pods_list);
            pod_metric.extract_pod_labels(&pods_list);
            pod_metric.extract_pod_restart_count_and_uptime(&pods_list);

            if let Err(err) = pod_metric
                .extract_owner_references(resolver, &pods_list)
                .await
            {
                warn!(
                    "Failed to extract owner references for pod {}: {}",
                    pod_metric.pod_name, err
                );
            }

            // Enrich with stats data if available
            if let Some(stats_json) = stats_data.get(&pod_metric.pod_uuid) {
                pod_metric.enrich_with_stats(stats_json.clone());
            }

            if let Some(metric) = pod_metric.delta_from(previous.clone()) {
                payload.push(metric);
            }

            metrics.push(pod_metric.clone());

            trace!("Pod: {:?}", pod_metric);
        }
    }

    info!("Extracted {} metrics", metrics.len());

    let config = Config::from_env();
    let base = Url::parse(&config.endpoint).expect("Could not parse endpoint");
    let path = format!("metrics/kubernetes?api_key={}", config.api_key);
    let url = base.join(&path).expect("Could not build request URL");

    let reqwest_client = Client::builder().timeout(Duration::from_secs(30)).build()?;

    info!("Sending {} metrics to Appsignal", payload.len());

    let mut batch_message = KubernetesMetricsBatch::new();
    for metric in payload.into_iter() {
        batch_message.mut_metrics().push(metric);
        if batch_message.compute_size() > BATCH_SIZE_THRESHOLD {
            // Remove the metric that went over the threshold from the batch,
            // unless it is the only metric in the batch.
            let metric = if batch_message.mut_metrics().len() > 1 {
                metrics.pop()
            } else {
                None
            };

            let batch = std::mem::take(&mut batch_message);
            send_batch(batch, &url, &reqwest_client).await?;

            // Add the metric that went over the threshold to the next batch.
            if let Some(metric) = metric {
                batch_message.mut_metrics().push(metric);
            }
        }
    }

    if !batch_message.get_metrics().is_empty() {
        send_batch(batch_message, &url, &reqwest_client).await?;
    }

    info!("All metrics sent");

    Ok(metrics)
}

async fn send_batch(
    batch: KubernetesMetricsBatch,
    url: &Url,
    client: &Client,
) -> Result<Response, Error> {
    let batch_bytes = batch.write_to_bytes().expect("Could not serialize batch");

    let response = client.post(url.clone()).body(batch_bytes).send().await?;

    info!(
        "Batch of {} metrics sent: {:?}",
        batch.get_metrics().len(),
        response.status()
    );

    Ok(response)
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

    fn digitalocean_fixture() -> serde_json::Value {
        let file =
            File::open("test/fixtures/digitalocean.json").expect("Could not open example file");
        serde_json::from_reader(file).expect("Could not parse example file")
    }

    fn negative_fixture() -> serde_json::Value {
        let file = File::open("test/fixtures/negative.json").expect("Could not open example file");
        serde_json::from_reader(file).expect("Could not parse example file")
    }

    #[test]
    fn extract_node_metrics_with_empty_results() {
        assert_eq!(None, KubernetesMetrics::from_node_json(json!([])));
    }

    #[test]
    fn extract_node_metrics_with_results() {
        let metric =
            KubernetesMetrics::from_node_json(digitalocean_fixture()["node"].clone()).unwrap();

        assert_eq!("pool-k1f1it7zb-ekz6u", metric.node_name);

        assert!(metric.is_node());
        assert!(!metric.is_pod());
        assert!(!metric.is_volume());

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

        assert_eq!(52, metric.memory_usage);

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

        assert_eq!(26, metric.disk_usage);

        assert_eq!(15432, metric.rlimit_maxpid);
        assert_eq!(363, metric.rlimit_curproc);

        assert_eq!(42, metric.swap_usage_bytes);
        assert_eq!(42, metric.swap_available_bytes);

        assert_eq!(50, metric.swap_usage);
    }

    #[test]
    fn extract_node_metrics_with_zero_disk_capacity_bytes() {
        let metric = KubernetesMetrics::from_node_json(json!({
          "nodeName": "node",
          "fs": {
              "capacityBytes": 0_u64,
              "usedBytes": 1024_u64
          }
        }))
        .unwrap();

        assert_eq!(100, metric.disk_usage);
    }

    #[test]
    fn extract_node_metrics_with_zero_disk_used_bytes() {
        let metric = KubernetesMetrics::from_node_json(json!({
          "nodeName": "node",
          "fs": {
              "capacityBytes": 1024_u64,
              "usedBytes": 0_u64
          }
        }))
        .unwrap();

        assert_eq!(0, metric.disk_usage);
    }

    #[test]
    fn extract_node_metrics_with_more_used_disk_bytes_than_capacity_bytes() {
        let metric = KubernetesMetrics::from_node_json(json!({
          "nodeName": "node",
          "fs": {
              "capacityBytes": 1024_u64,
              "usedBytes": 2048_u64
          }
        }))
        .unwrap();

        assert_eq!(100, metric.disk_usage);
    }

    #[test]
    fn extract_node_metrics_with_negative_disk_capacity_bytes() {
        let metric = KubernetesMetrics::from_node_json(json!({
          "nodeName": "node",
          "fs": {
              "capacityBytes": -1024_i64,
              "usedBytes": 1024_u64
          }
        }))
        .unwrap();

        assert_eq!(0, metric.disk_usage);
    }

    #[test]
    fn extract_node_metrics_with_negative_disk_used_bytes() {
        let metric = KubernetesMetrics::from_node_json(json!({
          "nodeName": "node",
          "fs": {
              "capacityBytes": 1024_u64,
              "usedBytes": -1024_i64
          }
        }))
        .unwrap();

        assert_eq!(0, metric.disk_usage);
    }

    #[test]
    fn extract_node_metrics_with_negative_memory_available_bytes() {
        let metric = KubernetesMetrics::from_node_json(json!({
          "nodeName": "node",
          "memory": {
              "availableBytes": -1024_i64,
              "usageBytes": 512_u64
          }
        }))
        .unwrap();

        assert_eq!(0, metric.memory_usage);
    }

    #[test]
    fn extract_node_metrics_with_swap_data() {
        let metric = KubernetesMetrics::from_node_json(json!({
          "nodeName": "node",
          "swap": {
              "time": "2025-01-31T10:57:18Z",
              "swapAvailableBytes": 10465738752_u64,
              "swapUsageBytes": 1024_u64
          }
        }))
        .unwrap();

        assert_eq!("node", metric.node_name);
        assert_eq!(1024, metric.swap_usage_bytes);
        assert_eq!(10465738752, metric.swap_available_bytes);
    }

    #[test]
    fn extract_node_metrics_with_negative_results() {
        let metric = KubernetesMetrics::from_node_json(negative_fixture()["node"].clone()).unwrap();

        assert_eq!("pool-k1f1it7zb-ekz6u", metric.node_name);

        assert_eq!(0, metric.cpu_usage_nano_cores);
        assert_eq!(0, metric.cpu_usage_core_nano_seconds);

        assert_eq!(0, metric.memory_available_bytes);
        assert_eq!(0, metric.memory_usage_bytes);
        assert_eq!(0, metric.memory_working_set_bytes);
        assert_eq!(0, metric.memory_rss_bytes);
        assert_eq!(0, metric.memory_page_faults);
        assert_eq!(0, metric.memory_major_page_faults);

        assert_eq!(0, metric.memory_usage);

        assert_eq!(0, metric.network_rx_bytes);
        assert_eq!(0, metric.network_rx_errors);
        assert_eq!(0, metric.network_tx_bytes);
        assert_eq!(0, metric.network_tx_errors);

        assert_eq!(0, metric.fs_available_bytes);
        assert_eq!(0, metric.fs_capacity_bytes);
        assert_eq!(0, metric.fs_used_bytes);
        assert_eq!(0, metric.fs_inodes_free);
        assert_eq!(0, metric.fs_inodes);
        assert_eq!(0, metric.fs_inodes_used);

        assert_eq!(0, metric.disk_usage);

        assert_eq!(0, metric.rlimit_maxpid);
        assert_eq!(0, metric.rlimit_curproc);

        assert_eq!(0, metric.swap_usage_bytes);
        assert_eq!(0, metric.swap_available_bytes);

        assert_eq!(0, metric.swap_usage);
    }

    #[test]
    fn enrich_with_stats_from_fixture() {
        use k8s_openapi::api::core::v1::{Pod, PodSpec};

        // Create a base pod metric
        let mut pod = Pod::default();
        pod.metadata.name = Some("konnectivity-agent-8qf4d".to_string());
        pod.metadata.namespace = Some("kube-system".to_string());
        pod.metadata.uid = Some("eba341db-5f3c-4cbf-9f2d-1ca9e926c7e4".to_string());

        let pod_spec = PodSpec {
            node_name: Some("node".to_string()),
            ..Default::default()
        };
        pod.spec = Some(pod_spec);

        let mut metric = KubernetesMetrics::from_pod_api(&pod).unwrap();

        // Enrich with stats from the fixture
        metric.enrich_with_stats(digitalocean_fixture()["pods"][0].clone());

        assert_eq!("node", metric.node_name);
        assert_eq!("konnectivity-agent-8qf4d", metric.pod_name);
        assert_eq!("kube-system", metric.pod_namespace);
        assert_eq!("eba341db-5f3c-4cbf-9f2d-1ca9e926c7e4", metric.pod_uuid);

        assert!(!metric.is_node());
        assert!(metric.is_pod());
        assert!(!metric.is_volume());

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
                "availableBytes": 8318251008_u64,
                "capacityBytes": 8318263296_u64,
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

        assert!(!metric.is_node());
        assert!(!metric.is_pod());
        assert!(metric.is_volume());

        assert!(metric.timestamp > 1736429031);
        assert!(metric.timestamp % 60 == 0);

        assert_eq!(8318251008, metric.fs_available_bytes);
    }

    #[test]
    fn delta_subtracts_network_data() {
        let metric =
            KubernetesMetrics::from_node_json(digitalocean_fixture()["node"].clone()).unwrap();

        let new = metric.delta(metric.clone());

        assert_eq!(0, new.network_rx_bytes);
        assert_eq!(0, new.network_rx_errors);
        assert_eq!(0, new.network_tx_bytes);
        assert_eq!(0, new.network_tx_errors);
    }

    #[test]
    fn delta_from_node() {
        let node = KubernetesMetrics::from_node_json(json!({
            "nodeName": "node",
            "network": {
                "rxBytes": 6011987255_u64,
            }
        }))
        .unwrap();

        let previous = vec![
            KubernetesMetrics::from_node_json(json!({
                "nodeName": "other_node",
                "network": {
                    "rxBytes": 6011987255_u64,
                }
            }))
            .unwrap(),
            KubernetesMetrics::from_node_json(json!({
                "nodeName": "node",
                "network": {
                    "rxBytes": 6011987250_u64,
                }
            }))
            .unwrap(),
        ];

        let new = node.delta_from(previous).unwrap();

        assert_eq!(5, new.network_rx_bytes);
    }

    #[test]
    fn delta_from_pod() {
        use k8s_openapi::api::core::v1::{Pod, PodSpec};

        // Create current pod metric
        let mut pod = Pod::default();
        pod.metadata.name = Some("pod".to_string());
        pod.metadata.namespace = Some("default".to_string());
        pod.metadata.uid = Some("eba341db-5f3c-4cbf-9f2d-1ca9e926c7e4".to_string());

        let pod_spec = PodSpec {
            node_name: Some("node".to_string()),
            ..Default::default()
        };
        pod.spec = Some(pod_spec);

        let mut current = KubernetesMetrics::from_pod_api(&pod).unwrap();
        current.enrich_with_stats(json!({
            "network": {
                "rxBytes": 2732202444_u64,
            }
        }));

        // Create previous pod metrics
        let mut previous_different = current.clone();
        previous_different.set_pod_uuid("differen-tuid-4cbf-9f2d-1ca9e926c7e4".to_string());
        previous_different.set_network_rx_bytes(2732202444);

        let mut previous_same = current.clone();
        previous_same.set_network_rx_bytes(2732202440);

        let previous = vec![previous_different, previous_same];

        let new = current.delta_from(previous).unwrap();

        assert_eq!(4, new.network_rx_bytes);
    }

    #[test]
    fn enrich_with_stats_handles_negative_values() {
        use k8s_openapi::api::core::v1::{Pod, PodSpec};

        // Create a base pod metric
        let mut pod = Pod::default();
        pod.metadata.name = Some("test-pod".to_string());
        pod.metadata.namespace = Some("default".to_string());
        pod.metadata.uid = Some("test-uid".to_string());

        let pod_spec = PodSpec {
            node_name: Some("node".to_string()),
            ..Default::default()
        };
        pod.spec = Some(pod_spec);

        let mut metric = KubernetesMetrics::from_pod_api(&pod).unwrap();

        // Enrich with negative fixture data
        metric.enrich_with_stats(negative_fixture()["pods"][0].clone());

        assert_eq!(0, metric.cpu_usage_nano_cores);
        assert_eq!(0, metric.cpu_usage_core_nano_seconds);

        assert_eq!(0, metric.memory_usage_bytes);
        assert_eq!(0, metric.memory_working_set_bytes);
        assert_eq!(0, metric.memory_rss_bytes);
        assert_eq!(0, metric.memory_page_faults);
        assert_eq!(0, metric.memory_major_page_faults);

        assert_eq!(0, metric.network_rx_bytes);
        assert_eq!(0, metric.network_rx_errors);
        assert_eq!(0, metric.network_tx_bytes);
        assert_eq!(0, metric.network_tx_errors);

        assert_eq!(0, metric.ephemeral_storage_available_bytes);
        assert_eq!(0, metric.ephemeral_storage_capacity_bytes);
        assert_eq!(0, metric.ephemeral_storage_used_bytes);
        assert_eq!(0, metric.ephemeral_storage_inodes_free);
        assert_eq!(0, metric.ephemeral_storage_inodes);
        assert_eq!(0, metric.ephemeral_storage_inodes_used);

        assert_eq!(0, metric.process_count);

        assert_eq!(0, metric.swap_usage_bytes);
    }

    #[test]
    fn from_pod_api_creates_base_metrics() {
        use k8s_openapi::api::core::v1::{Pod, PodSpec};

        let mut pod = Pod::default();
        pod.metadata.name = Some("test-pod".to_string());
        pod.metadata.namespace = Some("default".to_string());
        pod.metadata.uid = Some("test-uid-123".to_string());

        let pod_spec = PodSpec {
            node_name: Some("test-node".to_string()),
            ..Default::default()
        };
        pod.spec = Some(pod_spec);

        let pod_status = k8s_openapi::api::core::v1::PodStatus {
            phase: Some("Running".to_string()),
            ..Default::default()
        };
        pod.status = Some(pod_status);

        let metric = KubernetesMetrics::from_pod_api(&pod).unwrap();

        assert_eq!("test-pod", metric.pod_name);
        assert_eq!("default", metric.pod_namespace);
        assert_eq!("test-uid-123", metric.pod_uuid);
        assert_eq!("test-node", metric.node_name);
        assert_eq!(
            crate::protocol::kubernetes::PodPhase::POD_PHASE_RUNNING,
            metric.pod_phase
        );
        assert!(metric.is_pod());
        assert!(!metric.is_node());
        assert!(!metric.is_volume());

        // Should have default values for stats-based metrics
        assert_eq!(0, metric.cpu_usage_nano_cores);
        assert_eq!(0, metric.memory_usage_bytes);
        assert_eq!(0, metric.network_rx_bytes);
    }

    #[test]
    fn enrich_with_stats_adds_metrics() {
        use k8s_openapi::api::core::v1::{Pod, PodSpec};

        let mut pod = Pod::default();
        pod.metadata.name = Some("test-pod".to_string());
        pod.metadata.namespace = Some("default".to_string());
        pod.metadata.uid = Some("test-uid-123".to_string());

        let pod_spec = PodSpec {
            node_name: Some("test-node".to_string()),
            ..Default::default()
        };
        pod.spec = Some(pod_spec);

        let mut metric = KubernetesMetrics::from_pod_api(&pod).unwrap();

        let stats_json = json!({
            "cpu": {
                "usageNanoCores": 1000000,
                "usageCoreNanoSeconds": 2000000000
            },
            "memory": {
                "usageBytes": 1048576,
                "workingSetBytes": 2097152
            },
            "network": {
                "rxBytes": 1024,
                "txBytes": 2048
            }
        });

        metric.enrich_with_stats(stats_json);

        assert_eq!(1000000, metric.cpu_usage_nano_cores);
        assert_eq!(2000000000, metric.cpu_usage_core_nano_seconds);
        assert_eq!(1048576, metric.memory_usage_bytes);
        assert_eq!(2097152, metric.memory_working_set_bytes);
        assert_eq!(1024, metric.network_rx_bytes);
        assert_eq!(2048, metric.network_tx_bytes);
    }
}
