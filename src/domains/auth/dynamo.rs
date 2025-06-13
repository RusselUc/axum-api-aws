use aws_config::meta::region::RegionProviderChain;
use aws_sdk_cognitoidentityprovider::operation::confirm_sign_up::ConfirmSignUpError;
use aws_sdk_cognitoidentityprovider::operation::initiate_auth::InitiateAuthError;
use aws_sdk_cognitoidentityprovider::operation::sign_up::SignUpError;
use aws_sdk_cognitoidentityprovider::{
    config::Region,
    error::SdkError,
    types::{AttributeType, AuthFlowType},
    Client as CognitoClient,
};

use crate::models::user::User;

use aws_sdk_dynamodb::Client as DynamoClient;
use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::{collections::HashMap, env};
use uuid::Uuid;

use aws_sdk_cognitoidentityprovider::Client;
// use aws_types::Credentials;

type HmacSha256 = Hmac<Sha256>;

pub struct CognitoUserManager {
    pub client: CognitoClient,
    pub dynamo_client: DynamoClient,
    client_id: String,
    client_secret: String,
    user_pool_id: String,
}

impl CognitoUserManager {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");

        // let config = aws_config::from_env().region(region_provider).load().await;

        // let config = aws_config::from_env()
        //     .region(region_provider)
        //     .endpoint_url("http://localhost:8000") // 👈 Esta línea es clave
        //     .load()
        //     .await;

        let config = aws_config::from_env().region(region_provider).load().await;

        let client = CognitoClient::new(&config);
        let dynamo_client = DynamoClient::new(&config);

        let client_id =
            env::var("AWS_COGNITO_CLIENT_ID").expect("AWS_COGNITO_CLIENT_ID must be set");
        let client_secret =
            env::var("AWS_COGNITO_CLIENT_SECRET").expect("AWS_COGNITO_CLIENT_SECRET must be set");
        let user_pool_id =
            env::var("AWS_COGNITO_USER_POOL_ID").expect("AWS_COGNITO_USER_POOL_ID must be set");

