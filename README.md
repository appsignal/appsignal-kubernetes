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

## Automated Dashboard

After installing AppSignal for Kubernetes into a cluster, an Automated Dashboard automatically appears on AppSignal showing you an overview of the nodes in your Kubernetes cluster.
The reported metrics can also be used to create custom dashboards through the [Dashboard and Graph Builder](https://appsignal.com/redirect-to/app?to=dashboard&overlay=dashboardForm).

## Support

Please don't hesitate to [contact us](mailto:support@appsignal.com) if we can assist you in getting AppSignal for Kubernetes setup.
