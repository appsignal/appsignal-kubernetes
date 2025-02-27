## Usage

[Helm](https://helm.sh) must be installed to use the charts.  Please refer to
Helm's [documentation](https://helm.sh/docs) to get started.

Once Helm has been set up correctly, add the repo as follows:

```
helm repo add appsignal-kubernetes https://appsignal.github.io/appsignal-kubernetes
```

If you had already added this repo earlier, run `helm repo update` to retrieve
the latest versions of the packages.  You can then run `helm search repo
appsignal-kubernetes` to see the charts.

To install the appsignal-kubernetes chart:

```
helm install my-appsignal-kubernetes appsignal-kubernetes/appsignal-kubernetes
```

To uninstall the chart:

```
helm uninstall my-appsignal-kubernetes
```
