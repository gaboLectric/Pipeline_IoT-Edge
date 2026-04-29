# Guía de Despliegue del Cluster IoT/Edge

Esta guía describe paso a paso cómo desplegar el sistema en modo cluster distribuido.

## Requisitos del Sistema

Cada máquina necesita:
- Docker 20.10+ y Docker Compose 2.0+
- WireGuard instalado
- 2GB RAM mínimo
- Conexión a internet (para descargar imágenes Docker)

## Arquitectura del Despliegue

```
┌─────────────────────────────────────────────────────────────┐
│                    VPN WireGuard 10.10.10.0/24              │
│                                                             │
│  ┌─────────────────┐          ┌─────────────────┐           │
│  │     HUB         │          │   Edge Node 1   │           │
│  │  10.10.10.1     │◄────────►│  10.10.10.2     │           │
│  │                 │   VPN      │                 │           │
│  │ ┌─────────────┐ │          │ ┌─────────────┐ │           │
│  │ │ Coordinator │ │          │ │    Edge     │ │           │
│  │ │   :3000     │ │          │ │   :4000     │ │           │
│  │ └─────────────┘ │          │ └──────┬──────┘ │           │
│  │                 │          │        │        │           │
│  │ ┌─────────────┐ │          │ ┌──────┴──────┐ │           │
│  │ │   Edge      │ │          │ │  Sensores   │ │           │
│  │ │  (opcional) │ │          │ │  (local)    │ │           │
│  │ └─────────────┘ │          │ └─────────────┘ │           │
│  └─────────────────┘          └─────────────────┘           │
│           ▲                                                   │
│           │          ┌─────────────────┐                      │
│           │          │   Edge Node 2   │                      │
│           └─────────►│  10.10.10.3     │                      │
│             VPN      │                 │                      │
│                      │ ┌─────────────┐ │                      │
│                      │ │    Edge     │ │                      │
│                      │ │   :4000     │ │                      │
│                      │ └──────┬──────┘ │                      │
│                      │        │        │                      │
│                      │ ┌──────┴──────┐ │                      │
│                      │ │  Sensores   │ │                      │
│                      │ └─────────────┘ │                      │
│                      └─────────────────┘                      │
└─────────────────────────────────────────────────────────────┘
```

## Preparación Previa (Todos los Integrantes)

### 1. Clonar el Repositorio

```bash
git clone https://github.com/gaboLectric/Pipeline_IoT-Edge.git
cd Pipeline_IoT-Edge
```

### 2. Instalar WireGuard

**macOS:**
```bash
brew install wireguard-tools
```

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install wireguard
```

**Windows:**
Descargar instalador desde [wireguard.com](https://www.wireguard.com/install/)

### 3. Verificar Docker

```bash
docker --version
docker-compose --version
```

## Paso 1: Configurar la VPN WireGuard

Ver guía completa en: `vpn/README.md`

**Resumen rápido:**

### En el Hub (quien será Coordinator):

1. Generar claves: `wg genkey | tee hub.key | wg pubkey > hub.pub`
2. Crear `/etc/wireguard/wg0.conf` con IP `10.10.10.1/24`
3. Compartir `hub.pub` con todos los integrantes

### En cada Edge:

1. Generar claves: `wg genkey | tee edge.key | wg pubkey > edge.pub`
2. Crear `wg0.conf` con IP asignada (10.10.10.2, 10.10.10.3, etc.)
3. Compartir `edge.pub` con el Hub
4. Iniciar: `sudo wg-quick up wg0`
5. Verificar: `ping 10.10.10.1`

## Paso 2: Desplegar el Coordinator (Solo en el Hub)

### Opción A: Usar el script de despliegue (Recomendado)

```bash
cd Pipeline_IoT-Edge
./scripts/deploy-coordinator.sh
```

El script:
- Verifica prerequisitos
- Verifica conectividad VPN
- Construye e inicia el Coordinator
- Muestra logs en tiempo real

### Opción B: Manual

```bash
cd docker
export PORT=3000
docker-compose -f docker-compose.coordinator.yml up --build
```

### Verificar que funciona

Desde el mismo Hub:
```bash
curl http://localhost:3000
docker logs coordinator
```

## Paso 3: Desplegar los Edges (Cada Integrante)

### Opción A: Usar el script de despliegue (Recomendado)

```bash
cd Pipeline_IoT-Edge

# Definir variables
export EDGE_ID="edge-rogelio"  # Identificador único
export COORDINATOR_IP="10.10.10.1"

./scripts/deploy-edge.sh
```

El script:
- Verifica conectividad VPN
- Verifica que el Coordinator responde
- Construye e inicia Edge + Sensores
- Muestra logs

### Opción B: Manual

```bash
cd docker

export COORDINATOR_IP=10.10.10.1
export EDGE_ID=edge-rogelio

