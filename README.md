# Rust AWS Lambda Template

[![Publish crates](https://github.com/anuradhawick/rs_template/actions/workflows/publish-crates.yml/badge.svg)](https://github.com/anuradhawick/rs_template/actions/workflows/publish-crates.yml)
[![Crates.io](https://img.shields.io/crates/v/lambdamux.svg)](https://crates.io/crates/lambdamux)
[![Docs.rs](https://docs.rs/lambdamux/badge.svg)](https://docs.rs/lambdamux)
[![License](https://img.shields.io/crates/l/lambdamux.svg)](https://crates.io/crates/lambdamux)

This repository contains a Rust-based AWS Lambda function, managed using the cargo-lambda tool and deployed via Terraform.

The workspace now includes a publishable router family built around `lambdamux`:

- `lambdamux` is the user-facing facade crate.
- `lambdamux-core` contains the generic trie-based routing primitives.
- `lambdamux-macro` provides the `#[route(...)]` and `generate_routes!()` macros.

If you publish these crates to crates.io, publish them in this order: `lambdamux-core`, `lambdamux-macro`, then `lambdamux`.

## Prerequisites

Before you begin, ensure you have the following installed:

- Rust: Install Rust by following the instructions at [rust-lang.org](https://www.rust-lang.org/tools/install).
- cargo-lambda: A tool to build and deploy AWS Lambda functions written in Rust from [cargo-lambda.info](https://www.cargo-lambda.info/guide/getting-started.html).
- Terraform: Used for provisioning the AWS infrastructure from [hashicorp.com](https://developer.hashicorp.com/terraform/install).

## Building and Deploying the Lambda Function

### 1. Build and Testing the Lambda Function

First, you need to build the Lambda function for the x86_64-unknown-linux-gnu target, which is required for AWS Lambda:

```bash
cargo lambda build --release --target x86_64-unknown-linux-gnu
```

This will produce a binary in the ./target/lambda/<function-name> directory. This is automatically handled by terraform. To test the suite run the following command.

```bash
cargo test
```

If you only want to test one function/module run the following command.

```bash
# to test the whole test_lambda package
cargo test --package test_lambda --bin test_lambda -- hello::hello_tests --show-output
# to test the hello::hello_tests::hello_get_test test
cargo test --package test_lambda --bin test_lambda -- hello::hello_tests::hello_get_test --exact --show-output
```

### 2. Deploy Using Terraform

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

## Adding More Functions and Endpoints

To add a rust module (a new lambda function) use the following command.

```bash
cd rust
cargo new test_lambda_2
```

Update the cargo.toml with following dependencies.

```toml
[dependencies]
aws_lambda_events = "0.15.1"
lambda_runtime = "0.13.0"
serde = { version = "1.0.207", features = ["derive"] }
serde_json = "1.0.124"
tokio = "1.39.2"
lambdamux = { path = "../lambdamux" }
```

Update the `main.rs` of the newly created package as follows. Note that handler function is placed in `main.rs` to provide the maximum configurability. If you wish to move it to a different file you can do it. In that case compiler will likely need you to move the `mod hello; use hello::*` accordingly as well.

```rust
use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use aws_lambda_events::encodings::Error;
use lambda_runtime::{service_fn, LambdaEvent};
// import modules and module functions here
mod hello;
use hello::*;

async fn handler(
    event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse, Error> {
    lambdamux::handle_apigw_v2!(event)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // we initate the event loop here
    lambda_runtime::run(service_fn(handler)).await
}
```

I have added more comments inside the rust files.

## Authentication

You can authenticate inside the `main.rs` or `hello.rs`. Event object has the complete request context including the auth contexts.

## Development

Follow the style in `test_lambda` folder to add test, routes, etc. You can always add more libraries as needed.

Now you must add new terraform codes to wire up the lambda to an endpoint. Look at the examples provided in [https://github.com/terraform-aws-modules/terraform-aws-apigateway-v2/tree/master/examples](https://github.com/terraform-aws-modules/terraform-aws-apigateway-v2/tree/master/examples)

If you are using the new lambda to just receive event, use `serde_json::Value` or `aws_lambda_events` event type to capture (sns, dynamodb, etc). You can also capture events to structs using `serde_json` crate.
