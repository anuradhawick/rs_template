# lambdamux

`lambdamux` is the user-facing crate in this workspace.

It provides:

- attribute-based route registration with `#[route(...)]`
- compile-time route table generation with `generate_routes!()`
- API Gateway v1 and v2 adapters that hide the internal route cache

Typical downstream usage depends only on `lambdamux` and writes handlers with route annotations.
