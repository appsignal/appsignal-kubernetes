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
    ) -> Option<AppsignalMetric> {
        value.as_f64().map(|value| Self {
            name: metric_name.to_string(),
            metric_type: "gauge".to_string(),
            value: value as f32,
            tags,
        })
    }
}

#[tokio::main]
async fn main() {
    let duration = Duration::new(60, 0);
    let mut interval = tokio::time::interval(duration);

    loop {
        interval.tick().await;
        if let Err(error) = run().await {
            eprintln!("Failed to extract metrics: {}", &error);
        };
    }
}

async fn run() -> Result<(), Error> {
    let kube_client = kube::Client::try_default().await?;
    let api: Api<Node> = Api::all(kube_client.clone());
    let nodes = api.list(&ListParams::default()).await?;
    let mut out = Vec::new();

    for node in nodes {
        let url = format!("/api/v1/nodes/{}/proxy/stats/summary", node.name_any());

        let kube_request = Request::get(url).body(Default::default())?;
        let kube_response = kube_client
            .request::<serde_json::Value>(kube_request)
            .await?;

        extract_metrics(&kube_response, &mut out);
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

fn extract_metrics(kube_response: &Value, out: &mut Vec<AppsignalMetric>) {
    extract_node_metrics(&kube_response["node"], out);

    if let Some(pods) = kube_response["pods"].as_array() {
        for pod in pods {
            extract_pod_metrics(pod, out);

            if let Some(volumes) = pod["volume"].as_array() {
                for volume in volumes {
                    extract_volume_metrics(volume, out);
                }
            }
        }
    };
}

fn extract_volume_metrics(results: &Value, out: &mut Vec<AppsignalMetric>) {
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
        let mut tags = HashMap::with_capacity(1);
        tags.insert("volume".to_owned(), volume_name.to_owned());

        if let Some(metric) = AppsignalMetric::new(metric_name, tags, metric_value) {
            out.push(metric);
        }
    }
}

fn extract_pod_metrics(pod_results: &Value, out: &mut Vec<AppsignalMetric>) {
    let pod_name = if let Some(name) = pod_results["podRef"]["name"].as_str() {
        name
    } else {
        eprintln!("Could not extract pod name");
        return;
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
        let mut tags = HashMap::with_capacity(1);
        tags.insert("pod".to_owned(), pod_name.to_owned());

        if let Some(metric) = AppsignalMetric::new(metric_name, tags, metric_value) {
            out.push(metric);
        }
    }
}

fn extract_node_metrics(node_results: &Value, out: &mut Vec<AppsignalMetric>) {
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
        let mut tags = HashMap::with_capacity(1);
        tags.insert("node".to_owned(), node_name.to_owned());

        if let Some(metric) = AppsignalMetric::new(metric_name, tags, metric_value) {
            out.push(metric);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use serde_json::json;

    #[test]
    fn extract_metrics_without_response() {
      let mut out = Vec::new();
      extract_metrics(
          &json!({}),
          &mut out,
      );
      assert_eq!(out.len(), 0);
    }

    #[test]
    fn extract_metrics_with_response() {
      let mut out = Vec::new();
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
      assert!(out.contains(&AppsignalMetric::new(
          "node_cpu_usage_nano_cores",
          HashMap::from([("node".to_string(), "some_node".to_string())]),
          &json!(111111111)
      ).expect("Could not create metric")), "node_cpu_usage_nano_cores not found");
      assert!(out.contains(&AppsignalMetric::new(
          "pod_cpu_usage_nano_cores",
          HashMap::from([("pod".to_string(), "some_pod".to_string())]),
          &json!(222222222)
      ).expect("Could not create metric")), "pod_cpu_usage_nano_cores for some_pod not found");
      assert!(out.contains(&AppsignalMetric::new(
          "pod_cpu_usage_nano_cores",
          HashMap::from([("pod".to_string(), "other_pod".to_string())]),
          &json!(333333333)
      ).expect("Could not create metric")), "pod_cpu_usage_nano_cores for other_pod not found");
      assert!(out.contains(&AppsignalMetric::new(
          "volume_available_bytes",
          HashMap::from([("volume".to_string(), "some_volume".to_string())]),
          &json!(444444444)
      ).expect("Could not create metric")), "volume_available_bytes for some_volume not found");
      assert!(out.contains(&AppsignalMetric::new(
          "volume_available_bytes",
          HashMap::from([("volume".to_string(), "other_volume".to_string())]),
          &json!(555555555)
      ).expect("Could not create metric")), "volume_available_bytes for other_volume not found");
    }

    #[test]
    fn extract_node_metrics_without_results() {
        let mut out = Vec::new();
        extract_node_metrics(&json!({}), &mut out);
        assert_eq!(out.len(), 0);
    }

    #[test]
    fn extract_node_metrics_missing_name() {
        let mut out = Vec::new();
        extract_node_metrics(
            &json!({
              "cpu": {
               "time": "2024-03-29T12:21:36Z",
               "usageNanoCores": 232839439,
               "usageCoreNanoSeconds": 1118592000000 as u64
              },
            }),
            &mut out,
        );

        assert_eq!(out.len(), 0);
    }

    #[test]
    fn extract_node_metrics_with_results() {
        let mut out = Vec::new();
        extract_node_metrics(
            &json!({
              "nodeName": "some_node",
              "cpu": {
               "time": "2024-03-29T12:21:36Z",
               "usageNanoCores": 232839439,
               "usageCoreNanoSeconds": 1118592000000 as u64
              },
            }),
            &mut out,
        );

        assert_eq!(
            AppsignalMetric::new(
                "node_cpu_usage_nano_cores",
                HashMap::from([("node".to_string(), "some_node".to_string())]),
                &json!(232839439)
            )
            .expect("Could not create metric"),
            out[0]
        );

        assert_eq!(out.len(), 2);
    }

    #[test]
    fn extract_pod_metrics_without_results() {
        let mut out = Vec::new();
        extract_pod_metrics(&json!({}), &mut out);
        assert_eq!(out.len(), 0);
    }

    #[test]
    fn extract_pod_metrics_missing_name() {
        let mut out = Vec::new();
        extract_pod_metrics(
            &json!({
              "cpu": {
               "time": "2024-03-29T12:21:36Z",
               "usageNanoCores": 232839439,
               "usageCoreNanoSeconds": 1118592000000 as u64
              },
            }),
            &mut out,
        );

        assert_eq!(out.len(), 0);
    }

    #[test]
    fn extract_pod_metrics_with_results() {
        let mut out = Vec::new();
        extract_pod_metrics(
            &json!({
              "podRef": {
                  "name": "some_pod"
              },
              "cpu": {
               "time": "2024-03-29T12:21:36Z",
               "usageNanoCores": 232839439,
               "usageCoreNanoSeconds": 1118592000000 as u64
              },
            }),
            &mut out,
        );

        assert_eq!(
            AppsignalMetric::new(
                "pod_cpu_usage_nano_cores",
                HashMap::from([("pod".to_string(), "some_pod".to_string())]),
                &json!(232839439)
            )
            .expect("Could not create metric"),
            out[0]
        );

        assert_eq!(out.len(), 2);
    }

    #[test]
    fn extract_volume_metrics_without_results() {
        let mut out = Vec::new();
        extract_volume_metrics(&json!({}), &mut out);

        assert_eq!(out.len(), 0);
    }

    #[test]
    fn extract_volume_metrics_missing_name() {
        let mut out = Vec::new();
        extract_volume_metrics(
            &json!({
              "availableBytes": 232839439,
              "capacityBytes": 1118592000000 as u64,
            }),
            &mut out,
        );

        assert_eq!(out.len(), 0);
    }

    #[test]
    fn extract_volume_metrics_with_results() {
        let mut out = Vec::new();
        extract_volume_metrics(
            &json!({
              "name": "some_volume",
              "availableBytes": 232839439,
              "capacityBytes": 1118592000000 as u64,
            }),
            &mut out,
        );

        assert_eq!(
            AppsignalMetric::new(
                "volume_available_bytes",
                HashMap::from([("volume".to_string(), "some_volume".to_string())]),
                &json!(232839439)
            )
            .expect("Could not create metric"),
            out[0]
        );

        assert_eq!(out.len(), 2);
    }
}
