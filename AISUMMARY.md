# lambdamux Architecture Summary

`lambdamux` is split into three crates so the small generic routing engine, the procedural macro layer, and the AWS Lambda-facing API can evolve separately.

## Crate Roles

### `lambdamux-core`

`lambdamux-core` contains the generic router. It has no AWS Lambda dependency and does not know about API Gateway request or response types.

Its main pieces are:

- `Handler<Input, Output, Error>`: a function pointer type for route handlers.
- `RouteMatch<Input, Output, Error>`: the matched handler plus extracted path parameters.
- `Trie<Input, Output, Error>`: the route table.
- `TrieNode<Input, Output, Error>`: one segment in the route tree.

The core crate is generic over `Input`, `Output`, and `Error`, so the same routing structure can be used with Lambda events today and other handler shapes later.

### `lambdamux-macro`

`lambdamux-macro` provides the procedural macros:

- `#[route(path = "...", method = "...")]`
- `generate_routes!()`

The `#[route]` attribute parses the route metadata and records the function name, HTTP method, and path in a process-local list while the crate is being compiled. The function itself is emitted unchanged.

`generate_routes!()` reads the recorded route entries and expands to code that creates a `Trie`, then inserts each route into it.

For example, annotated handlers eventually become generated code shaped like this:

```rust
{
    use ::lambdamux::Trie;
    let mut trie = Trie::new();

    trie.insert("get", "/hello", hello_get);
    trie.insert("get", "/hello/:id", hello_id_get);
    trie.insert("post", "/hello", hello_post);

    trie
}
```

### `lambdamux`

`lambdamux` is the facade crate users normally depend on. It re-exports:

- `Trie` and `Handler` from `lambdamux-core`
- `route` and `generate_routes` from `lambdamux-macro`

It also provides AWS Lambda/API Gateway integration:

- `handle_apigw_v1!`
- `handle_apigw_v2!`
- API Gateway v1 and v2 dispatch helpers
- response conversion from `serde_json::Value` to API Gateway JSON responses

The facade crate is where generic routing becomes Lambda routing.

## How the Trie Works

A trie is a tree where each level represents one segment of the path.

For this route:

```text
GET /hello/:id
```

the trie stores something conceptually like:

```text
root
└── hello
    └── :
        ├── parameter_name = "id"
        ├── method = "GET"
        └── handler = hello_id_get
```

Static path segments are stored by their literal text, such as `hello`. Dynamic path segments such as `:id` are stored under the special key `:`, and the real parameter name, `id`, is stored on the node.

When a request comes in, `Trie::route(method, path)` walks the path segment by segment:

1. It first tries to match the exact static segment.
2. If there is no exact match, it tries the dynamic `:` child.
3. If a dynamic segment matches, it records the captured value in a parameter map.
4. At the final segment, it checks that the node is the end of a registered route and that the HTTP method matches.
5. If everything matches, it returns the handler and the captured path parameters.

For example:

```text
GET /hello/123
```

matches `/hello/:id` and returns:

```text
handler = hello_id_get
params = { "id": "123" }
```

The lookup is proportional to the number of path segments, not the total number of routes. That makes it a good fit for Lambda functions with many HTTP routes.

## Request Flow

At runtime, the user calls one of the facade macros from their Lambda handler:

```rust
lambdamux::handle_apigw_v2!(event)
```

That expands to a call into the API Gateway v2 dispatcher. The dispatcher:

1. Builds the route trie once using `generate_routes!()`.
2. Caches that trie in a `OnceLock`, so later Lambda invocations in the same warm runtime reuse it.
3. Reads the HTTP method and path from the API Gateway event.
4. Calls `Trie::route(method, path)`.
5. Extends the event's `path_parameters` with any dynamic route captures.
6. Calls the matched handler.
7. Converts the returned `serde_json::Value` into an API Gateway response body.
8. Returns `404` if no route matches, or `500` if the handler returns an error.

The API Gateway v1 flow is the same idea, but reads method/path from the v1 request shape and returns the v1 response type.

## Why Three Crates?

The split keeps each concern small:

- `lambdamux-core` can be tested as a plain Rust data structure.
- `lambdamux-macro` only handles compile-time route collection and code generation.
- `lambdamux` gives users the ergonomic Lambda API and hides the lower-level pieces.

This lets most users depend only on `lambdamux`, while the internal routing and macro crates stay independently publishable and reusable.
