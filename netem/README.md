# Guía de Simulación y Validación de Red (tc netem)

Esta carpeta contiene los scripts necesarios para simular condiciones adversas de red en el pipeline IoT/Edge.

## ⚠️ Diferencia: Prueba Local vs Cluster Distribuido

### Prueba Local (Legacy)
- Scripts aplican netem **dentro de contenedores Docker**
- Solo afecta tráfico entre contenedores en la misma máquina
- Usar scripts originales: `baseline.sh`, `apply_latency.sh`, `latencia_iot.sh`

### Cluster Distribuido (Actual)
- Scripts aplican netem en la **interfaz de red del host**
- Afecta tráfico entre máquinas a través de la VPN
- Usar scripts específicos: `latencia_vpn.sh`, `baseline_vpn.sh`

## Escenarios Disponibles - Cluster Distribuido

Para el despliegue real con VPN, usar estos scripts en las máquinas físicas:

### 1. `latencia_vpn.sh` - Simular Red IoT Real
Aplica **80ms de latencia + 20ms de jitter** en la interfaz de red.

```bash
# Detectar interfaz automáticamente
sudo ./netem/latencia_vpn.sh

# O especificar interfaz manualmente
sudo ./netem/latencia_vpn.sh eth0
```

### 2. `baseline_vpn.sh` - Restaurar Red Normal
Limpia todas las reglas de netem de la interfaz:

```bash
sudo ./netem/baseline_vpn.sh
# o con interfaz específica:
sudo ./netem/baseline_vpn.sh eth0
```

## ¿Dónde Aplicar netem en el Cluster?

### Opción A: Interfaz Física (eth0/en0)
**Afecta**: Todo el tráfico de red de la máquina.

```bash
# Linux
sudo tc qdisc add dev eth0 root netem delay 80ms 20ms

# macOS (usa pfctl, diferente sintaxis)
```

**Pros:**
- Afecta todo el tráfico incluyendo VPN
- Simula condiciones reales de red del host

**Contras:**
- Afecta otras aplicaciones
- Puede afectar SSH si se aplica en la máquina remota

### Opción B: Interfaz WireGuard (wg0) ⭐ Recomendado
**Afecta**: Solo tráfico que pasa por la VPN.

```bash
sudo tc qdisc add dev wg0 root netem delay 80ms 20ms
```

**Pros:**
- Solo afecta tráfico del cluster
- No interfiere con otras aplicaciones
- Más realista para el proyecto

**Contras:**
- NetEm debe aplicarse después de que wg0 esté activa
- Algunos sistemas aplican reglas en orden diferente

### ⚠️ Consideración Importante (del Documento E)

Según el post-mortem del proyecto, aplicar netem sobre `wg0` a veces **no afecta** el tráfico inter-contenedor porque los paquetes ya se desencapsularon antes de llegar a netem.

**Recomendación del proyecto:**
1. Primero intentar con interfaz física (`eth0`/`en0`)
2. Si no funciona, intentar con `wg0`
3. Documentar qué interfaz funcionó para la entrega

## Validación con iperf3 (Cluster Distribuido)

Para evidenciar que netem está funcionando en el cluster:

### Paso 1: Preparar el Servidor (Hub)
En la máquina del **Coordinator (10.10.10.1)**, iniciar iperf3:

```bash
iperf3 -s
```

### Paso 2: Aplicar Escenario (Edge)
En una máquina **Edge** (ej: 10.10.10.2), aplicar netem:

```bash
# Verificar que la VPN funciona
ping 10.10.10.1

# Aplicar latencia
sudo ./netem/latencia_vpn.sh
```

### Paso 3: Ejecutar Prueba
Desde la misma máquina Edge, lanzar iperf3 hacia el Coordinator:

```bash
iperf3 -c 10.10.10.1
```

### Qué observar:
- **Sin netem**: Latencia mínima (< 5ms en red local de VPN)
- **Con netem**: Latencia ~80ms con variación (jitter)

### Paso 4: Verificar con ping

```bash
ping 10.10.10.1
```

Deberías ver tiempos de respuesta alrededor de 80ms con ±20ms de variación.
