use axum::{
    routing::post,
    Json, Router, extract::State,
};
use common::{SensorReading, EdgeReport};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tokio::time;
use std::sync::atomic::{AtomicU64, Ordering};

// Este es el estado compartido en memoria.
// Axum guarda las lecturas aquí, y la tarea en segundo plano las toma de aquí.
struct AppState {
    readings: Arc<Mutex<Vec<SensorReading>>>,
    edge_id: String,
    coordinator_url: String,
    sequence_counter: Arc<AtomicU64>,
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
        sequence_counter: Arc::new(AtomicU64::new(0)),
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

            // 4. Procesamiento local con ventanas estrictas de 10 muestras
            let mut total_sum = 0.0;
            let mut total_count = 0;
            let mut anomaly_detected = false;
            
            // Procesar en ventanas exactas de 10 muestras
            for chunk in readings_to_process.chunks(10) {
                if chunk.len() == 10 {
                    // Calcular promedio móvil de esta ventana
                    let window_sum: f64 = chunk.iter().map(|r| r.value).sum();
                    let window_avg = window_sum / 10.0;
                    
                    total_sum += window_sum;
                    total_count += 10;
                    
                    // Detectar anomalía si el promedio de esta ventana supera 80.0
                    if window_avg > 80.0 {
                        anomaly_detected = true;
                    }
                    
                    println!("[Edge] Ventana procesada: promedio {:.2}, muestras: {}", window_avg, chunk.len());
                }
                // Ignorar ventanas incompletas (menos de 10 muestras)
            }
            
            // Si no tenemos ventanas completas, continuar
            if total_count == 0 {
                continue;
            }
            
            let window_avg = total_sum / total_count as f64;
            let count = total_count as u32;

            // 5. CÁLCULO DE LATENCIA E2E 
            let total_latency: u64 = readings_to_process.iter()
                .map(|r| now.saturating_sub(r.timestamp_ms))
                .sum();
            let avg_latency = total_latency / count as u64;

            // 6. Generar número de secuencia y armar el reporte
            let sequence_number = bg_state.sequence_counter.fetch_add(1, Ordering::SeqCst);
            let report = EdgeReport {
                edge_id: bg_state.edge_id.clone(),
                window_avg,
                anomaly_detected,
                sample_count: count,
                latency_ms: avg_latency as u64,
                sequence_number,
            };

            println!("[Edge] Procesadas {} lecturas. Latencia prom: {}ms. Enviando...", 
                count, avg_latency);

            // 7. Enviar vía Reqwest con reconexión automática usando backoff exponencial
            let mut retry_delay = Duration::from_millis(100); // Start with 100ms
            let max_delay = Duration::from_secs(30); // Max 30 seconds
            let mut attempts = 0;
            const MAX_ATTEMPTS: u32 = 5;
            
            while attempts < MAX_ATTEMPTS {
                match client.post(&bg_state.coordinator_url).json(&report).send().await {
                    Ok(response) => {
                        if response.status().is_success() {
                            println!("[Edge] Reporte enviado exitosamente (intento {})", attempts + 1);
                            break;
                        } else {
                            eprintln!("[Edge] Error del servidor: {}", response.status());
                        }
                    }
                    Err(e) => {
                        eprintln!("[Edge] Fallo al contactar al coordinador (intento {}): {}", attempts + 1, e);
                    }
                }
                
                attempts += 1;
                if attempts < MAX_ATTEMPTS {
                    println!("[Edge] Reintentando en {:?}...", retry_delay);
                    tokio::time::sleep(retry_delay).await;
                    
                    // Exponential backoff: double the delay for next attempt
                    retry_delay = std::cmp::min(retry_delay * 2, max_delay);
                }
            }
            
            if attempts == MAX_ATTEMPTS {
                eprintln!("[Edge] No se pudo enviar el reporte después de {} intentos", MAX_ATTEMPTS);
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