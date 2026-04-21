use r2d2_sqlite::rusqlite::{params, Connection, Error, Result};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderUpdate {
    pub id: i64,
    pub customer_name: String,
    pub total: f64,
    pub status: String,
}

/// Inicializa las tablas necesarias en la base de datos
pub fn setup_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS products (
            id INTEGER PRIMARY KEY, 
            name TEXT, 
            price REAL, 
            stock INTEGER
        );
        CREATE TABLE IF NOT EXISTS orders (
            id INTEGER PRIMARY KEY AUTOINCREMENT, 
            customer_name TEXT, 
            total REAL, 
            status TEXT, 
            direccion TEXT, 
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS order_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT, 
            order_id INTEGER, 
            product_id INTEGER, 
            quantity INTEGER, 
            subtotal REAL, 
            FOREIGN KEY(order_id) REFERENCES orders(id), 
            FOREIGN KEY(product_id) REFERENCES products(id)
        );
        -- Insertar datos semilla si la tabla está vacía
        INSERT OR IGNORE INTO products (id, name, price, stock) VALUES (1, 'Pizza Especial', 15.0, 100);
        INSERT OR IGNORE INTO products (id, name, price, stock) VALUES (2, 'Bebida 500ml', 5.0, 200);"
    )
}

/// Procesa un pedido completo dentro de una transacción síncrona (ejecutada vía spawn_blocking)
#[instrument(skip(conn))]
pub fn procesar_pedido_completo(
    conn: &mut Connection,
    nombre_cliente: &str,
    direccion: &str,
    carrito: Vec<(i32, i32)>, // Vec<(id_producto, cantidad)>
) -> Result<OrderUpdate> {
    let start = std::time::Instant::now();

    // Iniciamos la transacción: Todo o nada
    let tx = conn.transaction()?;

    // 1. Insertar la cabecera de la orden
    tx.execute(
        "INSERT INTO orders (customer_name, total, status, direccion) VALUES (?, 0.0, 'PENDIENTE', ?)",
        params![nombre_cliente, direccion],
    )?;
    let order_id = tx.last_insert_rowid();

    let mut total_acumulado = 0.0;

    // 2. Procesar cada item del carrito
    for (p_id, cant) in carrito {
        // Obtener datos del producto y verificar stock
        let (_name, precio, stock_actual): (String, f64, i32) = tx.query_row(
            "SELECT name, price, stock FROM products WHERE id = ?",
            params![p_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;

        if stock_actual < cant {
            // Error de stock: rusqlite hará rollback automáticamente al soltar 'tx'
            return Err(Error::StatementChangedRows(0));
        }

        let subtotal = precio * (cant as f64);
        total_acumulado += subtotal;

        // Insertar el detalle del pedido
        tx.execute(
            "INSERT INTO order_items (order_id, product_id, quantity, subtotal) VALUES (?, ?, ?, ?)",
            params![order_id, p_id, cant, subtotal],
        )?;

        // Descontar del inventario
        tx.execute(
            "UPDATE products SET stock = stock - ? WHERE id = ?",
            params![cant, p_id],
        )?;
    }

    // 3. Actualizar el total final en la orden
    tx.execute(
        "UPDATE orders SET total = ? WHERE id = ?",
        params![total_acumulado, order_id],
    )?;

    // Confirmar cambios en disco
    tx.commit()?;

    info!(
        order_id = order_id,
        duration_ms = start.elapsed().as_millis(),
        "✅ Transacción DB exitosa"
    );

    Ok(OrderUpdate {
        id: order_id,
        customer_name: nombre_cliente.to_string(),
        total: total_acumulado,
        status: "PROCESADO".to_string(),
    })
}
