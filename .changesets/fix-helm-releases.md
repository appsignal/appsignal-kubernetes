---
bump: patch
type: fix
---

Fix Helm releases. Helm releases versioned 1.1.2 and below would install version 1.0.0 of the AppSignal Kubernetes integration, which did not include the latest changes. This release ensures that the correct version is installed.
