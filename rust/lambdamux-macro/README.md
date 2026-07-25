# lambdamux-macro

`lambdamux-macro` contains the procedural macros used by `lambdamux`.

It provides:

- `#[route(path = ..., method = ...)]`
- `generate_routes!()`

The route macro accepts both synchronous and asynchronous functions. Generated route-table code adapts both forms to the boxed-future handler representation used internally.

Most downstream users should consume these through the `lambdamux` facade crate.

## License

Licensed under either GPL-3.0-only or Apache-2.0, at your option.
