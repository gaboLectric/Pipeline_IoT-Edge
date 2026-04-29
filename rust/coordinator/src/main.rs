use axum::{
    routing::{get, post},
    Json, Router, extract::State,
};
use common::{EdgeReport, Heartbeat, CoordStatus};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time;

// Colores ANSI por Edge (para distinguir en capturas de pantalla)
const EDGE_COLORS: [&str; 10] = [
    "\x1b[36m", // Cyan
    "\x1b[33m", // Amarillo
    "\x1b[35m", // Magenta
    "\x1b[32m", // Verde
    "\x1b[34m", // Azul
    "\x1b[91m", // Rojo claro
    "\x1b[96m", // Cyan claro
    "\x1b[93m", // Amarillo claro
    "\x1b[95m", // Magenta claro
    "\x1b[92m", // Verde claro
];
const NC: &str = "\x1b[0m"; // Reset

fn edge_color(edge_id: &str) -> &'static str {
    let mut hash: u64 = 0;
    for b in edge_id.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(b as u64);
    }
    EDGE_COLORS[(hash % EDGE_COLORS.len() as u64) as usize]
}

// Estructura interna para rastrear la salud de cada Edge
#[derive(Debug, Clone)]
struct NodeHealth {
    last_seen_ms: u64,
    is_online: bool,
    expected_seq: u64, // Para rastrear el siguiente mensaje esperado
}

