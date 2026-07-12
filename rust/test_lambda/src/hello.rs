use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use aws_lambda_events::encodings::Body;
use aws_lambda_events::http::{HeaderMap, Result};
use lambda_runtime::LambdaEvent;
use lambdamux::route;
use serde_json::{json, Value};

fn json_response(status_code: i64, value: Value) -> ApiGatewayV2httpResponse {
    let mut headers = HeaderMap::new();
    headers.insert("content-type", "application/json".parse().unwrap());

    let mut response = ApiGatewayV2httpResponse::default();
    response.status_code = status_code;
    response.body = Some(Body::Text(value.to_string()));
    response.headers = headers.clone();
    response.multi_value_headers = headers;
    response
}

// adding a GET request handler to path /hello
#[route(path = "/hello", method = "get")]
pub fn hello_get(_event: LambdaEvent<ApiGatewayV2httpRequest>) -> Result<ApiGatewayV2httpResponse> {
    Ok(json_response(200, json!({ "success": true })))
}

// adding a GET request handler to path /hello/:id to demo path params
#[route(path = "/hello/:id", method = "get")]
pub fn hello_id_get(
    event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse> {
    let id = event
        .payload
        .path_parameters
        .get("id")
        .unwrap_or(&String::from(""))
        .clone();
    Ok(json_response(
        200,
        json!({
            "success": true,
            "id": id
        }),
    ))
}

// adding a POST request handler to path /hello
#[route(path = "/hello", method = "post")]
pub fn hello_post(event: LambdaEvent<ApiGatewayV2httpRequest>) -> Result<ApiGatewayV2httpResponse> {
    // parsing the event body getting the serde_json::Value object
    // using unwrap_or(default) is recommended for smaller objects (like empty json)
    // if this becomes massive, use unwrap_or_else(|| "{}",into())
    let body: String = event.payload.body.unwrap_or("{}".into());
    let body: Value = serde_json::from_str(&body).unwrap_or(json!({}));

    // try to get the "name" otherwise return body with success false
    let Some(name) = body.get("name").and_then(|v| v.as_str()) else {
        return Ok(json_response(
            400,
            json!({
                "success": true,
                "name": "not found"
            }),
        ));
    };

    // if the "name" is there, construct response with success true
    Ok(json_response(
        201,
        json!({
            "success": true,
            "name": name
        }),
    ))
}

#[cfg(test)]
mod hello_tests {
    use std::collections::HashMap;

    use super::*;
    use lambda_runtime::Context;

    fn response_body(response: ApiGatewayV2httpResponse) -> Value {
        let Body::Text(text) = response.body.unwrap() else {
            panic!("expected text body")
        };

        serde_json::from_str(&text).unwrap()
    }

    #[test]
    fn hello_get_test() {
        // create a mock request and call the hello_get function
        let res = hello_get(LambdaEvent {
            // use defaults
            payload: ApiGatewayV2httpRequest::default(),
            context: Context::default(),
        });
        // assert that result is is not an error
        assert!(res.is_ok());
        let response = res.unwrap();
        assert_eq!(200, response.status_code);
        assert_eq!(json!({ "success": true }), response_body(response));
    }

    #[test]
    fn hello_id_get_test() {
        // create a mock request and call the hello_get function
        let mut payload = ApiGatewayV2httpRequest::default();
        payload.path_parameters = HashMap::from([("id".into(), "my_id".into())]);

        let res = hello_id_get(LambdaEvent {
            // use defaults
            payload,
            context: Context::default(),
        });
        // assert that result is is not an error
        assert!(res.is_ok());
        let response = res.unwrap();
        assert_eq!(200, response.status_code);
        assert_eq!(
            json!({ "success": true, "id": "my_id" }),
            response_body(response)
        );
    }

    #[test]
    fn hello_post_test() {
        // create the payload for testing
        let mut payload = ApiGatewayV2httpRequest::default();
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
        let res = hello_post(event);
        // assert that is is not an error
        assert!(res.is_ok());
        let response = res.unwrap();
        assert_eq!(201, response.status_code);
        assert_eq!(
            json!({ "name": "Anuradha", "success": true }),
            response_body(response)
        );
    }
}
