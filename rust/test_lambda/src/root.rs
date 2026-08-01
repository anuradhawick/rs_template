use crate::util::json_response;
use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use aws_lambda_events::http::Result;
use lambda_runtime::LambdaEvent;
use lambdamux::route;
use serde_json::{json, Value};

// adding a GET request handler to path /
#[route(path = "/", method = "get")]
pub async fn root_get(
    _event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse> {
    Ok(json_response(200, json!({ "success": true })))
}

// adding a POST request handler to path /
#[route(path = "/", method = "post")]
pub async fn root_post(
    event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse> {
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
mod root_tests {
    use super::*;
    use aws_lambda_events::encodings::Body;
    use lambda_runtime::Context;

    fn response_body(response: ApiGatewayV2httpResponse) -> Value {
        let Body::Text(text) = response.body.unwrap() else {
            panic!("expected text body")
        };

        serde_json::from_str(&text).unwrap()
    }

    #[tokio::test]
    async fn root_get_test() {
        // create a mock request and call the root_get function
        let res = root_get(LambdaEvent {
            // use defaults
            payload: ApiGatewayV2httpRequest::default(),
            context: Context::default(),
        })
        .await;
        // assert that result is is not an error
        assert!(res.is_ok());
        let response = res.unwrap();
        assert_eq!(200, response.status_code);
        assert_eq!(json!({ "success": true }), response_body(response));
    }

    #[tokio::test]
    async fn root_post_test() {
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
        let res = root_post(event).await;
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
