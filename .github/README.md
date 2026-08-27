# Repository automation policy

This repository is in bootstrap and intentionally contains no GitHub Actions workflows yet.

A workflow may be added only when all of the following are true:

1. the underlying command exists and is reproducible locally;
2. the workflow enforces a documented roadmap or release gate;
3. inputs, caches, permissions, and artifacts are explicit;
4. failure has a clear owner and remediation path;
5. the workflow does not publish, release, or mutate external systems without a separate accepted decision.

Decorative CI, scheduled jobs, release automation, CodeQL, Dependabot, Pages, and status bots are not enabled by convention during bootstrap. E7 defines the planned production automation milestone.
