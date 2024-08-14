provider "aws" {
  region = "us-east-1"
}

module "lambda_function" {
  source = "terraform-aws-modules/lambda/aws"

  function_name = "test_lambda"
  description   = "rust lambda function"
  handler       = "rust.handler"
  runtime       = "provided.al2023"

  source_path = [
    {
      path = "../rust/test_lambda"
      commands = [
        "cargo lambda build --release --lambda-dir target",
        "cd target/test_lambda",
        ":zip"
      ]
    }
  ]

  tags = var.tags
}
