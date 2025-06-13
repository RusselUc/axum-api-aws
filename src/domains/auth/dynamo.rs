use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{Client, types::{AttributeValue, AttributeDefinition, ScalarAttributeType, KeySchemaElement, KeyType, ProvisionedThroughput}};
use crate::models::user::User;

pub struct DynamoService {
    client: Client,
}

impl DynamoService {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
        let config = aws_config::from_env().region(region_provider).load().await;
        Ok(Self {
            client: Client::new(&config),
        })
    }

    pub async fn save_user(&self, user_id: &str, email: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.client
            .put_item()
            .table_name("users")
            .item("id", AttributeValue::S(user_id.into()))
            .item("email", AttributeValue::S(email.into()))
            .send()
            .await?;
        Ok(())
    }

    pub async fn list_users(&self) -> Result<Vec<User>, Box<dyn std::error::Error>> {
        let result = self.client.scan().table_name("users").send().await?;

        Ok(result
            .items
            .unwrap_or_default()
            .into_iter()
            .filter_map(|item| {
                let email = item.get("email")?.as_s().ok()?.to_string();
                Some(User { email })
            })
            .collect())
    }

    pub async fn ensure_users_table(&self) -> Result<(), Box<dyn std::error::Error>> {
        let table_name = "users";

        let existing_tables = self
            .client
            .list_tables()
            .send()
            .await?
            .table_names
            .unwrap_or_default();

        if existing_tables.contains(&table_name.to_string()) {
            println!("✅ Tabla '{}' ya existe", table_name);
            return Ok(());
        }

        self.client
            .create_table()
            .table_name(table_name)
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("id")
                    .attribute_type(ScalarAttributeType::S)
                    .build()?,
            )
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("id")
                    .key_type(KeyType::Hash)
                    .build()?,
            )
            .provisioned_throughput(
                ProvisionedThroughput::builder()
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
