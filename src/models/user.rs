use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ConfirmUser {
    pub email: String,
    pub code: String,
}

#[derive(Serialize)]
pub struct User {
    pub email: String,
}

#[derive(Serialize)]
pub struct ConfirmUserResponse {
    pub message: String,
    pub user: String,
}