docker-compose -f docker-compose.edge.yml up --build
```

## Paso 4: Verificación del Cluster

### 4.1 En el Coordinator (Hub)

Verificar que los edges envían heartbeats:

```bash
docker logs coordinator
```

Deberías ver mensajes como:
```
[Coordinator] Heartbeat recibido de edge-rogelio
[Coordinator] Heartbeat recibido de edge-axel
```

### 4.2 En cada Edge

Verificar que se conecta al Coordinator:

```bash
docker logs edge-processor
```

Deberías ver:
```
[Edge] Enviando heartbeat a http://10.10.10.1:3000/heartbeat
[Edge] Reporte enviado exitosamente
```

### 4.3 Verificar sensores

```bash
docker logs iot-sensor-1
docker logs iot-sensor-2
```

Deberías ver datos siendo enviados al Edge.

## Paso 5: Pruebas con tc netem

### Aplicar latencia en un Edge

En la máquina de un Edge:

```bash
# Aplicar 80ms de latencia + 20ms jitter
sudo ./netem/latencia_vpn.sh

# Verificar con ping
ping 10.10.10.1
```

### Verificar en el Coordinator

Los heartbeats de ese Edge deberían llegar con retraso. Si el retraso es mayor a 10 segundos, el Coordinator marcará el Edge como offline.

### Restaurar red normal

```bash
sudo ./netem/baseline_vpn.sh
```

## Comandos Útiles

### Ver estado de servicios

```bash
# En el Hub
docker-compose -f docker/docker-compose.coordinator.yml ps

# En cada Edge
docker-compose -f docker/docker-compose.edge.yml ps
```

### Ver logs

```bash
# Coordinator
docker logs -f coordinator

# Edge
docker logs -f edge-processor

# Sensores
docker logs iot-sensor-1
docker logs iot-sensor-2
```

### Reiniciar servicios

```bash
# En el Hub
docker-compose -f docker/docker-compose.coordinator.yml restart

# En cada Edge
docker-compose -f docker/docker-compose.edge.yml restart
```

### Detener todo

```bash
# En el Hub
docker-compose -f docker/docker-compose.coordinator.yml down

# En cada Edge
docker-compose -f docker/docker-compose.edge.yml down
```

## Troubleshooting

### El Edge no puede conectar al Coordinator

1. Verificar VPN: `ping 10.10.10.1`
2. Verificar Coordinator está corriendo: `docker ps` en el Hub
3. Verificar puerto 3000 está expuesto: `curl http://10.10.10.1:3000`
4. Verificar firewall en el Hub permite puerto 3000

### Los sensores no envían datos

1. Verificar Edge está corriendo: `docker ps`
2. Verificar logs del sensor: `docker logs iot-sensor-1`
3. Probar conexión manual: `curl http://localhost:4000/reading` desde la máquina
4. Verificar que sensor usa `network_mode: host` y `localhost:4000`

### Heartbeats no llegan al Coordinator

1. Verificar logs del Edge: `docker logs edge-processor`
2. Verificar `COORD_HB_URL` apunta a `http://10.10.10.1:3000/heartbeat`
3. Verificar VPN permite tráfico (no hay firewall bloqueando)
4. Probar manualmente desde el Edge:
   ```bash
   curl -X POST http://10.10.10.1:3000/heartbeat \
     -H "Content-Type: application/json" \
     -d '{"node_id":"test","role":"edge","timestamp_ms":12345}'
   ```

### Netem no afecta el tráfico

1. Verificar qué interfaz se está usando:
   ```bash
   ip route | grep default
   sudo wg show
   ```
2. Intentar con interfaz wg0: `sudo ./netem/latencia_vpn.sh wg0`
3. Intentar con interfaz física: `sudo ./netem/latencia_vpn.sh eth0`
4. Verificar reglas aplicadas: `tc qdisc show`

## Checklist Final

- [ ] VPN configurada en todas las máquinas
- [ ] Hub responde en 10.10.10.1:3000
- [ ] Cada Edge puede hacer ping a 10.10.10.1
- [ ] Heartbeats llegan al Coordinator desde todos los Edges
- [ ] Sensores envían datos a sus Edges locales
- [ ] Edges envían reportes al Coordinator
- [ ] Netem aplica latencia en el tráfico VPN
- [ ] Falla de Edge es detectada en < 10 segundos

## Documentación Adicional

- `vpn/README.md` - Configuración detallada de WireGuard
- `docker/README.md` - Opciones de Docker Compose
- `netem/README.md` - Guía de simulación de red
- `docs/CLUSTER_ROADMAP.md` - Roadmap del proyecto

## Soporte

Si encuentras problemas:

1. Revisar logs: `docker logs <contenedor>`
2. Verificar conectividad VPN: `ping 10.10.10.1`
3. Consultar la sección de Troubleshooting arriba
4. Crear un issue en el repositorio con logs y descripción del problema
