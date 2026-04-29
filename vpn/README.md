# Guía de Configuración VPN (WireGuard)

Esta guía describe cómo configurar la VPN WireGuard para el cluster distribuido IoT/Edge.

## Topología de Red

```
                    VPN 10.10.10.0/24
    ┌─────────────────────────────────────────────────┐
    │                                                 │
    │  ┌──────────────┐         ┌──────────────┐    │
    │  │   Hub        │◄───────►│  Edge 1      │    │
    │  │  10.10.10.1  │  VPN    │  10.10.10.2  │    │
    │  │ [Coordinator]│         │ [Edge Node]  │    │
    │  └──────────────┘         └──────────────┘    │
    │         ▲                      ▲              │
    │         │                      │              │
    │         └──────────┬─────────┘              │
    │                    ▼                          │
    │              ┌──────────────┐               │
    │              │  Edge 2      │               │
    │              │  10.10.10.3  │               │
    │              │ [Edge Node]  │               │
    │              └──────────────┘               │
    │                                                 │
    └─────────────────────────────────────────────────┘
```

**Roles:**
- **Hub (10.10.10.1)**: Máquina del Coordinator. Escucha conexiones VPN.
- **Edges (10.10.10.2+)**: Máquinas de los integrantes. Se conectan al Hub.

## Requisitos Previos

