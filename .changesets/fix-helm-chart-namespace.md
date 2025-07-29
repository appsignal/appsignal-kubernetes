---
bump: patch
type: fix
---

Fix Helm chart namespace creation. The chart mistakenly contained its own `appsignal` namespace resource, which interfered with the use of Helm's `--namespace` and `--create-namespace` flags. The chart now correctly uses the namespace specified by the user without creating an additional one.
