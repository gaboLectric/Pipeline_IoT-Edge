use axum::{
    routing::post,
    Json, Router, extract::State,
};
use common::{SensorReading, EdgeReport};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tokio::time;

// Este es el estado compartido en memoria.
// Axum guarda las lecturas aquí, y la tarea en segundo plano las toma de aquí.
struct AppState {
    readings: Arc<Mutex<Vec<SensorReading>>>,
    edge_id: String,
    coordinator_url: String,
}

#[tokio::main]
async fn main() {
    // Configuración básica (luego lo pasaremos a variables de entorno para Docker)
    let edge_id = "edge-1".to_string();
    let coordinator_url = "http://127.0.0.1:3000/submit_report".to_string();
    let port = "4000";

    // Inicializamos el estado vacío
    let shared_state = Arc::new(AppState {
        readings: Arc::new(Mutex::new(Vec::new())),
        edge_id: edge_id.clone(),
        coordinator_url: coordinator_url.clone(),
    });

    // Clonamos el puntero del estado para dárselo al hilo en segundo plano
    let bg_state = shared_state.clone();

    // ---------------------------------------------------------
    // HILO EN SEGUNDO PLANO: Procesa y envía al Coordinador
    // ---------------------------------------------------------
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        // Ajustado a 2 segundos para evitar "flapping"
        let mut interval = time::interval(Duration::from_secs(2));
        let mut seq_num: u64 = 0; // <-- Inicializamos el contador de secuencia

        loop {
            // 1. ESPERAR al siguiente ciclo
            interval.tick().await;
            
            // 2. OBTENER el tiempo actual JUSTO después del tick para mayor precisión
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
            
            // 3. Extraer y limpiar las lecturas
            let readings_to_process: Vec<SensorReading> = {
                let mut lock = bg_state.readings.lock().unwrap();
                let data = lock.clone();
                lock.clear(); 
                data
            };

            if readings_to_process.is_empty() {
                continue; 
            }

            seq_num += 1; // <-- Incrementamos en cada envío exitoso

            // 4. Procesamiento local (Promedio y Anomalías)
            let count = readings_to_process.len() as u32;
            let sum: f64 = readings_to_process.iter().map(|r| r.value).sum();
            let window_avg = sum / count as f64;
            let anomaly_detected = window_avg > 80.0;

            // 5. CÁLCULO DE LATENCIA E2E 
            let total_latency: u64 = readings_to_process.iter()
                .map(|r| now.saturating_sub(r.timestamp_ms))
                .sum();
            let avg_latency = total_latency / count as u64;

            // 6. Armar el reporte con la latencia REAL 
            let report = EdgeReport {
                edge_id: bg_state.edge_id.clone(),
                window_avg,
                anomaly_detected,
                sample_count: count,
                latency_ms: avg_latency as u64,
                sequence_number: seq_num, // <-- Adjuntamos la secuencia
            };

            println!("[Edge] Procesadas {} lecturas. Latencia prom: {}ms. Enviando...", 
                count, avg_latency);

            // 7. Enviar vía Reqwest
            if let Err(e) = client.post(&bg_state.coordinator_url).json(&report).send().await {
                eprintln!("[Edge] Fallo al contactar al coordinador: {}", e);
            }
        }
    });

    // ---------------------------------------------------------
    // HILO DE HEARTBEAT: Avisa al coordinador que el Edge está vivo
    // ---------------------------------------------------------
    let hb_state = shared_state.clone();
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let mut interval = time::interval(Duration::from_secs(3)); // Latido cada 3 segundos
        let hb_url = format!("{}/heartbeat", hb_state.coordinator_url.replace("/submit_report", ""));

        loop {
            interval.tick().await;

            let heartbeat = common::Heartbeat {
                node_id: hb_state.edge_id.clone(),
                role: "edge".to_string(),
                timestamp_ms: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            };

            match client.post(&hb_url).json(&heartbeat).send().await {
                Ok(_) => {}, // Éxito silencioso para no llenar el log
                Err(e) => eprintln!("[Edge] ❤️ Heartbeat fallido: {}", e),
            }
        }
    });

    // ---------------------------------------------------------
    // SERVIDOR AXUM: Escucha a los sensores
    // ---------------------------------------------------------
    let app = Router::new()
        .route("/reading", post(receive_reading))
        .with_state(shared_state);

    println!("Nodo Edge '{}' inicializado y escuchando en el puerto {}", edge_id, port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Endpoint que procesa el POST de los sensores
async fn receive_reading(
    State(state): State<Arc<AppState>>,
    Json(reading): Json<SensorReading>,
) -> Json<&'static str> {
    // Descomenta esto si quieres ver cada lectura en la consola (puede ser mucho texto)
    // println!("[Edge] Lectura recibida: {} de {}", reading.value, reading.sensor_id);
    
    // Guardamos la lectura en el arreglo de memoria protegido por el Mutex
    let mut lock = state.readings.lock().unwrap();
    lock.push(reading);
    
    Json("OK")
}
