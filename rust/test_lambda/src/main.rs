use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use aws_lambda_events::encodings::Error;
use lambda_runtime::{service_fn, LambdaEvent};

mod hello;
mod util;
use hello::*;
mod root;
use root::*;

// this function needs to be async
// if you like everything to be async, this can be achieved by slight modifications to
// Handler type in the lambdamux core crate
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

#[cfg(test)]
mod main_tests {
    use super::*;
    use aws_lambda_events::{
        apigw::{
            ApiGatewayV2httpRequest, ApiGatewayV2httpRequestContext,
            ApiGatewayV2httpRequestContextHttpDescription,
        },
        encodings::Body,
        http::Method,
    };
    use lambda_runtime::{Context, LambdaEvent};
    use serde_json::json;

    #[tokio::test]
    async fn routes_hello_post_test() {
        // create the payload for testing
        let mut http = ApiGatewayV2httpRequestContextHttpDescription::default();
        http.path = Some("/hello".into());
        http.method = Method::POST;

        let mut request_context = ApiGatewayV2httpRequestContext::default();
        request_context.http = http;

        let mut payload = ApiGatewayV2httpRequest::default();
        payload.request_context = request_context;
        payload.body = Some(
            json!({
                "name": "Anuradha"
            })
            .to_string(),
        );
        // compile the event
        let event = LambdaEvent {
            payload,
            // context is not used, so use default
            context: Context::default(),
        };
        // get the result object
        let res = handler(event).await;
        // assert that is is not an error
        assert!(res.is_ok());

        let response = res.unwrap();
        assert_eq!(201, response.status_code);

        // extract the body as string
        let Body::Text(text) = response.body.unwrap() else {
            panic!("Wrong body type returned")
        };
        // unwrap and validate the body; using unwrap in tests is totally fine
        assert_eq!(
            json!({ "name": "Anuradha", "success": true }).to_string(),
            text
        );
    }

    #[tokio::test]
    async fn routes_hello_id_post_test() {
        // create the payload for testing
        let mut http = ApiGatewayV2httpRequestContextHttpDescription::default();
        http.path = Some("/hello/0106".into());
        http.method = Method::GET;

        let mut request_context = ApiGatewayV2httpRequestContext::default();
        request_context.http = http;

        let mut payload = ApiGatewayV2httpRequest::default();
        payload.request_context = request_context;
        // compile the event
        let event = LambdaEvent {
            payload,
            // context is not used, so use default
            context: Context::default(),
        };
        // get the result object
        let res = handler(event).await;
        // assert that is is not an error
        assert!(res.is_ok());

        let response = res.unwrap();
        assert_eq!(200, response.status_code);

        // extract the body as string
        let Body::Text(text) = response.body.unwrap() else {
            panic!("Wrong body type returned")
        };
        // unwrap and validate the body; using unwrap in tests is totally fine
        assert_eq!(json!({ "id": "0106", "success": true }).to_string(), text);
    }

    #[tokio::test]
    async fn routes_root_post_test() {
        // create the payload for testing
        let mut http = ApiGatewayV2httpRequestContextHttpDescription::default();
        http.path = Some("/".into());
        http.method = Method::POST;

        let mut request_context = ApiGatewayV2httpRequestContext::default();
        request_context.http = http;

        let mut payload = ApiGatewayV2httpRequest::default();
        payload.body = Some(
            json!({
                "name": "Anuradha"
            })
            .to_string(),
        );
        payload.request_context = request_context;
        // compile the event
        let event = LambdaEvent {
            payload,
            // context is not used, so use default
            context: Context::default(),
        };
        // get the result object
        let res = handler(event).await;
        // assert that is is not an error
        assert!(res.is_ok());

        let response = res.unwrap();
        assert_eq!(201, response.status_code);

        // extract the body as string
        let Body::Text(text) = response.body.unwrap() else {
            panic!("Wrong body type returned")
        };
        // unwrap and validate the body; using unwrap in tests is totally fine
        assert_eq!(
            json!({ "name": "Anuradha", "success": true }).to_string(),
            text
        );
    }
}
