mod domains;
mod handlers;
mod models;
mod routes;
mod services;

use axum::Router;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let app = Router::new().merge(routes::users::routes());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind");

    axum::serve(listener, app).await.unwrap();
}
