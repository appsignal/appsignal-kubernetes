---
bump: patch
type: add
---

When a request to AppSignal returns a `401 Unauthorized` error status code, display a log message asking the customer to ensure they're using an app-level push API key.