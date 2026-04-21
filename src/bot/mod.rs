use tokio::sync::broadcast;
use r2d2_sqlite::rusqlite::Connection; 
use crate::db;

// Estados posibles de un cliente en la charla
pub enum ChatState {
    Greeting,
    SelectingProducts,
    ConfirmingOrder,
}

pub fn responder_mensaje(
    _whatsapp_id: &str,
    mensaje_entrante: &str,
    conn: &mut Connection,
    notifier: &broadcast::Sender<String>,
) -> String {
    let mensaje = mensaje_entrante.trim().to_lowercase();

    // 1. Lógica de saludo y registro rápido
    if mensaje == "hola" || mensaje == "menu" {
        return format!(
            "¡Hola! Bienvenido.\n\n\
            1. Pizza Especial - $15.50\n\
            2. Hamburguesa Max - $12.00\n\
            3. Café Negro - $3.50\n\n\
            Responde con el NÚMERO del producto para añadirlo."
        );
    }

    // 2. Lógica de selección de productos 
    match mensaje.as_str() {
       "1" | "2" | "3" => {
           let prod_id: i32 = mensaje.parse().unwrap();
           let carrito = vec![(prod_id, 1)]; 
           
           // Pasamos los 7 argumentos: conn, id, carrito, notifier, nombre, tel, dir
           match db::procesar_pedido_completo(
               conn, 
               1, 
               carrito, 
               notifier, 
               "Cliente de WhatsApp", 
               "+5491100000000", 
               "Dirección por definir"
           ) {
               Ok(_) => "✅ ¡Pedido registrado! Revisa el Dashboard.".into(),
               Err(_) => "❌ Error de stock o base de datos.".into(),
           }
       }
        _ => "No te entendí. Escribe 'MENU' para ver los productos disponibles.".into(),
    }
}