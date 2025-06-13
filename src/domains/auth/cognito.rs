use aws_config::meta::region::RegionProviderChain;
use aws_sdk_cognitoidentityprovider::{
    config::Region,
    error::SdkError,
    types::{AttributeType, AuthFlowType},
    Client,
    operation::{sign_up::SignUpError, confirm_sign_up::ConfirmSignUpError, initiate_auth::InitiateAuthError},
};
use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::{collections::HashMap, env};

type HmacSha256 = Hmac<Sha256>;

pub struct CognitoService {
    client: Client,
    client_id: String,
    client_secret: String,
    user_pool_id: String,
}

impl CognitoService {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
        let config = aws_config::from_env().region(region_provider).load().await;
        let client = Client::new(&config);

        let client_id = env::var("AWS_COGNITO_CLIENT_ID")?;
        let client_secret = env::var("AWS_COGNITO_CLIENT_SECRET")?;
        let user_pool_id = env::var("AWS_COGNITO_USER_POOL_ID")?;

        Ok(Self {
            client,
            client_id,
            client_secret,
            user_pool_id,
        })
    }

    fn calculate_secret_hash(&self, username: &str) -> String {
        let key = self.client_secret.as_bytes();
        let message = format!("{}{}", username, self.client_id);
        let mut mac = HmacSha256::new_from_slice(key).unwrap();
        mac.update(message.as_bytes());
        let result = mac.finalize().into_bytes();
        general_purpose::STANDARD.encode(result)
    }

    pub async fn register_user(
        &self,
        email: &str,
        password: &str,
        user_id: &str,
    ) -> Result<(), SdkError<SignUpError>> {
        let secret_hash = self.calculate_secret_hash(email);

        print!("Registering user: {}, email: {}\n", user_id, email);

        let res = self.client
    .sign_up()
    .client_id(&self.client_id)
    .username(email)
    .password(password)
    .secret_hash(secret_hash)
    .user_attributes(
        AttributeType::builder().name("email").value(email).build()?    
    )
    .send()
    .await;

match res {
    Ok(_) => Ok(()),
    Err(e) => {
        eprintln!("Error signing up: {:?}", e);
        Err(e)
    }
}
    }

    pub async fn authenticate_user_client_flow(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<String>, SdkError<InitiateAuthError>> {
        let secret_hash = self.calculate_secret_hash(username);

        let mut auth_params = HashMap::new();
        auth_params.insert("USERNAME".into(), username.into());
        auth_params.insert("PASSWORD".into(), password.into());
        auth_params.insert("SECRET_HASH".into(), secret_hash);

        let response = self
            .client
            .initiate_auth()
            .auth_flow(AuthFlowType::UserPasswordAuth)
            .client_id(&self.client_id)
            .set_auth_parameters(Some(auth_params))
            .send()
            .await?;

        Ok(response.authentication_result().and_then(|r| r.access_token().map(|t| t.to_string())))
    }

    pub async fn confirm_user(
        &self,
        username: &str,
        code: &str,
    ) -> Result<(), SdkError<ConfirmSignUpError>> {
        let secret_hash = self.calculate_secret_hash(username);

        self.client
            .confirm_sign_up()
            .client_id(&self.client_id)
            .username(username)
            .confirmation_code(code)
            .secret_hash(secret_hash)
            .send()
            .await?;

        Ok(())
    }

    pub async fn list_users(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let response = self
            .client
            .list_users()
            .user_pool_id(&self.user_pool_id)
            .limit(60)
            .send()
            .await?;

        Ok(response
            .users()
            .iter()
            .filter_map(|u| u.username().map(|s| s.to_string()))
            .collect())
    }
}
