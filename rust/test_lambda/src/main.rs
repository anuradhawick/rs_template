use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use aws_lambda_events::encodings::Error;
use aws_lambda_events::http::HeaderMap;
use lambda_runtime::{service_fn, LambdaEvent};
use once_cell::sync::Lazy;
use router_container::Trie;
use router_macro::generate_routes;

mod hello;
use hello::*;

// TRIE is a special data structure for faster routing
// this is the only allocation that happens related to routing
static TRIE: Lazy<Trie<ApiGatewayV2httpRequest>> = Lazy::new(|| generate_routes!());

// this function needs to be async
// if you like everything to be async, this can be achieved by slight modifications to
// Handler type in route_container crate
async fn handler(
    mut event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse, Error> {
    // construct router trie
    // extract method and path
    let method = event.payload.request_context.http.method.as_ref();
    let path = event
        .payload
        .request_context
        .http
        .path
        .as_deref()
        .unwrap_or("");
    // get handler and inject path params
    let Some((handler, params)) = TRIE.route(method, path) else {
        return Ok(ApiGatewayV2httpResponse {
            status_code: 404,
            body: Some("Route not found".into()),
            ..Default::default()
        });
    };
    event.payload.path_parameters.extend(params.into_iter());

    // try to call handle the routes received here
    let Ok(value) = handler(event) else {
        // if failed, report as 500 Server Error
        return Ok(ApiGatewayV2httpResponse {
            status_code: 500,
            body: Some("Internal server error".into()),
            ..Default::default()
        });
    };
    let mut headers = HeaderMap::new();
    headers.insert("content-type", "application/json".parse().unwrap());

    Ok(ApiGatewayV2httpResponse {
        status_code: 200,
        body: Some(value.to_string().into()),
        multi_value_headers: headers.clone(),
        headers,
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

    #[tokio::test]
    async fn routes_hello_id_post_test() {
        // create the payload for testing
        let payload = ApiGatewayV2httpRequest {
            // should have the body (POST request)
            request_context: ApiGatewayV2httpRequestContext {
                http: ApiGatewayV2httpRequestContextHttpDescription {
                    path: Some("/hello/0106".into()),
                    method: Method::GET,
                    ..Default::default()
                },
                ..Default::default()
            },
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
        assert_eq!(json!({ "id": "0106", "success": true }).to_string(), text);
    }
}
