# Rust AWS Lambda Template

This repository contains a Rust-based AWS Lambda function, managed using the cargo-lambda tool and deployed via Terraform.

## Prerequisites

Before you begin, ensure you have the following installed:

* Rust: Install Rust by following the instructions at rust-lang.org.
* cargo-lambda: A tool to build and deploy AWS Lambda functions written in Rust.
* Terraform: Used for provisioning the AWS infrastructure.

## Install Rust

If Rust is not installed, use the following command:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
## Install cargo-lambda

To install the cargo-lambda tool, run:

```bash
cargo install cargo-lambda
# or in OSX
brew install cargo-lambda
```

## Install Terraform
You can install Terraform by following the instructions on the Terraform website.
Alternatively, if you are using a package manager:

### Homebrew (macOS/Linux):

```bash
brew install terraform
```

### Chocolatey (Windows):

```bash
choco install terraform
```

## Building and Deploying the Lambda Function

### 1. Build the Lambda Function
First, you need to build the Lambda function for the x86_64-unknown-linux-gnu target, which is required for AWS Lambda:
```bash
cargo lambda build --release --target x86_64-unknown-linux-gnu
```
This will produce a binary in the ./target/lambda/<function-name> directory.

### 2. Setting Up Terraform

Navigate to the terraform directory where the infrastructure as code files are stored.

```bash
cd terraform
```

### 3. Initialize Terraform

Before deploying the infrastructure, you need to initialize Terraform:

```bash
terraform init
```

This command will download the necessary providers and set up your working directory.

### 4. Apply the Terraform Configuration

To deploy the AWS Lambda function and its associated resources, use the following command:

```bash
terraform apply
```
Terraform will prompt you to confirm the changes. Type yes to proceed with the deployment.

### 5. Verify the Deployment

After Terraform has successfully applied the configuration, it will output the relevant information, such as the API Gateway URL, Lambda ARN, etc.
Additional Commands

## Clean the Build Artifacts

To clean up the build artifacts, use:

```bash
cargo clean
```

## Redeploy the Lambda Function

After making changes to the code, you can rebuild and redeploy the Lambda function by repeating the build and apply steps:

```bash
cargo lambda build --release --target x86_64-unknown-linux-gnu
cd terraform
terraform apply
```
