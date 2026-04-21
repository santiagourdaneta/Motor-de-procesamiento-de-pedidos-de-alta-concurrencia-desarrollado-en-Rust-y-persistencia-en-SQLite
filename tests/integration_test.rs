// tests/integration_test.rs
use sistema_pedidos_rust::{db, schema};

#[tokio::test]
async fn test_flujo_completo_db_real() {
    let pool = schema::create_pool();
    let mut conn = pool.get().unwrap();
    let (tx, _) = tokio::sync::broadcast::channel(16);

    let res = db::procesar_pedido_completo(
        &mut conn,
        1,
        vec![(1, 1)],
        &tx,
        "Test",
        "123",
        "Calle Falsa",
    );
    assert!(res.is_ok());
}
