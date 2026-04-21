use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub price: f64,
    pub stock: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Customer {
    pub id: i32,
    pub wa_number: String,
    pub name: Option<String>,  
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    pub id: i32,
    pub customer_id: i32,
    pub total: f64,
    pub status: String,
    pub direccion: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderItem {
    pub id: i32,
    pub order_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    pub subtotal: f64,
}