use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use r2d2_sqlite::SqliteConnectionManager;
use std::sync::Arc;
use tokio::sync::{broadcast, Semaphore};

mod logic;

pub type DbPool = r2d2::Pool<SqliteConnectionManager>;

pub struct AppState {
    pub db_pool: DbPool,
    pub tx: broadcast::Sender<logic::OrderUpdate>,
    pub db_semaphore: Arc<Semaphore>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // 1. Inicializar DB Pool
    let manager = SqliteConnectionManager::file("nexus_orders.db");
    let pool = r2d2::Pool::new(manager).expect("Error al crear pool de DB");

    // 2. Preparar el Schema (Solo si no existe)
    {
        let conn = pool.get().unwrap();
        logic::setup_db(&conn).unwrap();
    }

    // 3. Estado compartido (Broadcast + Semáforo de 5 hilos)
    let (tx, _) = broadcast::channel(32);
    let state = Arc::new(AppState {
        db_pool: pool,
        tx,
        db_semaphore: Arc::new(Semaphore::new(5)),
    });

    // 4. Router
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/ws", get(ws_handler))
        .route("/simular", get(simular_pedido_handler)) // Para pruebas rápidas
        .with_state(state);

    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("📡 Nexus-Gateway corriendo en http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, _) = socket.split();
    let mut rx = state.tx.subscribe();

    // Notificación inicial para Playwright
    let _ = sender.send(Message::Text("SISTEMA ONLINE".into())).await;

    while let Ok(update) = rx.recv().await {
        if let Ok(msg) = serde_json::to_string(&update) {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    }
}

async fn simular_pedido_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let state_c = Arc::clone(&state);

    tokio::spawn(async move {
        // Clonar el Arc del semáforo para entregárselo a acquire_owned
        let _permit = state_c.db_semaphore.clone().acquire_owned().await.unwrap();

        // Clonar el estado para el hilo de bloqueo
        let state_for_db = state_c.clone();

        let update = tokio::task::spawn_blocking(move || {
            let mut conn = state_for_db.db_pool.get().unwrap();

            // Llamada real a la lógica
            logic::procesar_pedido_completo(
                &mut conn,
                "Santiago Urdaneta",
                "Calle 123",
                vec![(1, 1)],
            )
            .expect("Error en DB")
        })
        .await
        .unwrap();

        let _ = state_c.tx.send(update);
    });

    "Pedido en cola de procesamiento"
}
