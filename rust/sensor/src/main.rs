use common::SensorReading;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use rand::Rng;
use tokio::time;

#[tokio::main]
async fn main() {
    // Usamos variables de entorno para que Docker Compose pueda inyectar las IPs reales
    let sensor_id = std::env::var("SENSOR_ID").expect("FALTA VARIABLE: SENSOR_ID");
    let edge_url = std::env::var("EDGE_URL").expect("FALTA VARIABLE: EDGE_URL");
    let coord_hb_url = std::env::var("COORD_HB_URL").expect("FALTA VARIABLE: COORD_HB_URL");
    
    println!("🌡️ Iniciando {}...", sensor_id);
    println!("📡 Enviando ráfagas de datos a {}", edge_url);
    println!("❤️ Heartbeat configurado hacia {}", coord_hb_url);
    
    let client = reqwest::Client::new();
    
    // Hilo de Heartbeat para el Sensor
    let sensor_id_hb = sensor_id.clone();
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let mut interval = time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            let hb = common::Heartbeat {
                node_id: sensor_id_hb.clone(),
                role: "sensor".to_string(),
                timestamp_ms: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
            };
            let _ = client.post(&coord_hb_url).json(&hb).send().await;
        }
    });
    
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