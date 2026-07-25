# lambdamux Architecture Summary

`lambdamux` is split into three crates so the small generic routing engine, the procedural macro layer, and the AWS Lambda-facing API can evolve separately.

## Crate Roles

### `lambdamux-core`

`lambdamux-core` contains the generic router. It has no AWS Lambda dependency and does not know about API Gateway request or response types.

Its main pieces are:

- `HandlerFuture<Output, Error>`: a boxed, sendable future returned by an internal route handler.
- `Handler<Input, Output, Error>`: a function pointer that accepts the route input and returns a `HandlerFuture`.
- `RouteMatch<Input, Output, Error>`: the matched handler plus extracted path parameters.
- `Trie<Input, Output, Error>`: the route table.
- `TrieNode<Input, Output, Error>`: one segment in the route tree.

The core crate is generic over `Input`, `Output`, and `Error`. Its handler representation is asynchronous internally, while the macro layer adapts both synchronous and asynchronous user functions into that representation.

### `lambdamux-macro`

`lambdamux-macro` provides the procedural macros:

- `#[route(path = "...", method = "...")]`
- `generate_routes!()`

The `#[route]` attribute parses the route metadata and records the function name, HTTP method, path, and whether the function is asynchronous in a process-local list while the crate is being compiled. The function itself is emitted unchanged.

`generate_routes!()` reads the recorded route entries and expands to code that creates a `Trie`, then inserts each route into it. Async functions are boxed directly; synchronous functions are wrapped in an async block first. This means both forms can coexist in one route table.

For example, annotated handlers eventually become generated code shaped like this:

```rust
{
    use ::lambdamux::Trie;
    let mut trie = Trie::new();

    trie.insert("get", "/hello", |event| Box::pin(hello_get(event)));
    trie.insert("get", "/hello/:id", |event| {
        Box::pin(async move { hello_id_get(event) })
    });
    trie.insert("post", "/hello", |event| Box::pin(hello_post(event)));

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
- native API Gateway response return types from route handlers

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
        └── handlers
            ├── "GET"  = hello_id_get
            └── "POST" = hello_id_post
```

Static path segments are stored by their literal text, such as `hello`. Dynamic path segments such as `:id` are stored under the special key `:`, and the real parameter name, `id`, is stored on the node.

Each terminal node stores a map from normalized uppercase HTTP methods to handlers. This allows the same path to have independent handlers for methods such as `GET` and `POST`. Inserting the same path and method again replaces only that method's handler.

When a request comes in, `Trie::route(method, path)` walks the path segment by segment:

1. It first tries to match the exact static segment.
2. If there is no exact match, it tries the dynamic `:` child.
3. If a dynamic segment matches, it records the captured value in a parameter map.
4. At the final segment, it checks that the node is the end of a registered route.
5. It normalizes the requested HTTP method to uppercase and looks up its handler in the node's handler map.
6. If everything matches, it returns the handler and the captured path parameters.

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
6. Calls and awaits the matched handler.
7. Returns the handler's API Gateway response object unchanged.
8. Returns `404` if no route matches, or `500` if the handler returns an error.

The API Gateway v1 flow is the same idea, but reads method/path from the v1 request shape and returns the v1 response type.

## Why Three Crates?

The split keeps each concern small:

- `lambdamux-core` can be tested as a plain Rust data structure.
- `lambdamux-macro` only handles compile-time route collection and code generation.
- `lambdamux` gives users the ergonomic Lambda API and hides the lower-level pieces.

This lets most users depend only on `lambdamux`, while the internal routing and macro crates stay independently publishable and reusable.
