use axum::{
    routing::post,
    Json, Router, extract::State,
};
use common::{SensorReading, EdgeReport};
use std::sync::{Arc, Mutex};
use std::time::Duration;
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
    let coordinator_url = "http://10.10.10.1:3000/submit_report".to_string();
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
        // Configuramos el ciclo para ejecutarse cada 5 segundos
        let mut interval = time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;
            
            // 1. Extraer y limpiar las lecturas (rápido para no bloquear el mutex)
            let readings_to_process: Vec<SensorReading> = {
                let mut lock = bg_state.readings.lock().unwrap();
                let data = lock.clone();
                lock.clear(); // Vaciamos el búfer después de copiar
                data
            };

            if readings_to_process.is_empty() {
                continue; // Si no hay datos de sensores, no enviamos reporte
            }

            // 2. Procesamiento local (Promedio Móvil y Anomalías)
            let count = readings_to_process.len() as u32;
            let sum: f64 = readings_to_process.iter().map(|r| r.value).sum();
            let window_avg = sum / count as f64;
            
            // Lógica simple de anomalía (ej: Temperatura mayor a 80 grados)
            let anomaly_detected = window_avg > 80.0;

            // 3. Armar el reporte usando la estructura de `common`
            let report = EdgeReport {
                edge_id: bg_state.edge_id.clone(),
                window_avg,
                anomaly_detected,
                sample_count: count,
                latency_ms: 0, // Aquí luego inyectaremos la medición de red
            };

            println!("[Edge] Procesadas {} lecturas. Promedio: {:.2}. Anomalía: {}. Enviando a coordinador...", 
                count, window_avg, anomaly_detected);

            // 4. Enviar vía Reqwest
            if let Err(e) = client.post(&bg_state.coordinator_url).json(&report).send().await {
                eprintln!("[Edge] Fallo al contactar al coordinador: {}", e);
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