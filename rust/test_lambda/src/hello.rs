use aws_lambda_events::apigw::ApiGatewayV2httpRequest;
use aws_lambda_events::http::Result;
use lambda_runtime::LambdaEvent;
use router_macro::route;
use serde_json::{json, Value};

// adding a GET request handler to path /hello
#[route(path = "/hello", method = "get")]
fn hello_get(_event: LambdaEvent<ApiGatewayV2httpRequest>) -> Result<Value> {
    Ok(json!({
        "success": true
    }))
}

// adding a POST request handler to path /hello
#[route(path = "/hello", method = "post")]
fn hello_post(event: LambdaEvent<ApiGatewayV2httpRequest>) -> Result<Value> {
    // parsing the event body getting the serde_json::Value object
    // using unwrap_or(default) is recommended for smaller objects (like empty json)
    // if this becomes massive, use unwrap_or_else(|| "{}",into())
    let body: String = event.payload.body.unwrap_or("{}".into());
    let body: Value = serde_json::from_str(&body).unwrap_or(json!({}));

    // try to get the "name" otherwise return body with success false
    let Some(name) = body.get("name").and_then(|v| v.as_str()) else {
        return Ok(json!({
            "success": true,
            "name": "not found"
        }));
    };

    // if the "name" is there, construct response with success true
    Ok(json!({
        "success": true,
        "name": name
    }))
}

#[cfg(test)]
mod hello_tests {
    use super::*;
    use lambda_runtime::Context;

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
        // unwrap and validate the body; using unwrap in tests is totally fine
        assert_eq!(json!({ "success": true }), res.unwrap());
    }

    #[test]
    fn hello_post_test() {
        // create the payload for testing
        let payload = ApiGatewayV2httpRequest {
            // should have the body (POST request)
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
        let res = hello_post(event);
        // assert that is is not an error
        assert!(res.is_ok());
        // unwrap and validate the body; using unwrap in tests is totally fine
        assert_eq!(json!({ "name": "Anuradha", "success": true }), res.unwrap());
    }
}
