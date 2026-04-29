# Configuración Docker del Pipeline IoT/Edge

Este directorio contiene las configuraciones Docker para los diferentes modos de operación del sistema.

## Archivos Disponibles

### 1. `docker-compose.yml` (Legacy - Pruebas Locales)

**Uso:** Desarrollo y pruebas en una sola máquina.

```bash
cd docker
docker-compose up --build
```

**Características:**
- Todos los servicios en una sola máquina
- Comunicación por red Docker bridge (`iot-network`)
- Sin requisitos de VPN
- Ideal para desarrollo y debugging inicial

---

### 2. `docker-compose.coordinator.yml` (Cluster - Hub)

**Uso:** Ejecutar en la máquina designada como **Hub** (Coordinator).

**Requisitos:**
- IP fija en la VPN: `10.10.10.1`
- Puerto 3000 expuesto al exterior
- WireGuard configurado y activo

```bash
# En la máquina Hub (10.10.10.1)
cd docker
docker-compose -f docker-compose.coordinator.yml up --build
```

**Servicios:**
- `coordinator`: Recibe heartbeats y reportes de todos los edges vía VPN

---

### 3. `docker-compose.edge.yml` (Cluster - Nodos Edge)

**Uso:** Ejecutar en **cada máquina** del equipo (excepto el Hub).

**Requisitos:**
- WireGuard conectado al Hub
- Variable `COORDINATOR_IP` configurada
- Variable `EDGE_ID` único por máquina

```bash
# En cada máquina edge (ej: 10.10.10.2, 10.10.10.3, etc.)
export COORDINATOR_IP=10.10.10.1
export EDGE_ID=edge-node-2  # Único por máquina

cd docker
docker-compose -f docker-compose.edge.yml up --build
```

**Servicios:**
- `edge`: Procesa datos y envía al coordinator vía VPN
- `sensor-1`, `sensor-2`: Generan datos (comunicación local con edge)

---

## Arquitectura del Cluster

```
┌─────────────────────────────────────────────────────────────────┐
│                         VPN 10.10.10.0/24                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐                          ┌──────────────┐   │
│  │   Hub        │◄─────────────────────────│  Máquina 2   │   │
│  │  10.10.10.1  │      Heartbeats          │  10.10.10.2  │   │
│  │              │◄─────────────────────────│   [Edge]     │   │
│  │ [Coordinator]│      Edge Reports       │   /\         │   │
│  │      │       │                          │   /  \       │   │
│  │      │       │                          │[S1]  [S2]     │   │
│  └──────┼───────┘                          └──────────────┘   │
│         │                                                     │
│         │                          ┌──────────────┐            │
│         │                          │  Máquina 3   │            │
│         └─────────────────────────►│  10.10.10.3  │            │
│            Heartbeats/Reports      │   [Edge]     │            │
│                                    │   /\         │            │
│                                    │   /  \       │            │
│                                    │[S1]  [S2]     │            │
│                                    └──────────────┘            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Guía Rápida de Despliegue

### Paso 1: Preparar el Hub

1. Configurar WireGuard con IP `10.10.10.1`
2. Abrir puerto 3000 en el firewall
3. Ejecutar:
   ```bash
   docker-compose -f docker-compose.coordinator.yml up --build
   ```

### Paso 2: Preparar cada Máquina Edge

1. Configurar WireGuard conectado al Hub
2. Asignar IP única (10.10.10.2, 10.10.10.3, etc.)
3. Verificar conectividad: `ping 10.10.10.1`
4. Ejecutar:
   ```bash
   export COORDINATOR_IP=10.10.10.1
   export EDGE_ID=edge-node-X  # X = número de máquina
   docker-compose -f docker-compose.edge.yml up --build
   ```

### Paso 3: Verificación

En el Hub, revisar logs del coordinator:
```bash
docker logs coordinator
```

Deberías ver heartbeats llegando de los diferentes edges.

## Variables de Entorno

### Coordinator
- `PORT`: Puerto de escucha (default: 3000)
- `BIND_ADDRESS`: Interfaz de red (default: 0.0.0.0)
- `RUST_LOG`: Nivel de logging (default: info)

### Edge
- `COORDINATOR_IP`: IP del Hub en VPN (requerido)
- `EDGE_ID`: Identificador único del edge (requerido)
- `PORT`: Puerto local del edge (default: 4000)

### Sensor
- `EDGE_URL`: URL del edge local (default: http://localhost:4000/reading)
- `SENSOR_ID`: Identificador del sensor

## Solución de Problemas

### Los edges no pueden conectar al coordinator
- Verificar VPN: `ping 10.10.10.1` desde la máquina edge
- Verificar que el coordinator escuche en 0.0.0.0 (no solo localhost)
- Revisar firewall del Hub

### Los sensores no pueden conectar al edge
- Verificar que el edge esté corriendo: `docker ps`
- Verificar `network_mode: host` en los sensores
- Probar: `curl http://localhost:4000/reading` desde la máquina

### Heartbeats no llegan
- Verificar `COORD_HB_URL` apunte a `http://10.10.10.1:3000/heartbeat`
- Revisar logs del edge: `docker logs edge-processor`
- Verificar logs del coordinator: `docker logs coordinator`
