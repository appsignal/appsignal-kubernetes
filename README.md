# AppSignal for Kubernetes

Extracts Kubenetes Node data.

## Installation

In a Kubernetes cluster, set up your AppSignal API key (find your *Front-end* API key in [App settings](https://appsignal.com/redirect-to/app?to=info)) by creating a secret:

    kubectl create secret generic appsignal --from-literal=api-key=00000000-0000-0000-0000-000000000000

Then, add the AppSignal deployment to your cluster:

    kubectl apply https://raw.githubusercontent.com/appsignal/appsignal-kubernetes/deployment/deployment.yaml

AppSignal for Kubernetes will start sending Kubernetes automatically.

## Metrics

AppSignal for Kubernetes extracts metrics for all nodes running in a cluster every minute.

It extracts the following metrics from the `/api/v1/nodes/<NODE>/proxy/stats/summary` endpoint:

- node_cpu_usage_nano_cores
- node_cpu_usage_core_nano_seconds
- node_memory_usage_bytes
- node_memory_working_set_bytes
- node_memory_rss_bytes
- node_memory_page_faults
- node_memory_major_page_faults
- node_network_rx_bytes
- node_network_rx_errors
- node_network_tx_bytes
- node_network_tx_errors
- node_fs_available_bytes
- node_s_capacity_bytes
- node_fs_inodes_free
- node_fs_inodes
- node_fs_inodes_used
- node_rlimit_maxpid
- node_rlimit_curproc
- node_swap_available_bytes
- node_swap_usage_bytes

## Dashboard

To see your metrics, add a custom dashboard with the reported fields.

Here's an example of a dashboard showing all currently reported values:

``` json
{
  "title": "k8s",
  "description": "",
  "visuals": [
    {
      "title": "Node CPU Usage",
      "description": "usageNanoCores, usageCoreNanoSeconds",
      "line_label": "%name% %node%",
      "display": "LINE",
      "format": "number",
      "draw_null_as_zero": true,
      "metrics": [
        {
          "name": "node_cpu_usage_nano_cores",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        },
        {
          "name": "node_cpu_usage_core_nano_seconds",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        }
      ],
      "type": "timeseries"
    },
    {
      "title": "Node Memory Usage",
      "description": "usageBytes, workingSetBytes, rssBytes, pageFaults, majorPageFaults",
      "line_label": "%name% %node%",
      "display": "LINE",
      "format": "number",
      "draw_null_as_zero": true,
      "metrics": [
        {
          "name": "node_memory_usage_bytes",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        },
        {
          "name": "node_memory_working_set_bytes",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        },
        {
          "name": "node_memory_rss_bytes",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        },
        {
          "name": "node_memory_page_faults",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        },
        {
          "name": "node_memory_major_page_faults",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        }
      ],
      "type": "timeseries"
    },
    {
      "title": "Node Network Usage",
      "description": "rxBytes, rxErrors, txBytes, txErrors",
      "line_label": "%name% %node%",
      "display": "LINE",
      "format": "number",
      "draw_null_as_zero": true,
      "metrics": [
        {
          "name": "node_network_rx_bytes",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        },
        {
          "name": "node_network_rx_errors",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        },
        {
          "name": "node_network_tx_bytes",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        },
        {
          "name": "node_network_tx_errors",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        }
      ],
      "type": "timeseries"
    },
    {
      "title": "Node File System Usage",
      "description": "availableBytes, capacityBytes, usedBytes",
      "line_label": "%name% %node%",
      "display": "LINE",
      "format": "number",
      "draw_null_as_zero": true,
      "metrics": [
        {
          "name": "node_fs_available_bytes",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        },
        {
          "name": "node_fs_capacity_bytes",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        },
        {
          "name": "node_fs_used_bytes",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        }
      ],
      "type": "timeseries"
    },
    {
      "title": "Node Inodes Usage",
      "description": "inodes, inodesFree, inodesUsed",
      "line_label": "%name% %node%",
      "display": "LINE",
      "format": "number",
      "draw_null_as_zero": true,
      "metrics": [
        {
          "name": "node_fs_inodes",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        },
        {
          "name": "node_fs_inodes_free",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        },
        {
          "name": "node_fs_inodes_used",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        }
      ],
      "type": "timeseries"
    },
    {
      "title": "Node Resource Limits",
      "description": "maxpid, curproc",
      "line_label": "%name% %node%",
      "display": "LINE",
      "format": "number",
      "draw_null_as_zero": true,
      "metrics": [
        {
          "name": "node_rlimit_maxpid",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        },
        {
          "name": "node_rlimit_curproc",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        }
      ],
      "type": "timeseries"
    },
    {
      "title": "Node Swap",
      "description": "swapAvailableBytes, swapUsageBytes",
      "line_label": "%name% %node%",
      "display": "LINE",
      "format": "number",
      "draw_null_as_zero": true,
      "metrics": [
        {
          "name": "node_swap_available_bytes",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        },
        {
          "name": "node_swap_usage_bytes",
          "fields": [
            {
              "field": "GAUGE"
            }
          ],
          "tags": [
            {
              "key": "node",
              "value": "*"
            }
          ]
        }
      ],
      "type": "timeseries"
    }
  ]
}
```