// El estado global del Coordinador
struct AppState {
    edge_nodes: Mutex<HashMap<String, NodeHealth>>,
    total_readings: Mutex<u64>,
    total_anomalies: Mutex<u32>,
    start_time_ms: u64,
    latency_history: Mutex<Vec<u64>>, 
    lost_messages: Mutex<u64>,
}

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    
    let state = Arc::new(AppState {
        edge_nodes: Mutex::new(HashMap::new()),
        total_readings: Mutex::new(0),
        total_anomalies: Mutex::new(0),
        start_time_ms: current_time_ms(),
        latency_history: Mutex::new(Vec::with_capacity(1000)), // Evita realojamientos
        lost_messages: Mutex::new(0),
    });

    let bg_state = state.clone();

    // ---------------------------------------------------------
    // HILO EN SEGUNDO PLANO: El perro guardián (Detector de fallos)
    // ---------------------------------------------------------
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(2)); // Revisa cada 2 segundos
        let timeout_ms = 10_000; // 10s sin señales = nodo caído
    
        loop {
        interval.tick().await;
        let now = current_time_ms();
        let mut nodes = bg_state.edge_nodes.lock().unwrap();

        for (id, health) in nodes.iter_mut() {
            // Solo alertar si realmente ha pasado el tiempo Y el nodo estaba online
            if health.is_online {
                let diff = now.saturating_sub(health.last_seen_ms);
                if diff > timeout_ms {
                    health.is_online = false;
                    println!("{}[ALERTA] Falla detectada en {}. Sin señal por {} ms.{}", edge_color(id), id, diff, NC);
                }
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
    
    // Detección de Mensajes Perdidos
    if let Some(health) = nodes.get_mut(&report.edge_id) {
        health.last_seen_ms = now;
        if !health.is_online {
            println!("{}[RECUPERACIÓN] El nodo {} ha vuelto a conectarse.{}", edge_color(&report.edge_id), report.edge_id, NC);
            health.is_online = true;
        }
        
        // Verificamos si hubo salto de secuencia
        if report.sequence_number > health.expected_seq && health.expected_seq > 0 {
            let lost = report.sequence_number - health.expected_seq;
            *state.lost_messages.lock().unwrap() += lost;
            println!("{}[ADVERTENCIA] Se perdieron {} mensajes del nodo {}{}", edge_color(&report.edge_id), lost, report.edge_id, NC);
        }
        // Actualizamos la siguiente secuencia esperada
        health.expected_seq = report.sequence_number + 1;

    } else {
        println!("{}[REGISTRO] Nuevo nodo Edge registrado: {}{}", edge_color(&report.edge_id), report.edge_id, NC);
        nodes.insert(report.edge_id.clone(), NodeHealth { 
            last_seen_ms: now, 
            is_online: true,
            expected_seq: report.sequence_number + 1 
        });
    }

    // Registrar la latencia (limitamos a 1000 para no agotar la memoria)
    {
        let mut history = state.latency_history.lock().unwrap();
        if history.len() >= 1000 {
            history.remove(0); // Eliminamos el más viejo si llegamos al límite
        }
        history.push(report.latency_ms);
    }

    // Actualizar métricas globales
    *state.total_readings.lock().unwrap() += report.sample_count as u64;
    let net_latency = now.saturating_sub(report.timestamp_ms);
    if report.anomaly_detected {
        *state.total_anomalies.lock().unwrap() += 1;
        println!("{}[ANOMALÍA] {} - Latencia red: {}ms{}", edge_color(&report.edge_id), report.edge_id, net_latency, NC);
    } else {
        println!("{}[REPORTE] {} ({} muestras) - Latencia red: {}ms{}", edge_color(&report.edge_id), report.edge_id, report.sample_count, net_latency, NC);
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
    
    // Calcular latencia del heartbeat: tiempo actual - timestamp que envió el Edge
    let heartbeat_latency = now.saturating_sub(hb.timestamp_ms);
    
    // NUEVA LÓGICA: Si el nodo no existe, lo creamos (Auto-registro)
    if let Some(health) = nodes.get_mut(&hb.node_id) {
        health.last_seen_ms = now;
        if !health.is_online {
            health.is_online = true;
            println!("{}[RECUPERACIÓN] {} volvió vía heartbeat (latencia: {}ms){}", edge_color(&hb.node_id), hb.node_id, heartbeat_latency, NC);
        } else {
            println!("{}[HEARTBEAT] {} - Latencia: {}ms{}", edge_color(&hb.node_id), hb.node_id, heartbeat_latency, NC);
        }
    } else {
        println!("{}[REGISTRO] Nuevo nodo {} detectado vía heartbeat (latencia: {}ms){}", edge_color(&hb.node_id), hb.node_id, heartbeat_latency, NC);
        nodes.insert(hb.node_id.clone(), NodeHealth { 
            last_seen_ms: now, 
            is_online: true,
            expected_seq: 0 
        });
    }
    
    Json("Ack")
}

// 3. Endpoint de monitoreo de estado general
async fn get_status(State(state): State<Arc<AppState>>) -> Json<CoordStatus> {
    let nodes = state.edge_nodes.lock().unwrap();
    let active_edges = nodes.values().filter(|h| h.is_online).count() as u32;
    
    let total_r = *state.total_readings.lock().unwrap();
    let total_a = *state.total_anomalies.lock().unwrap();
    let lost_m = *state.lost_messages.lock().unwrap();
    
    let uptime_s = (current_time_ms() - state.start_time_ms) / 1000;
    
    // Throughput y Tasa de anomalías
    let throughput = if uptime_s > 0 {
        total_r as f64 / uptime_s as f64
    } else {
        0.0
    };

    let anomaly_rate = if total_r > 0 {
        (total_a as f64 / total_r as f64) * 100.0
    } else {
        0.0
    };

    // Percentiles P50 y P99
    let mut p50 = 0;
    let mut p99 = 0;
    
    let mut lats = state.latency_history.lock().unwrap().clone();
    if !lats.is_empty() {
        lats.sort_unstable(); // Esencial ordenar para percentiles
        let len = lats.len() as f64;
        
        let idx_50 = (len * 0.50).round() as usize;
        let idx_99 = (len * 0.99).round() as usize;
        
        // Evitamos out-of-bounds restando 1 de forma segura
        p50 = lats[idx_50.saturating_sub(1)];
        p99 = lats[idx_99.saturating_sub(1)];
    }
    
    let status = CoordStatus {
        active_edges,
        total_readings: total_r,
        anomalies_last_min: total_a,
        uptime_s,
        throughput_msg_s: throughput,
        anomaly_rate_pct: anomaly_rate,
        latency_p50_ms: p50,
        latency_p99_ms: p99,
        lost_messages: lost_m,
    };
    
    Json(status)
}

// Función auxiliar
fn current_time_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}
