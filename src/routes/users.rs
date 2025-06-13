use axum::{routing::post, Router};

use crate::handlers::users::{confirm_user, create_user, list_users};

pub fn routes() -> Router {
    Router::new()
        .route("/users", post(create_user).get(list_users))
        .route("/users/confirm", post(confirm_user))
}
