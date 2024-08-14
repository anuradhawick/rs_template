module "app_api" {
  source = "terraform-aws-modules/apigateway-v2/aws"

  name          = "${var.app-name}-http-api"
  description   = "${var.app-name} HTTP API Gateway"
  protocol_type = "HTTP"

  create_domain_name = false
  create_certificate = false

  cors_configuration = {
    allow_headers  = ["*"]
    allow_methods  = ["*"]
    allow_origins  = ["*"]
    expose_headers = ["*"]
    max_age        = 3600
  }

  # Routes & Integration(s)
  routes = {
    "ANY /{proxy+}" = {
      integration = {
        uri                    = module.lambda_function.lambda_function_invoke_arn
        payload_format_version = "2.0"
      }
    }
  }

  tags = var.tags
}

resource "aws_lambda_permission" "capbuid_rest_api_lambda_permissions" {
  statement_id  = "${var.app-name}_rest_api_lambda_permissions"
  action        = "lambda:InvokeFunction"
  function_name = module.lambda_function.lambda_function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${module.app_api.api_execution_arn}/**"
}
