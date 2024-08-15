use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use aws_lambda_events::encodings::Error;
use lambda_runtime::{service_fn, LambdaEvent};
use router_container::handle_request;
mod hello;

async fn handler(
    event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse, Error> {
    // try to call handle the routes received here
    let Ok(res) = handle_request(event) else {
        // if failed, report as 500 Server Error
        return Ok(ApiGatewayV2httpResponse {
            status_code: 500,
            body: Some("Internal server error".into()),
            ..Default::default()
        });
    };
    Ok(res)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // we initate the event loop here
    lambda_runtime::run(service_fn(handler)).await
}