        Ok(Self {
            client,
            dynamo_client,
            client_id,
            client_secret,
            user_pool_id,
        })
    }

    pub async fn list_users_from_dynamo(&self) -> Result<Vec<User>, Box<dyn std::error::Error>> {
        use aws_sdk_dynamodb::types::AttributeValue;

        let result = self.dynamo_client.scan().table_name("users").send().await?;

        let users = result
            .items
            .unwrap_or_default()
            .into_iter()
            .filter_map(|item| {
                let email = item.get("email")?.as_s().ok()?.to_string();

                Some(User { email })
            })
            .collect();

        Ok(users)
    }

    pub async fn list_users(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let response = self
            .client
            .list_users()
            .user_pool_id(&self.user_pool_id)
            .limit(60) // Opcional, máximo 60 por request (máximo que acepta AWS)
            .send()
            .await?;

        // Extraer los usernames (o puedes extraer cualquier otro atributo)
        let users = response
            .users()
            .iter()
            .filter_map(|user| user.username().map(|s| s.to_string()))
            .collect();

        Ok(users)
    }

    pub async fn save_user_to_dynamo(
        &self,
        user_id: &str,
        email: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use aws_sdk_dynamodb::types::AttributeValue;

        self.dynamo_client
            .put_item()
            .table_name("users")
            .item("id", AttributeValue::S(user_id.to_string()))
            .item("email", AttributeValue::S(email.to_string()))
            .send()
            .await?;

        println!("✅ Usuario guardado en DynamoDB con ID: {}", user_id);
        Ok(())
    }

    fn calculate_secret_hash(&self, username: &str) -> String {
        let key = self.client_secret.as_bytes();
        let message = format!("{}{}", username, self.client_id);

        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(message.as_bytes());

        let result = mac.finalize().into_bytes();
        general_purpose::STANDARD.encode(&result)
    }

    pub async fn register_user_client_flow(
        &self,
        email: &str,
        password: &str,
    ) -> Result<String, SdkError<SignUpError>> {
        let secret_hash = self.calculate_secret_hash(email);

        self.client
            .sign_up()
            .client_id(&self.client_id)
            .username(email)
            .password(password)
            .secret_hash(secret_hash)
            .user_attributes(
                AttributeType::builder()
                    .name("email")
                    .value(email)
                    .build()?,
            )
            .send()
            .await?;

        Ok(email.to_string())
    }

    // Cambiado para confirmación por código (flujo cliente)
    pub async fn confirm_user(
        &self,
        username: &str,
        confirmation_code: &str,
    ) -> Result<(), SdkError<ConfirmSignUpError>> {
        let secret_hash = self.calculate_secret_hash(username);

        self.client
            .confirm_sign_up()
            .client_id(&self.client_id)
            .username(username)
            .confirmation_code(confirmation_code)
            .secret_hash(secret_hash)
            .send()
            .await?;

        println!("✅ Usuario confirmado: {}", username);
        Ok(())
    }

    pub async fn authenticate_user_client_flow(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<String>, SdkError<InitiateAuthError>> {
        let secret_hash = self.calculate_secret_hash(username);

        let mut auth_params = HashMap::new();
        auth_params.insert("USERNAME".to_string(), username.to_string());
        auth_params.insert("PASSWORD".to_string(), password.to_string());
        auth_params.insert("SECRET_HASH".to_string(), secret_hash);

        let response = self
            .client
            .initiate_auth()
            .auth_flow(AuthFlowType::UserPasswordAuth)
            .client_id(&self.client_id)
            .set_auth_parameters(Some(auth_params))
            .send()
            .await?;

        if let Some(result) = response.authentication_result() {
            if let Some(token) = result.access_token() {
                return Ok(Some(token.to_string()));
            }
        }

        Ok(None)
    }

    // pub async fn register_user(
    //     &self,
    //     email: &str,
    //     password: &str,
    // ) -> Result<String, Box<dyn std::error::Error>> {
    //     let username = self.register_user_client_flow(email, password).await?;
    //     Ok(username)
    // }

    pub async fn register_user(
        &self,
        email: &str,
        password: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let username = self.register_user_client_flow(email, password).await?;

        // Guardar en DynamoDB
        self.save_user_to_dynamo(&username, email).await?;

        Ok(username)
    }

    pub async fn authenticate_user(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        self.authenticate_user_client_flow(username, password)
            .await
            .map_err(Into::into)
    }

    pub async fn get_user_from_token(
        &self,
        _token: &str,
    ) -> Result<Option<HashMap<String, String>>, Box<dyn std::error::Error>> {
        Ok(Some(HashMap::from([(
            "email".to_string(),
            "demo@example.com".to_string(),
        )])))
    }

    pub async fn ensure_users_table(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = &self.dynamo_client;

        let table_name = "users";

        // Primero intentamos ver si existe la tabla
        let tables = client
            .list_tables()
            .send()
            .await?
            .table_names
            .unwrap_or_default();

        if tables.contains(&table_name.to_string()) {
            println!("✅ Tabla '{}' ya existe", table_name);
            return Ok(());
        }

        println!("🔨 Creando tabla '{}'...", table_name);

        client
            .create_table()
            .table_name(table_name)
            .attribute_definitions(
                aws_sdk_dynamodb::types::AttributeDefinition::builder()
                    .attribute_name("id")
                    .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
                    .build()?,
            )
            .key_schema(
                aws_sdk_dynamodb::types::KeySchemaElement::builder()
                    .attribute_name("id")
                    .key_type(aws_sdk_dynamodb::types::KeyType::Hash)
                    .build()?,
            )
            .provisioned_throughput(
                aws_sdk_dynamodb::types::ProvisionedThroughput::builder()
                    .read_capacity_units(5)
                    .write_capacity_units(5)
                    .build()?,
            )
            .send()
            .await?;

        println!("✅ Tabla '{}' creada", table_name);
        Ok(())
    }
}
