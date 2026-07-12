# lambdamux-macro

`lambdamux-macro` contains the procedural macros used by `lambdamux`.

It provides:

- `#[route(path = ..., method = ...)]`
- `generate_routes!()`

Most downstream users should consume these through the `lambdamux` facade crate.

## License

Licensed under either GPL-3.0-only or Apache-2.0, at your option.
