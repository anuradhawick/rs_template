use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use aws_lambda_events::encodings::Error;
use lambda_runtime::{service_fn, LambdaEvent};
use router_container::Trie;
use router_macro::generate_routes;

mod hello;
use hello::*;

async fn handler(
    event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse, Error> {
    let trie: Trie<ApiGatewayV2httpRequest> = generate_routes!();
    let method = event.payload.request_context.http.method.to_string();
    let path = event.payload.request_context.http.path.unwrap_or("".into());
    let Some((handler, params)) = trie.route(&method, &path) else {
        return Ok(ApiGatewayV2httpResponse {
            status_code: 404,
            body: Some("Route not found".into()),
            ..Default::default()
        });
    };

    println!("{:?}", params);
    // try to call handle the routes received here
    // let Ok(res) = handle_request(event) else {
    //     // if failed, report as 500 Server Error
    // return Ok(ApiGatewayV2httpResponse {
    //     status_code: 500,
    //     body: Some("Internal server error".into()),
    //     ..Default::default()
    // });
    // };
    // Ok(res)

    Ok(ApiGatewayV2httpResponse {
        status_code: 500,
        body: Some("Internal server error".into()),
        ..Default::default()
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // we initate the event loop here
    lambda_runtime::run(service_fn(handler)).await
}

#[cfg(test)]
mod main_tests {
    use aws_lambda_events::{
        apigw::{
            ApiGatewayV2httpRequest, ApiGatewayV2httpRequestContext,
            ApiGatewayV2httpRequestContextHttpDescription,
        },
        encodings::Body,
        http::{method, Method},
    };
    use lambda_runtime::{Context, LambdaEvent};
    use serde_json::json;

    use crate::handler;

    #[tokio::test]
    async fn routes_hello_post_test() {
        // create the payload for testing
        let payload = ApiGatewayV2httpRequest {
            // should have the body (POST request)
            request_context: ApiGatewayV2httpRequestContext {
                http: ApiGatewayV2httpRequestContextHttpDescription {
                    path: Some("/hello".into()),
                    method: Method::POST,
                    ..Default::default()
                },
                ..Default::default()
            },
            body: Some(
                json!({
                    "name": "Anuradha"
                })
                .to_string(),
            ),
            // rest do not care, use defaults
            ..Default::default()
        };
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

        // extract the body as string
        let Body::Text(text) = res.unwrap().body.unwrap() else {
            panic!("Wrong body type returned")
        };
        // unwrap and validate the body; using unwrap in tests is totally fine
        assert_eq!(
            json!({ "name": "Anuradha", "success": true }).to_string(),
            text
        );
    }
}
