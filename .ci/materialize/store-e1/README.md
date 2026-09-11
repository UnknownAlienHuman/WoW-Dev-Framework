# `wow-store`

`wow-store` is the E1 persistent content-addressed storage owner.

- blobs are addressed by SHA-256 and rehashed on every read;
- immutable snapshots bind normalized logical paths to exact blob identities and lengths;
- snapshot manifests are canonical JSON and content-addressed;
- publication refs are append-only checksummed journals protected by an inter-process lock;
- compare-and-swap rejects stale writers;
- a torn trailing journal record is ignored, while complete corruption fails closed;
- existing content-addressed files are never overwritten with different bytes.

The root directory is explicit and locally trusted. This crate does not discover projects, parse Lua, choose a client/profile, run migrations for higher-level schemas, expose SQL, perform network access, or decide retention/garbage collection.
