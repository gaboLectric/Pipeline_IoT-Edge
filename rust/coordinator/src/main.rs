use axum::{
    routing::{get, post},
    Json, Router, extract::State,
};
use common::{EdgeReport, Heartbeat, CoordStatus};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time;

// Estructura interna para rastrear la salud de cada Edge
#[derive(Debug, Clone)]
struct NodeHealth {
    last_seen_ms: u64,
    is_online: bool,
}

// El estado global del Coordinador
struct AppState {
    edge_nodes: Mutex<HashMap<String, NodeHealth>>,
    total_readings: Mutex<u64>,
    total_anomalies: Mutex<u32>,
    start_time_ms: u64,
}

#[tokio::main]
async fn main() {
    let port = "3000";
    
    let state = Arc::new(AppState {
        edge_nodes: Mutex::new(HashMap::new()),
        total_readings: Mutex::new(0),
        total_anomalies: Mutex::new(0),
        start_time_ms: current_time_ms(),
    });

    let bg_state = state.clone();

    // ---------------------------------------------------------
    // HILO EN SEGUNDO PLANO: El perro guardián (Detector de fallos)
    // ---------------------------------------------------------
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(3)); // Revisa cada 3 segundos
        let timeout_ms = 10_000; // 10s sin señales = nodo caído
    
        loop {
            interval.tick().await;
    
            let now = current_time_ms();
    
            let mut nodes = bg_state.edge_nodes
                .lock()
                .unwrap_or_else(|e| e.into_inner());
    
            for (id, health) in nodes.iter_mut() {
                let diff = now.saturating_sub(health.last_seen_ms);
    
                if health.is_online && diff > timeout_ms {
                    health.is_online = false;
    
                    println!(
                        "[ALERTA] Falla detectada en el nodo: {}. No responde desde hace {} ms.",
                        id,
                        diff
                    );
                }
            }
        }
    });

    // ---------------------------------------------------------
    // SERVIDOR AXUM
    // ---------------------------------------------------------
    let app = Router::new()
        .route("/submit_report", post(receive_report))
        .route("/heartbeat", post(receive_heartbeat))
        .route("/status", get(get_status))
        .with_state(state);

    println!("Coordinador iniciado y escuchando en el puerto {}", port);
    println!("Esperando reportes y heartbeats de los nodos Edge...");
    
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// 1. Endpoint para recibir los promedios del Edge
async fn receive_report(
    State(state): State<Arc<AppState>>,
    Json(report): Json<EdgeReport>,
) -> Json<&'static str> {
    
    let now = current_time_ms();
    let mut nodes = state.edge_nodes.lock().unwrap();
    
    // Lógica de reconexión
    if let Some(health) = nodes.get_mut(&report.edge_id) {
        if !health.is_online {
            println!("[RECUPERACIÓN] El nodo {} ha vuelto a conectarse tras una falla.", report.edge_id);
            health.is_online = true;
        }
        health.last_seen_ms = now;
    } else {
        println!("Nuevo nodo Edge registrado: {}", report.edge_id);
        nodes.insert(report.edge_id.clone(), NodeHealth { last_seen_ms: now, is_online: true });
    }

    // Actualizar métricas globales
    *state.total_readings.lock().unwrap() += report.sample_count as u64;
    if report.anomaly_detected {
        *state.total_anomalies.lock().unwrap() += 1;
        println!("[COORDINADOR] Reporte de anomalía recibido desde {}", report.edge_id);
    } else {
        println!("Reporte normal procesado desde {} ({} muestras)", report.edge_id, report.sample_count);
    }

    Json("Reporte procesado exitosamente")
}

// 2. Endpoint para Heartbeats puros (opcional pero útil)
async fn receive_heartbeat(
    State(state): State<Arc<AppState>>,
    Json(hb): Json<Heartbeat>,
) -> Json<&'static str> {
    let now = current_time_ms();
    let mut nodes = state.edge_nodes.lock().unwrap();
    
    if let Some(health) = nodes.get_mut(&hb.node_id) {
        health.last_seen_ms = now;
    }
    Json("Ack")
}

// 3. Endpoint de monitoreo de estado general
async fn get_status(State(state): State<Arc<AppState>>) -> Json<CoordStatus> {
    let nodes = state.edge_nodes.lock().unwrap();
    let active_edges = nodes.values().filter(|h| h.is_online).count() as u32;
    
    let status = CoordStatus {
        active_edges,
        total_readings: *state.total_readings.lock().unwrap(),
        anomalies_last_min: *state.total_anomalies.lock().unwrap(),
        uptime_s: (current_time_ms() - state.start_time_ms) / 1000,
    };
    
    Json(status)
}

// Función auxiliar
fn current_time_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}
