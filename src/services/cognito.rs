use crate::domains::auth::user_manager::UserManager;
use axum::http::StatusCode;

pub async fn init_cognito_user_manager() -> Result<UserManager, (StatusCode, String)> {
    UserManager::new()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
