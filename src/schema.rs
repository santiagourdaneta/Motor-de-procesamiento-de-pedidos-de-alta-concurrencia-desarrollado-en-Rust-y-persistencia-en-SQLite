use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
pub use r2d2_sqlite::rusqlite::Result; 

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn create_pool() -> DbPool {
    let manager = SqliteConnectionManager::file("negocio.db");
    // Configuramos el pool: 10 conexiones simultáneas listas
    Pool::builder()
        .max_size(10) 
        .build(manager)
        .expect("Fallo al crear el pool de conexiones")
}

pub fn setup_db(pool: &DbPool) -> Result<()> {
    let conn = pool.get().expect("No se pudo obtener conexión del pool");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS products (id INTEGER PRIMARY KEY, name TEXT, price REAL, stock INTEGER);
         CREATE TABLE IF NOT EXISTS orders (id INTEGER PRIMARY KEY AUTOINCREMENT, customer_id INTEGER, total REAL, status TEXT, direccion TEXT, created_at DATETIME DEFAULT CURRENT_TIMESTAMP);
        CREATE TABLE IF NOT EXISTS customers (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            wa_number   TEXT NOT NULL UNIQUE,
            name        TEXT
        );
        CREATE TABLE IF NOT EXISTS order_items (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        order_id    INTEGER NOT NULL,
        product_id  INTEGER NOT NULL,
        quantity    INTEGER NOT NULL,
        subtotal    REAL NOT NULL,
        FOREIGN KEY(order_id) REFERENCES orders(id),
        FOREIGN KEY(product_id) REFERENCES products(id)
    )"
    )
}

 