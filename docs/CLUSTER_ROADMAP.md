# Roadmap: Evolución a Cluster Distribuido

> Este documento describe la transición desde la **prueba local de concepto** hacia la **arquitectura distribuida real** del sistema IoT/Edge.

## Contexto

La implementación actual funcionó como **prueba de concepto local** (todos los servicios en una máquina). Ahora se procede a la fase de **cluster distribuido** donde cada integrante del equipo ejecuta nodos edge y sensores en sus propias máquinas, comunicándose a través de VPN.

---

## Phase 1: Separación de Arquitectura Docker (Prep)

**Objetivo:** Dividir el monolito Docker en configuraciones independientes para cada rol.

**Cambios:**
- Crear `docker-compose.coordinator.yml` (solo para el Hub - 10.10.10.1)
- Crear `docker-compose.edge.yml` (para cada máquina del equipo)
- Actualizar variables de entorno para IPs VPN en lugar de nombres de servicio Docker
- Documentar qué equipo ejecuta qué

**Archivos afectados:**
- `docker/docker-compose.coordinator.yml` (nuevo)
- `docker/docker-compose.edge.yml` (nuevo)
- `docker/docker-compose.yml` (deprecado/marcado como local-only)

---

## Phase 2: Corrección de Heartbeats (Arquitectura)

**Objetivo:** Alinear el sistema con la arquitectura donde solo los Edges envían heartbeats.

**Razonamiento:** En el cluster distribuido, el Coordinator necesita monitorear la salud de los Edges remotos (en diferentes máquinas físicas). Los sensores son efímeros y locales a cada máquina, por lo que no necesitan heartbeat directo.

**Cambios:**
- Remover envío de heartbeats desde `rust/sensor/src/main.rs`
- Actualizar `rust/common/src/lib.rs` - marcar `role` en Heartbeat como opcional o remover "sensor"
- Verificar que `rust/coordinator/src/main.rs` solo trackee edges (ignorar heartbeats de sensores si llegan)

**Archivos afectados:**
- `rust/sensor/src/main.rs`
- `rust/common/src/lib.rs` (opcional)
- `docker-compose.edge.yml` (remover COORD_HB_URL del sensor)

---

## Phase 3: Configuración VPN WireGuard (Infraestructura)

**Objetivo:** Actualizar la configuración de VPN para la red 10.10.10.0/24 del cluster real.

**Cambios:**
- Actualizar `vpn/wg0.conf.template` con red 10.10.10.0/24
- Documentar asignación de IPs por equipo:
  - Hub (Coordinator): 10.10.10.1
  - Miembro 1: 10.10.10.2
  - Miembro 2: 10.10.10.3
  - etc.
- Crear instrucciones de setup VPN por sistema operativo

**Archivos afectados:**
- `vpn/wg0.conf.template`
- `vpn/README.md` (nuevo o actualizar existente)

---

## Phase 4: URLs de Comunicación Distribuida (Conectividad)

**Objetivo:** Actualizar todas las URLs de comunicación para usar IPs VPN en lugar de nombres Docker.

**Cambios:**
- Sensor → Edge: Cambiar de `http://edge:4000` a `http://localhost:4000` (misma máquina)
- Edge → Coordinator: Cambiar de `http://coordinator:3000` a `http://10.10.10.1:3000`
- Edge Heartbeat → Coordinator: `http://10.10.10.1:3000/heartbeat`

**Archivos afectados:**
- `docker/docker-compose.edge.yml` (variables de entorno)
- `docker/docker-compose.coordinator.yml` (puertos expuestos)
- Documentación de despliegue

---

## Phase 5: Netem para Red Distribuida (Testing)

**Objetivo:** Adaptar los scripts de netem para afectar el tráfico entre máquinas (VPN) en lugar de tráfico Docker local.

**Cambios:**
- Crear documentación sobre dónde aplicar netem:
  - Opción A: Interfaz física (eth0/en0) - afecta todo el tráfico
  - Opción B: Interfaz WireGuard (wg0) - afecta solo VPN
- Crear scripts `netem/latencia_vpn.sh` con detección de interfaz
- Documentar consideraciones: aplicar netem en la máquina que inicia la comunicación

**Archivos afectados:**
- `netem/README.md` (actualizar)
- `netem/latencia_vpn.sh` (nuevo)

---

## Phase 6: Scripts de Despliegue y Documentación (Ops)

**Objetivo:** Facilitar el despliegue distribuido con scripts y guías claras.

**Cambios:**
- Crear `scripts/deploy-coordinator.sh` (para el Hub)
- Crear `scripts/deploy-edge.sh` (para cada miembro)
- Actualizar README principal con arquitectura distribuida
- Crear `docs/DEPLOYMENT_GUIDE.md` con pasos por rol

**Archivos afectados:**
- `scripts/` (nuevo directorio)
- `README.md`
- `docs/DEPLOYMENT_GUIDE.md` (nuevo)

---

## Checklist de Verificación del Cluster

- [ ] VPN levantada entre todas las máquinas (`ping 10.10.10.1` funciona desde edges)
- [ ] Coordinator responde en 10.10.10.1:3000
- [ ] Cada Edge envía heartbeats y son recibidos
- [ ] Edges envían reportes de anomalías al Coordinator
- [ ] Sensores locales envían datos a su Edge local
- [ ] Netem aplicado en interfaz correcta simula latencia entre máquinas
- [ ] Falla de un Edge es detectada por Coordinator en < 10 segundos

---

## Notas para el Equipo

1. **Quién es el Hub:** Designar una máquina como Hub (Coordinator). Idealmente la más estable.
2. **WireGuard:** Cada miembro necesita su propia configuración de peer en el Hub.
3. **Firewall:** Asegurar que el puerto 51820/UDP esté abierto en el Hub.
4. **Docker:** Todos necesitan Docker y Docker Compose instalados.
5. **Red local:** Los contenedores en la misma máquina se comunican por localhost/bridge, no por VPN.
