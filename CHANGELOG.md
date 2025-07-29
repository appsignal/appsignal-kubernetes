# AppSignal for Kubernetes Changelog

## 1.2.2

_Published on 2025-07-29._

### Fixed

- Fix Helm chart version. Ensure the default value for the AppSignal for Kubernetes version to install for a given Helm chart release matches the AppSignal for Kubernetes version being released. (patch [1645706](https://github.com/appsignal/appsignal-kubernetes/commit/164570616bd6b1e9ff2909067d33073da2117688))

## 1.2.1

_Published on 2025-07-29._

### Fixed

- Fix Helm chart namespace creation. The chart mistakenly contained its own `appsignal` namespace resource, which interfered with the use of Helm's `--namespace` and `--create-namespace` flags. The chart now correctly uses the namespace specified by the user without creating an additional one. (patch [7a163b1](https://github.com/appsignal/appsignal-kubernetes/commit/7a163b1b02e371971b7ddc9b7361f46dde73595b))

## 1.2.0

_Published on 2025-07-23._

### Added

- Report labels for pods and nodes (patch [f469c3d](https://github.com/appsignal/appsignal-kubernetes/commit/f469c3d851079da57e85ad928dac8767e010df3f))
- Report pod uptime and restart count (patch [f469c3d](https://github.com/appsignal/appsignal-kubernetes/commit/f469c3d851079da57e85ad928dac8767e010df3f))
- Report container statuses. This allows for more accurate reporting of error states that are not necessarily reflected in the pod phase, such as when a container is waiting to be restarted after a crash loop. (patch [8fb4993](https://github.com/appsignal/appsignal-kubernetes/commit/8fb4993c434469e615e5557d6bbadff0319d406e))
- Report top-level owners for pods. When reporting information about a pod to AppSignal, report its top-level owner resources, such as a Deployment or Job. (patch [a3f0e17](https://github.com/appsignal/appsignal-kubernetes/commit/a3f0e1709f86535e43cb4214be6a5335b18169bd))

### Changed

- Report metrics in batches. This should improve performance for clusters with lots of Kubernetes resources. (minor [c7b13ca](https://github.com/appsignal/appsignal-kubernetes/commit/c7b13ca770fb511efd7c376e225a428969ceeac4))

### Fixed

- Report metrics for all pods. Before this fix, metrics for a pod would only be reported if one or more of its containers was currently running. (patch [9ead1c5](https://github.com/appsignal/appsignal-kubernetes/commit/9ead1c5e0b3fde5c4ba9c07e7ab35403053649ad))
- Fix Helm releases. Helm releases versioned 1.1.2 and below would install version 1.0.0 of the AppSignal Kubernetes integration, which did not include the latest changes. This release ensures that the correct version is installed. (patch [a3f0e17](https://github.com/appsignal/appsignal-kubernetes/commit/a3f0e1709f86535e43cb4214be6a5335b18169bd))

## 1.1.2

_Published on 2025-05-08._

### Fixed

- Fix permissions for accessing pod metrics. (patch [310c32a](https://github.com/appsignal/appsignal-kubernetes/commit/310c32a5575386c8c62852cad809f2a4c6f4018a))

## 1.1.1

_Published on 2025-04-17._

### Added

- Version bump without internal changes (patch [d1c159c](https://github.com/appsignal/appsignal-kubernetes/commit/d1c159c98529909657a6f366eea8d86d5aa2e7de))

## 1.1.0

_Published on 2025-04-14._

### Added

- Add per-pod phase to sent metrics (minor [5ed49ca](https://github.com/appsignal/appsignal-kubernetes/commit/5ed49cac0c4394d32aad3fe3f2d919fa57244cae), [d729c11](https://github.com/appsignal/appsignal-kubernetes/commit/d729c1145a6a0a02228bdb0a470951730a0749ca))

## 1.0.1

_Published on 2025-03-27._

### Added

- Print configuration on startup to aid in debugging (patch [2645bd3](https://github.com/appsignal/appsignal-kubernetes/commit/2645bd307a77fb1cca6a4a45d2771d743da6ad64))

### Fixed

- Ensure reported values are always positive (patch [59abd58](https://github.com/appsignal/appsignal-kubernetes/commit/59abd583d1b3da7cc7fa8219258fff7821745103))

## 0.2.0

_Published on 2024-09-27._

### Added

- Report volume metrics. For each volume in a pod, report metrics about its available and used capacity, tagged by the name of the volume and the name of the pod in which it is mounted.

## 0.1.0

_Published on 2024-05-03._

Initial release 🚀

When installed in your Kubernetes cluster, AppSignal for Kubernetes will report metrics about your Kubernetes nodes and pods.
