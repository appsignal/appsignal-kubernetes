# AppSignal for Kubernetes

AppSignal for Kubernetes is an agent that collects and sends metrics about your Kubernetes cluster to your AppSignal account.

## Installation

First, set up your AppSignal API key (find your _App-specific_ API key in [App settings](https://appsignal.com/redirect-to/app?to=info)) by creating a secret:

    kubectl create secret generic appsignal --from-literal=api-key=00000000-0000-0000-0000-000000000000 --namespace appsignal

### Using kubectl

Install AppSignal for Kubernetes by applying the deployment manifest:

    kubectl apply -f https://raw.githubusercontent.com/appsignal/appsignal-kubernetes/main/deployment.yaml

This will create the `appsignal` namespace and deploy the AppSignal for Kubernetes agent in that namespace.

### Using Helm

Add the AppSignal Helm repository and install the chart:

    helm repo add appsignal-kubernetes https://appsignal.github.io/appsignal-kubernetes
    helm install appsignal-kubernetes appsignal-kubernetes/appsignal-kubernetes --create-namespace --namespace appsignal

## Cluster Metrics

AppSignal for Kubernetes will start sending Kubernetes metrics automatically.

After installing AppSignal for Kubernetes, AppSignal's Host Metrics are automatically replaced with Cluster Metrics to display cluster metrics.

## Development

### Publish new releases

To publish a new release, follow these steps:

- Trigger a new workflow via this [workflow](https://github.com/appsignal/appsignal-kubernetes/actions/workflows/publish_release.yaml).

The last tag will used as the new version published to the [appsignal/appsignal-kubernetes](https://hub.docker.com/repository/docker/appsignal/appsignal-kubernetes/tags) image on Docker Hub.

## Support

Please don't hesitate to [contact us](mailto:support@appsignal.com) if we can assist you in getting AppSignal for Kubernetes setup.
