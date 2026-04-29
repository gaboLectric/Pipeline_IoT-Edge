# Pipeline IoT/Edge - Cluster Distribuido

Proyecto de sistemas avanzados para implementar un pipeline de procesamiento de datos IoT con arquitectura Edge computing distribuida.

**Equipo:** laterceraeslavencida

## Arquitectura del Sistema

El sistema está diseñado como un **cluster distribuido** donde cada integrante del equipo ejecuta nodos Edge y Sensores en su propia máquina, comunicándose a través de VPN WireGuard con un Coordinator centralizado.

```
                    VPN WireGuard 10.10.10.0/24
    ┌─────────────────────────────────────────────────────────────┐
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
    │                                                             │
    └─────────────────────────────────────────────────────────────┘
```

### Roles del Sistema

- **Hub (10.10.10.1)**: Máquina del Coordinator (único en el sistema)
- **Edge Nodes (10.10.10.2+)**: Cada máquina ejecuta 1 Edge + 2+ Sensores
- **Sensores**: Locales a cada máquina, envían datos al Edge de su máquina

## Estructura del Proyecto

```
Pipeline_IoT:Edge/
├── vpn/                    # Configuración WireGuard
│   ├── wg0.conf.template   # Template de configuración VPN
│   └── README.md           # Guía de setup VPN
├── docker/                 # Configuración Docker
│   ├── docker-compose.coordinator.yml   # Solo Hub
│   ├── docker-compose.edge.yml          # Cada máquina
│   ├── docker-compose.yml                 # Legacy (local-only)
│   └── README.md                          # Guía de Docker
├── rust/                  # Código fuente Rust
│   ├── sensor/           # Microservicio sensor
│   ├── edge/             # Microservicio edge processing
│   └── coordinator/      # Microservicio coordinador
├── netem/                # Scripts de emulación de red
│   ├── latencia_vpn.sh   # Para cluster distribuido
│   ├── baseline_vpn.sh
│   └── README.md
├── scripts/              # Scripts de despliegue
│   ├── deploy-coordinator.sh
│   └── deploy-edge.sh
└── docs/                 # Documentación
    ├── CLUSTER_ROADMAP.md
    └── DEPLOYMENT_GUIDE.md
```

## Tecnologías

- **Rust**: Lenguaje principal para microservicios
- **Docker**: Contenerización de servicios
- **WireGuard**: VPN para comunicación segura entre máquinas
- **NetEm**: Emulación de condiciones de red en el cluster
- **Axum**: Framework web para microservicios (Rust)
- **Tokio**: Runtime asíncrono para Rust

## Inicio Rápido - Cluster Distribuido

### 1. Prerrequisitos

- Docker y Docker Compose instalados
- WireGuard instalado (`brew install wireguard-tools` en macOS)
- Clonar este repositorio

### 2. Configurar VPN

Ver guía completa en `vpn/README.md`

**Resumen:**
- Hub: IP `10.10.10.1/24`, escucha en puerto 51820
- Cada Edge: IP única (10.10.10.2, 10.10.10.3, etc.)
- Intercambiar claves públicas entre Hub y Edges

### 3. Desplegar Coordinator (Solo en el Hub)

```bash
./scripts/deploy-coordinator.sh
# o manualmente:
docker-compose -f docker/docker-compose.coordinator.yml up --build
```

### 4. Desplegar Edges (Cada Integrante)

```bash
export COORDINATOR_IP=10.10.10.1
export EDGE_ID=edge-tunombre
./scripts/deploy-edge.sh
# o manualmente:
docker-compose -f docker/docker-compose.edge.yml up --build
```

### 5. Verificar Heartbeats

En el Hub, verificar que llegan heartbeats de todos los edges:
```bash
docker logs coordinator
```

## Despliegue Local (Legacy - Desarrollo)

Para desarrollo local sin VPN, usar el docker-compose legado:

```bash
# Solo para pruebas locales en una máquina
docker-compose -f docker/docker-compose.yml up --build
```

Ver `docker/README.md` para más detalles.

## Simulación de Red (tc netem)

Para probar tolerancia a fallos con red degradada:

```bash
# En cualquier máquina Edge, aplicar latencia
sudo ./netem/latencia_vpn.sh

# Verificar con ping
ping 10.10.10.1

# Restaurar red normal
sudo ./netem/baseline_vpn.sh
```

Ver `netem/README.md` para guía completa.

## Documentación

- `docs/DEPLOYMENT_GUIDE.md` - Guía paso a paso de despliegue
- `docs/CLUSTER_ROADMAP.md` - Roadmap y evolución del proyecto
- `vpn/README.md` - Configuración de VPN WireGuard
- `docker/README.md` - Opciones de Docker Compose
- `netem/README.md` - Guía de simulación de red

## Seguridad

- Las claves privadas WireGuard están excluidas del repositorio
- Usar `vpn/wg0.conf.template` como base para configuraciones
- Solo compartir claves **públicas**, nunca las privadas
- El puerto 51820/UDP debe estar protegido en el Hub
