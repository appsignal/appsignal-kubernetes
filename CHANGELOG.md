# AppSignal for Kubernetes Changelog

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
