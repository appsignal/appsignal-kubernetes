---
bump: patch
type: add
---

Report container statuses. This allows for more accurate reporting of error states that are not necessarily reflected in the pod phase, such as when a container is waiting to be restarted after a crash loop.
