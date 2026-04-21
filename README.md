# ⚡ Rust Order Engine

Un motor de pedidos ultra-ligero diseñado para máximo rendimiento en hardware legacy.

### 🚀 Stack Tecnológico
- **Backend:** Rust con `Axum` (Servidor Web) y `Tokio` (Runtime asíncrono).
- **Persistencia:** `SQLite` con pool de conexiones `r2d2`.
- **Real-time:** WebSockets con `tokio-broadcast` para actualizaciones instantáneas.
- **Testing:** Pruebas E2E automáticas con `Playwright`.

### 🛡️ Características Clave
- **Control de Concurrencia:** Uso de semáforos para prevenir saturación de I/O en discos HDD.
- **Zero-Node Backend:** Máxima eficiencia de memoria (aprox. 15MB RAM en reposo).
- **Atomicidad:** Transacciones SQL completas para asegurar stock e integridad.

Frontend (HTML5/Bulma): Se conecta vía WebSocket al servidor.

Motor (Axum/Tokio): Recibe peticiones y las gestiona de forma asíncrona.

Semáforo Atómico: Controla que solo 5 hilos toquen el disco (SQLite) al mismo tiempo para evitar el cuello de botella del hardware.

Broadcast Channel: Una vez que la DB confirma el guardado, el mensaje se "difunde" a todos los dashboards conectados en milisegundos.