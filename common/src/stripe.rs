use stripe::{Client, CreateCustomer, Customer};

use crate::error::{AppError, Res};

pub fn create_client(secret_key: &str) -> Client {
    Client::new(secret_key)
}

pub async fn create_customer(client: &Client, email: &str, name: &str) -> Res<Customer> {
    let params = CreateCustomer {
        email: Some(email),
        name: Some(name),
        ..Default::default()
    };

    Customer::create(client, params)
        .await
        .map_err(AppError::from)
}