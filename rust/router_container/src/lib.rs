#[macro_use]
extern crate lazy_static;
use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use aws_lambda_events::http::{HeaderMap, Result};
use lambda_runtime::LambdaEvent;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

// Define a type for your route handlers. For simplicity, we use a function pointer that takes no arguments and returns nothing.
type Handler = fn(LambdaEvent<ApiGatewayV2httpRequest>) -> Result<Value>;

// This is the global route registry.
lazy_static! {
    pub static ref ROUTES: Mutex<HashMap<(String, String), Handler>> = Mutex::new(HashMap::new());
}

// This function is used by the generated code to register routes.
pub fn register_route(path: &str, method: &str, handler: Handler) {
    let mut routes = ROUTES.lock().unwrap();
    routes.insert(
        (path.to_string(), method.to_uppercase().to_string()),
        handler,
    );
}

// Handle the request
pub fn handle_request(
    event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse> {
    let routes = ROUTES.lock().unwrap();
    let path = event
        .payload
        .request_context
        .http
        .path
        .clone()
        .unwrap_or("-".to_string());
    let method = event.payload.request_context.http.method.to_string();

    println!(
        "Event received: {}",
        serde_json::to_string(&event.payload).unwrap()
    );

    if let Some(handler) = routes.get(&(path, method)) {
        let value = handler(event)?;
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/json".parse().unwrap());
        return Ok(ApiGatewayV2httpResponse {
            status_code: 200,
            body: Some(value.to_string().into()),
            multi_value_headers: headers.clone(),
            headers,
            ..Default::default()
        });
    }

    Ok(ApiGatewayV2httpResponse {
        status_code: 404,
        body: Some(format!("Handler not found for path: {:?}", event).into()),
        ..Default::default()
    })
}
