use crate::domains::auth::dynamo::CognitoUserManager;
use axum::http::StatusCode;

pub async fn init_cognito_user_manager() -> Result<CognitoUserManager, (StatusCode, String)> {
    CognitoUserManager::new()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
