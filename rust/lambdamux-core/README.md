# lambdamux-core

`lambdamux-core` contains the generic routing primitives used by `lambdamux`.

It provides the trie-based matcher and boxed-future handler type aliases without depending on AWS Lambda event types. The `lambdamux` macros adapt both synchronous and asynchronous route functions to this internal handler representation.

## License

Licensed under either GPL-3.0-only or Apache-2.0, at your option.
