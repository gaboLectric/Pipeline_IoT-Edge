# Pipeline IoT/Edge

Proyecto de sistemas avanzados para implementar un pipeline de procesamiento de datos IoT con arquitectura Edge computing.

**Equipo:** laterceraeslavencida

## Estructura del Proyecto

```
Pipeline_IoT:Edge/
├── vpn/                    # Plantillas de configuración WireGuard
├── docker/                 # Configuración Docker y Dockerfiles
│   ├── sensor/            # Dockerfile para nodo sensor
│   ├── edge/              # Dockerfile para nodo edge
│   └── coordinator/       # Dockerfile para coordinador
├── rust/                  # Código fuente Rust
│   ├── sensor/           # Microservicio sensor
│   ├── edge/             # Microservicio edge processing
│   └── coordinator/      # Microservicio coordinador
├── netem/                # Scripts de emulación de red
├── docs/                 # Documentación y diagramas
└── docker-compose.yml    # Orquestación de servicios
```

## Tecnologías

- **Rust**: Lenguaje principal para microservicios
- **Docker**: Contenerización de servicios
- **WireGuard**: VPN para comunicación segura
- **NetEm**: Emulación de condiciones de red

## Inicio Rápido

```bash
# Construir y levantar servicios
docker-compose up --build

# Detener servicios
docker-compose down
```

## Seguridad

- Las claves privadas y certificados están excluidos del repositorio
- Use `wg0.conf.template` como base para configuraciones WireGuard
- Variables de entorno en archivo `.env` (no incluido en repo)
