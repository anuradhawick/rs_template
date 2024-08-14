use aws_lambda_events::apigw::ApiGatewayV2httpRequest;
use aws_lambda_events::http::Result;
use lambda_runtime::LambdaEvent;
use router_macro::route;
use serde_json::{json, Value};

#[route(path = "/hello", method = "get")]
fn hello_get(_event: LambdaEvent<ApiGatewayV2httpRequest>) -> Result<Value> {
    Ok(json!({
        "success": true
    }))
}

#[route(path = "/hello", method = "post")]
fn hello_post(event: LambdaEvent<ApiGatewayV2httpRequest>) -> Result<Value> {
    let body: String = event.payload.body.unwrap_or("{}".into());
    let body: Value = serde_json::from_str(&body).unwrap_or(json!({}));

    let Some(name) = body.get("name").and_then(|v| v.as_str()) else {
        return Ok(json!({
            "success": true,
            "name": "not found"
        }));
    };

    Ok(json!({
        "success": true,
        "name": name
    }))
}
