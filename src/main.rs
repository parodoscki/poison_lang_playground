use axum::{
    http::{HeaderValue, Method},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use lib_helper::compile_pipeline_and_get_string;

// The structure of the data coming from your website
#[derive(Deserialize)]
struct CompileRequest {
    code: String,
}

#[derive(Serialize)]
struct CompileResponse {
    llvm_ir: String,
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::POST]);

    let app = Router::new()
        .route("/compile", post(handle_compile))
        .layer(cors);

    let port = std::env::var("PORT").unwrap_or_else(|_| "10000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    
    println!("🚀 PSN Compiler Backend online at http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_compile(Json(payload): Json<CompileRequest>) -> Json<CompileResponse> {
    let output_ir = match compile_pipeline_and_get_string(&payload.code) {
        Ok(llvm_ir) => llvm_ir,
        Err(err_msg) => format!("; PSN Compilation Error:\n; {}", err_msg),
    };

    Json(CompileResponse { llvm_ir: output_ir })
}
