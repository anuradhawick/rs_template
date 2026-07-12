use aws_lambda_events::encodings::Error;
use aws_lambda_events::http;
use lambda_runtime::LambdaEvent;

pub use lambdamux_core::{Handler, Trie};
pub use lambdamux_macro::{generate_routes, route};

pub type LambdaHandler<Request, Response> = Handler<LambdaEvent<Request>, Response, http::Error>;
pub type LambdaTrie<Request, Response> = Trie<LambdaEvent<Request>, Response, http::Error>;

pub mod lambda {
    use std::sync::OnceLock;

    use aws_lambda_events::{
        apigw::{
            ApiGatewayProxyRequest, ApiGatewayProxyResponse, ApiGatewayV2httpRequest,
            ApiGatewayV2httpResponse,
        },
        encodings::{Body, Error},
    };
    use lambda_runtime::LambdaEvent;

    use crate::LambdaTrie;

    fn server_error_v1() -> ApiGatewayProxyResponse {
        let mut response = ApiGatewayProxyResponse::default();
        response.status_code = 500;
        response.body = Some(Body::Text("Internal server error".into()));
        response
    }

    fn not_found_v1() -> ApiGatewayProxyResponse {
        let mut response = ApiGatewayProxyResponse::default();
        response.status_code = 404;
        response.body = Some(Body::Text("Route not found".into()));
        response
    }

    fn server_error_v2() -> ApiGatewayV2httpResponse {
        let mut response = ApiGatewayV2httpResponse::default();
        response.status_code = 500;
        response.body = Some(Body::Text("Internal server error".into()));
        response
    }

    fn not_found_v2() -> ApiGatewayV2httpResponse {
        let mut response = ApiGatewayV2httpResponse::default();
        response.status_code = 404;
        response.body = Some(Body::Text("Route not found".into()));
        response
    }

    pub mod apigw_v1 {
        use super::*;

        static ROUTES: OnceLock<LambdaTrie<ApiGatewayProxyRequest, ApiGatewayProxyResponse>> =
            OnceLock::new();

        pub async fn handle<F>(
            mut event: LambdaEvent<ApiGatewayProxyRequest>,
            init: F,
        ) -> Result<ApiGatewayProxyResponse, Error>
        where
            F: FnOnce() -> LambdaTrie<ApiGatewayProxyRequest, ApiGatewayProxyResponse>,
        {
            let method = event.payload.http_method.as_ref();
            let path = event.payload.path.as_deref().unwrap_or("");
            let routes = ROUTES.get_or_init(init);

            let Some((handler, params)) = routes.route(method, path) else {
                return Ok(not_found_v1());
            };

            event.payload.path_parameters.extend(params);

            let Ok(response) = handler(event) else {
                return Ok(server_error_v1());
            };

            Ok(response)
        }
    }

    pub mod apigw_v2 {
        use super::*;

        static ROUTES: OnceLock<LambdaTrie<ApiGatewayV2httpRequest, ApiGatewayV2httpResponse>> =
            OnceLock::new();

        pub async fn handle<F>(
            mut event: LambdaEvent<ApiGatewayV2httpRequest>,
            init: F,
        ) -> Result<ApiGatewayV2httpResponse, Error>
        where
            F: FnOnce() -> LambdaTrie<ApiGatewayV2httpRequest, ApiGatewayV2httpResponse>,
        {
            let method = event.payload.request_context.http.method.as_ref();
            let path = event
                .payload
                .request_context
                .http
                .path
                .as_deref()
                .unwrap_or("");
            let routes = ROUTES.get_or_init(init);

            let Some((handler, params)) = routes.route(method, path) else {
                return Ok(not_found_v2());
            };

            event.payload.path_parameters.extend(params);

            let Ok(response) = handler(event) else {
                return Ok(server_error_v2());
            };

            Ok(response)
        }
    }
}

#[macro_export]
macro_rules! handle_apigw_v1 {
    ($event:expr) => {{
        $crate::lambda::apigw_v1::handle($event, || $crate::generate_routes!()).await
    }};
}

#[macro_export]
macro_rules! handle_apigw_v2 {
    ($event:expr) => {{
        $crate::lambda::apigw_v2::handle($event, || $crate::generate_routes!()).await
    }};
}

pub type LambdaError = Error;
