use aws_lambda_events::{apigw::ApiGatewayV2httpResponse, encodings::Body, http::HeaderMap};
use serde_json::Value;

pub(crate) fn json_response(status_code: i64, value: Value) -> ApiGatewayV2httpResponse {
    let mut headers = HeaderMap::new();
    headers.insert("content-type", "application/json".parse().unwrap());

    let mut response = ApiGatewayV2httpResponse::default();
    response.status_code = status_code;
    response.body = Some(Body::Text(value.to_string()));
    response.headers = headers.clone();
    response.multi_value_headers = headers;
    response
}
