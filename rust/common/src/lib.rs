use serde::{Deserialize, Serialize};

// Datos crudos que genera el nodo Sensor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    pub sensor_id: String,
    pub timestamp_ms: u64,
    pub value: f64,
    pub unit: String,
}

// Resumen procesado que el nodo Edge envía al Coordinator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeReport {
    pub edge_id: String,
    pub window_avg: f64,
    pub anomaly_detected: bool,
    pub sample_count: u32,
    pub latency_ms: u64,
    pub sequence_number: u64,
}

// Estado general que mantendrá el Coordinator en memoria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordStatus {
    pub active_edges: u32,
    pub total_readings: u64,
    pub anomalies_last_min: u32,
    pub uptime_s: u64,
    pub throughput_msg_s: f64,
    pub anomaly_rate_pct: f64,
    pub latency_p50_ms: u64,
    pub latency_p99_ms: u64,
    pub lost_messages: u64,
}

// Señal de vida para la tolerancia a fallos (vital para las pruebas con tc netem)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub node_id: String,
    pub role: String, // "sensor", "edge", "coordinator"
    pub timestamp_ms: u64,
}