use super::cognito::CognitoService;
use super::dynamo::DynamoService;
use crate::models::user::User;
use uuid::Uuid;

pub struct UserManager {
    cognito: CognitoService,
    dynamo: DynamoService,
}

impl UserManager {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let cognito = CognitoService::new().await?;
        let dynamo = DynamoService::new().await?;
        Ok(Self { cognito, dynamo })
    }

    pub async fn register_user(
        &self,
        email: &str,
        password: &str,
        user_id: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // let user_id = Uuid::new_v4().to_string(); // 👈 Generamos el UUID

        self.cognito.register_user(email, password, user_id.as_str()).await?;
        self.dynamo.save_user(user_id.as_str(), email).await?;

        Ok(user_id)
    }

    pub async fn authenticate_user(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        self.cognito
            .authenticate_user_client_flow(username, password)
            .await
            .map_err(Into::into)
    }

    pub async fn confirm_email(
    &self,
    email: &str,
    code: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    self.cognito
        .confirm_user(email, code)
        .await
        .map_err(Into::into)
}


    pub async fn list_users_from_dynamo(&self) -> Result<Vec<User>, Box<dyn std::error::Error>> {
        self.dynamo.list_users().await
    }

    pub async fn list_users_from_cognito(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        self.cognito.list_users().await
    }

    pub async fn ensure_users_table(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.dynamo.ensure_users_table().await
    }
}