- WireGuard instalado en todas las máquinas:
  - **macOS**: `brew install wireguard-tools`
  - **Linux**: `sudo apt install wireguard`
  - **Windows**: Descargar de [wireguard.com](https://www.wireguard.com/install/)

- Puerto 51820/UDP abierto en el Hub (firewall/router).

## Paso 1: Preparar el Hub (Coordinator)

El Hub es la máquina que ejecutará el Coordinator. Debe tener IP pública accesible o estar en la misma red que los edges.

### 1.1 Generar claves del Hub

```bash
cd vpn
wg genkey | tee hub_private.key | wg pubkey > hub_public.key
```

### 1.2 Crear archivo de configuración del Hub

Crear `/etc/wireguard/wg0.conf` (Linux/macOS) o importar en la app (Windows):

```ini
[Interface]
PrivateKey = <contenido de hub_private.key>
Address = 10.10.10.1/24
ListenPort = 51820

# Peer: Edge 1 (Máquina de Rogelio)
[Peer]
PublicKey = <clave pública del edge 1>
AllowedIPs = 10.10.10.2/32

# Peer: Edge 2 (Máquina de Axel)
[Peer]
PublicKey = <clave pública del edge 2>
AllowedIPs = 10.10.10.3/32

# Peer: Edge 3 (Máquina de Yael)
[Peer]
PublicKey = <clave pública del edge 3>
AllowedIPs = 10.10.10.4/32

# Agregar más según integrantes...
```

### 1.3 Iniciar WireGuard en el Hub

```bash
# Linux/macOS
sudo wg-quick up wg0

# Verificar estado
sudo wg show

# Verificar IP
ip addr show wg0  # Linux
ifconfig wg0      # macOS
```

### 1.4 Compartir clave pública del Hub

Enviar el contenido de `hub_public.key` a todos los integrantes.

## Paso 2: Preparar Edge Nodes (Integrantes)

Cada integrante sigue estos pasos en su máquina.

### 2.1 Generar claves del Edge

```bash
cd vpn
wg genkey | tee edge_private.key | wg pubkey > edge_public.key
```

### 2.2 Crear archivo de configuración del Edge

Reemplazar:
- `<clave_privada_edge>`: contenido de `edge_private.key`
- `<clave_publica_hub>`: la clave que compartió el administrador del Hub
- `<ip_publica_hub>`: IP pública del Hub (o IP en la red local si todos están en la misma red)
- `X`: número asignado al integrante (2, 3, 4...)

```ini
[Interface]
PrivateKey = <clave_privada_edge>
Address = 10.10.10.X/32

[Peer]
PublicKey = <clave_publica_hub>
AllowedIPs = 10.10.10.0/24
Endpoint = <ip_publica_hub>:51820
PersistentKeepalive = 25
```

### 2.3 Iniciar WireGuard en el Edge

```bash
# Linux/macOS
sudo wg-quick up wg0

# Verificar conexión
ping 10.10.10.1
```

### 2.4 Compartir clave pública del Edge

Enviar el contenido de `edge_public.key` al administrador del Hub para que agregue el peer.

## Paso 3: Completar Configuración del Hub

El administrador del Hub debe:

1. Recibir las claves públicas de todos los edges
2. Agregar cada peer a `/etc/wireguard/wg0.conf`:

```ini
[Peer]
PublicKey = <clave_publica_edge_N>
AllowedIPs = 10.10.10.N/32
```

3. Recargar WireGuard:

```bash
sudo wg-quick down wg0
sudo wg-quick up wg0
```

## Verificación de Conectividad

### Desde el Hub (10.10.10.1):

```bash
# Ver todos los peers conectados
sudo wg show

# Ping a cada edge
ping 10.10.10.2
ping 10.10.10.3
ping 10.10.10.4
```

### Desde cada Edge:

```bash
# Verificar conexión al Hub
ping 10.10.10.1

# Verificar que otros edges son visibles (opcional)
ping 10.10.10.3  # desde edge 2
```

## Asignación de IPs Sugerida

| Integrante | Rol | IP VPN | IP Pública/Endpoint |
|------------|-----|--------|---------------------|
| Rogelio | Hub + Coordinator | 10.10.10.1 | IP pública del Hub |
| Rogelio | Edge + Sensores | 10.10.10.2 | (misma máquina que Hub) |
| Axel | Edge + Sensores | 10.10.10.3 | IP pública de Axel |
| Yael | Edge + Sensores | 10.10.10.4 | IP pública de Yael |
| Gabo | Edge + Sensores | 10.10.10.5 | IP pública de Gabo |

## Troubleshooting

### No se puede conectar al Hub

1. Verificar que el puerto 51820 esté abierto en el firewall del Hub
2. Verificar que WireGuard esté corriendo en el Hub: `sudo wg show`
3. Verificar que la clave pública del edge esté correctamente agregada al Hub
4. Verificar que la IP pública/endpoint del Hub sea correcta

### Ping no funciona

1. Verificar que las claves públicas sean correctas (un carácter mal copiado = fallo)
2. Verificar las AllowedIPs en ambos lados
3. Verificar que no haya firewall bloqueando ICMP

### Conexión intermitente

1. Asegurar que `PersistentKeepalive = 25` esté configurado en los edges
2. Verificar que el Hub tenga IP pública estática (o usar DDNS)

### Dos edges no se ven entre sí

Por defecto, los edges solo se conectan al Hub. Para que se vean entre sí (mesh), se necesita:
- Cada edge tenga peers de otros edges (más complejo)
- O: Configurar IP forwarding en el Hub (más fácil)

Para el proyecto, **no es necesario** que los edges se vean entre sí, solo que vean al Coordinator en 10.10.10.1.

## Comandos Útiles

```bash
# Ver estado de WireGuard
sudo wg show

# Ver logs (Linux con systemd)
sudo journalctl -u wg-quick@wg0 -f

# Detener WireGuard
sudo wg-quick down wg0

# Reiniciar WireGuard
sudo wg-quick down wg0 && sudo wg-quick up wg0

# Ver interfaces de red
ip addr  # Linux
ifconfig # macOS/Linux alternativo
```

## Seguridad

- **Nunca compartir claves privadas** (archivos `*_private.key`)
- Solo compartir claves públicas (archivos `*_public.key`)
- Las claves privadas deben permanecer en sus respectivas máquinas
- El archivo `wg0.conf` contiene la clave privada - mantenerlo seguro
