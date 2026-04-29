use common::SensorReading;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use rand::Rng;
use tokio::time;

#[tokio::main]
async fn main() {
    // Usamos variables de entorno para que Docker Compose pueda inyectar las IPs reales
    let sensor_id = std::env::var("SENSOR_ID").expect("FALTA VARIABLE: SENSOR_ID");
    let edge_url = std::env::var("EDGE_URL").expect("FALTA VARIABLE: EDGE_URL");

    println!("🌡️ Iniciando {}...", sensor_id);
    println!("📡 Enviando ráfagas de datos a {}", edge_url);

    let client = reqwest::Client::new();

    // Configuramos el sensor para enviar 2 lecturas por segundo (cada 500ms)
    let mut interval = time::interval(Duration::from_millis(500));
    let mut rng = rand::thread_rng();

    loop {
        interval.tick().await;

        // Temperatura base aleatoria (ej. 20.0 a 30.0 grados)
        let base_temp: f64 = rng.gen_range(20.0..30.0);
        
        // Simular una anomalía térmica (> 80.0) con un 5% de probabilidad
        let temp = if rng.gen_bool(0.05) {
            rng.gen_range(85.0..95.0)
        } else {
            base_temp
        };

        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Construimos el contrato de datos usando la librería común
        let reading = SensorReading {
            sensor_id: sensor_id.clone(),
            timestamp_ms,
            value: temp,
            unit: "Celsius".to_string(),
        };

        // Enviar la lectura vía HTTP POST al Edge
        match client.post(&edge_url).json(&reading).send().await {
            Ok(_) => {
                if temp > 80.0 {
                    println!("[Sensor] Anomalía inyectada: {:.2}°C", temp);
                } else {
                    println!("[Sensor] Dato enviado: {:.2}°C", temp);
                }
            },
            Err(e) => eprintln!("[Sensor] Error al enviar al Edge: {}", e),
        }
    }
}