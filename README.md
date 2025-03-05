# AppSignal for Kubernetes

Extracts Kubenetes Cluster Metrics.

## Installation

In a Kubernetes cluster, set up your AppSignal API key by creating a secret:

    kubectl create secret generic appsignal --from-literal=api-key=00000000-0000-0000-0000-000000000000

Then, add the AppSignal deployment to your cluster:

    kubectl apply -f https://raw.githubusercontent.com/appsignal/appsignal-kubernetes/main/deployment.yaml

AppSignal for Kubernetes will start sending Kubernetes automatically.

## Cluster Metrics

After installing AppSignal for Kubernetes into a cluster, AppSignal's Host Metrics are automatically replaced with Cluster Metrics to display cluster metrics.

## Development

### Publish new releases

To publish a new release, follow these steps:

- Trigger a new workflow via this [workflow](https://github.com/appsignal/appsignal-kubernetes/actions/workflows/publish_release.yaml).

The last tag will used as the new version published to the [appsignal/appsignal-kubernetes](https://hub.docker.com/repository/docker/appsignal/appsignal-kubernetes/tags) image on Docker Hub.

## Support

Please don't hesitate to [contact us](mailto:support@appsignal.com) if we can assist you in getting AppSignal for Kubernetes setup.
