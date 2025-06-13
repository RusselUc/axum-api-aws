use crate::models::user::{ConfirmUser, ConfirmUserResponse, CreateUser, User};
use crate::services::cognito::init_cognito_user_manager;
use axum::{extract::Json, extract::Query, http::StatusCode};
use std::collections::HashMap;

fn map_internal_error<E: std::fmt::Display>(err: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

pub async fn create_user(
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, (StatusCode, String)> {
    let manager = init_cognito_user_manager().await?;

    manager
        .register_user_client_flow(&payload.email, &payload.password)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(User {
        email: payload.email,
    }))
}

pub async fn confirm_user(
    Json(payload): Json<ConfirmUser>,
) -> Result<Json<ConfirmUserResponse>, (StatusCode, String)> {
    let manager = init_cognito_user_manager().await?;

    manager
        .confirm_user(&payload.email, &payload.code)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ConfirmUserResponse {
        message: "User confirmed successfully".to_string(),
        user: payload.email,
    }))
}

pub async fn list_users(
    Query(_params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let manager = init_cognito_user_manager().await?;
    let users = manager
        .list_users_from_dynamo()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(users))
}
