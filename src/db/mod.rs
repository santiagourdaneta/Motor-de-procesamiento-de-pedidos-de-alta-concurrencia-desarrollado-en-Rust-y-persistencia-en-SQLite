use r2d2_sqlite::rusqlite::{params, Connection, Result, Error};
use tracing::{info, warn, instrument};

#[instrument(skip(conn, notifier))] // Esto mide automáticamente el tiempo de la función
pub fn procesar_pedido_completo(
    conn: &mut Connection, 
    customer_id: i32, 
    carrito: Vec<(i32, i32)>, // Lista de (id_producto, cantidad)
    notifier: &tokio::sync::broadcast::Sender<String>,
    nombre_cliente: &str,
    telefono: &str,
    direccion: &str
) -> Result<()> {
    let start = std::time::Instant::now();
    let tx = conn.transaction()?;

    // 1. Insertar la orden con la dirección
    tx.execute(
        "INSERT INTO orders (customer_id, total, status, direccion) VALUES (?, 0.0, 'PENDIENTE', ?)",
        params![customer_id, direccion],
    )?;
    let order_id = tx.last_insert_rowid();

    let mut total_acumulado = 0.0;
    let mut lista_items_texto = Vec::new();

    // 2. Procesar múltiples productos
        for (p_id, cant) in carrito {
            // Obtenemos precio y stock real
            let (name, precio, stock_actual): (String, f64, i32) = tx.query_row(
                "SELECT name, price, stock FROM products WHERE id = ?",
                params![p_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?;

            // VALIDACIÓN CRÍTICA:
            if stock_actual < cant {
                    return Err(Error::StatementChangedRows(0)); 
                }
            let subtotal = precio * (cant as f64);
            total_acumulado += subtotal;
            lista_items_texto.push(format!("{} x{}", name, cant));

            // Insertar item
            tx.execute(
                "INSERT INTO order_items (order_id, product_id, quantity, subtotal) VALUES (?, ?, ?, ?)",
                params![order_id, p_id, cant, subtotal],
            )?;

            // DESCUENTO DE STOCK:
            tx.execute(
                "UPDATE products SET stock = stock - ? WHERE id = ?",
                params![cant, p_id],
            )?;
        }
    tx.execute("UPDATE orders SET total = ? WHERE id = ?", params![total_acumulado, order_id])?;
    tx.commit()?;

    

    info!(
        order_id = order_id, 
        duration_ms = start.elapsed().as_millis(),
        "✅ Pedido procesado exitosamente"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema;
    use tokio::sync::broadcast;

    // Función auxiliar para crear un entorno de prueba limpio
    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        // Ejecutamos el schema manualmente para la DB en memoria
        conn.execute_batch(
            "CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT, price REAL, stock INTEGER);
             CREATE TABLE customers (id INTEGER PRIMARY KEY, wa_number TEXT, name TEXT);
             CREATE TABLE orders (id INTEGER PRIMARY KEY AUTOINCREMENT, customer_id INTEGER, total REAL, status TEXT, direccion TEXT);
             CREATE TABLE order_items (id INTEGER PRIMARY KEY AUTOINCREMENT, order_id INTEGER, product_id INTEGER, quantity INTEGER, subtotal REAL);"
        ).unwrap();
        
        // Insertamos datos semilla
        conn.execute("INSERT INTO products VALUES (1, 'Pizza', 10.0, 5)", []).unwrap();
        conn.execute("INSERT INTO customers VALUES (1, '123', 'Test User')", []).unwrap();
        conn
    }

    #[test]
    fn test_pedido_exitoso_y_descuento_stock() {
        let mut conn = setup_test_db();
        let (tx, _) = broadcast::channel(16);
        
        let carrito = vec![(1, 2)]; // 2 Pizzas de $10.0
        
        let resultado = procesar_pedido_completo(
            &mut conn, 1, carrito, &tx, "User", "123", "Calle 1"
        );

        assert!(resultado.is_ok());

        // Verificar que el stock bajó de 5 a 3
        let stock: i32 = conn.query_row("SELECT stock FROM products WHERE id = 1", [], |r| r.get(0)).unwrap();
        assert_eq!(stock, 3);

        // Verificar que el total es $20.0
        let total: f64 = conn.query_row("SELECT total FROM orders WHERE id = 1", [], |r| r.get(0)).unwrap();
        assert_eq!(total, 20.0);
    }

    #[test]
    fn test_error_por_stock_insuficiente() {
        let mut conn = setup_test_db();
        let (tx, _) = broadcast::channel(16);
        
        let carrito = vec![(1, 10)]; // Pedimos 10, pero solo hay 5
        
        let resultado = procesar_pedido_completo(
            &mut conn, 1, carrito, &tx, "User", "123", "Calle 1"
        );

        // Debe fallar
        assert!(resultado.is_err());

        // El stock debe seguir intacto (gracias a la Transacción SQL)
        let stock: i32 = conn.query_row("SELECT stock FROM products WHERE id = 1", [], |r| r.get(0)).unwrap();
        assert_eq!(stock, 5);
    }
}