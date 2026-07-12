# Rust AWS Lambda Template

[![Publish crates](https://github.com/anuradhawick/rs_template/actions/workflows/publish-crates.yml/badge.svg)](https://github.com/anuradhawick/rs_template/actions/workflows/publish-crates.yml)
[![Crates.io](https://img.shields.io/crates/v/lambdamux.svg)](https://crates.io/crates/lambdamux)
[![Docs.rs](https://docs.rs/lambdamux/badge.svg)](https://docs.rs/lambdamux)
[![License](https://img.shields.io/crates/l/lambdamux.svg)](https://crates.io/crates/lambdamux)

This repository contains a Rust-based AWS Lambda HTTP API template, managed with `cargo-lambda` and deployed with Terraform.

The workspace includes a publishable router family built around `lambdamux`:

- `lambdamux` is the user-facing facade crate.
- `lambdamux-core` contains the generic trie-based routing primitives.
- `lambdamux-macro` provides the `#[route(...)]` and `generate_routes!()` macros.

`lambdamux` lets you write route handlers with attributes and dispatch API Gateway requests without managing a global route cache yourself.

For a short explanation of how the three crates work together, including the trie router, see [AISUMMARY.md](AISUMMARY.md).

## What it gives you

- route registration with `#[route(path = ..., method = ...)]`
- compile-time route table generation with `generate_routes!()`
- built-in API Gateway v2 dispatch with `handle_apigw_v2!`
- built-in API Gateway v1 dispatch with `handle_apigw_v1!`
- path parameter matching such as `/hello/:id`

## Prerequisites

Before you begin, ensure you have the following installed:

- Rust: Install Rust by following the instructions at [rust-lang.org](https://www.rust-lang.org/tools/install).
- cargo-lambda: A tool to build and deploy AWS Lambda functions written in Rust from [cargo-lambda.info](https://www.cargo-lambda.info/guide/getting-started.html).
- Terraform: Used for provisioning the AWS infrastructure from [hashicorp.com](https://developer.hashicorp.com/terraform/install).

## Installation

Add these dependencies to your Lambda crate:

```toml
[dependencies]
aws_lambda_events = "1.2.0"
lambda_runtime = "1.3.0"
serde_json = "1.0"
tokio = "1.52.3"
lambdamux = "1.0.0"
```

Inside this repository, `rust/test_lambda` uses the local workspace crate:

```toml
lambdamux = { version = "1.0.0", path = "../lambdamux" }
```

## Quick Start

Create a route module such as `hello.rs`:

```rust
use aws_lambda_events::apigw::ApiGatewayV2httpRequest;
use aws_lambda_events::http::Result;
use lambda_runtime::LambdaEvent;
use lambdamux::route;
use serde_json::{json, Value};

#[route(path = "/hello", method = "get")]
pub fn hello_get(_event: LambdaEvent<ApiGatewayV2httpRequest>) -> Result<Value> {
    Ok(json!({ "success": true }))
}

#[route(path = "/hello/:id", method = "get")]
pub fn hello_id_get(event: LambdaEvent<ApiGatewayV2httpRequest>) -> Result<Value> {
    let id = event
        .payload
        .path_parameters
        .get("id")
        .cloned()
        .unwrap_or_default();

    Ok(json!({
        "success": true,
        "id": id,
    }))
}

#[route(path = "/hello", method = "post")]
pub fn hello_post(event: LambdaEvent<ApiGatewayV2httpRequest>) -> Result<Value> {
    let body = event.payload.body.unwrap_or("{}".into());
    let body: Value = serde_json::from_str(&body).unwrap_or(json!({}));

    let Some(name) = body.get("name").and_then(|value| value.as_str()) else {
        return Ok(json!({
            "success": true,
            "name": "not found",
        }));
    };

    Ok(json!({
        "success": true,
        "name": name,
    }))
}
```

Then wire the Lambda entry point in `main.rs`:

```rust
use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use aws_lambda_events::encodings::Error;
use lambda_runtime::{service_fn, LambdaEvent};

mod hello;
use hello::*;

async fn handler(
    event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse, Error> {
    lambdamux::handle_apigw_v2!(event)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(service_fn(handler)).await
}
```

That is enough to route requests like:

- `GET /hello`
- `GET /hello/123`
- `POST /hello`

## API Gateway v1

If your Lambda is fronted by API Gateway REST API payloads instead of HTTP API v2 payloads, use `ApiGatewayProxyRequest`, `ApiGatewayProxyResponse`, and the `handle_apigw_v1!` macro instead.

## Building and Testing

Run the workspace tests from the Rust workspace:

```bash
cd rust
cargo test --workspace
```

First, you need to build the Lambda function for the x86_64-unknown-linux-gnu target, which is required for AWS Lambda:

```bash
cargo lambda build --release --target x86_64-unknown-linux-gnu
```

This produces a binary in `target/lambda/<function-name>`. The Terraform example handles this build automatically.

If you only want to test one function/module run the following command.

```bash
# to test the whole test_lambda package
cargo test --package test_lambda --bin test_lambda -- hello::hello_tests --show-output
# to test the hello::hello_tests::hello_get_test test
cargo test --package test_lambda --bin test_lambda -- hello::hello_tests::hello_get_test --exact --show-output
```

## Deploy Using Terraform

Navigate to the terraform directory where the infrastructure as code files are stored.

```bash
cd terraform
```

Before deploying the infrastructure, you need to initialize Terraform:

```bash
terraform init
```

This command will download the necessary providers and set up your working directory. To deploy the AWS Lambda function and its associated resources, use the following command:

```bash
terraform apply
```

Terraform will prompt you to confirm the changes. Type yes to proceed with the deployment.

The Terraform example currently:

- builds the Rust Lambda from `rust/test_lambda`
- packages it with `cargo lambda build`
- creates an API Gateway HTTP API
- forwards `ANY /{proxy+}` to the Lambda function

If your crate path or Lambda binary name is different, update the Terraform `source_path` and function settings before applying.

## Adding More Functions and Endpoints

Follow the style in `rust/test_lambda` to add route modules, tests, and handlers.

To add another Lambda crate to the workspace:

```bash
cd rust
cargo new test_lambda_2
```

Then add the dependencies from the installation section, wire a handler with `handle_apigw_v2!`, and add Terraform to route traffic to the new Lambda.

## Authentication

You can authenticate inside `main.rs` or a route module such as `hello.rs`. The event object includes the complete request context, including API Gateway authorizer context.

## Development

Useful checks before publishing or deploying:

```bash
cd rust
cargo test --workspace
cargo deny check
```

When publishing the crates manually, publish them in dependency order: `lambdamux-core`, `lambdamux-macro`, then `lambdamux`. The GitHub release workflow follows the same order.

Add Terraform configuration to wire new Lambda functions to API endpoints. The examples in [terraform-aws-apigateway-v2](https://github.com/terraform-aws-modules/terraform-aws-apigateway-v2/tree/master/examples) are a useful reference.

If you are using the new lambda to just receive event, use `serde_json::Value` or `aws_lambda_events` event type to capture (sns, dynamodb, etc). You can also capture events to structs using `serde_json` crate.

## License

Licensed under either GPL-3.0-only or Apache-2.0, at your option.
