# Guía de Simulación de Red Degradada - tc netem (Documento B)

## Componente Técnico Central del Proyecto 2

Todos los equipos deben implementar la simulación de red degradada con `tc netem`, independientemente de su solución de VPN.

## 6.1 Escenarios Mínimos Requeridos

| Escenario | Parámetros tc netem | Script | Objetivo de Medición |
|-----------|-------------------|--------|---------------------|
| **1. Baseline** | Sin degradación | `baseline_vpn.sh` | Throughput y latencia de referencia |
| **2. Latencia IoT** | `delay 80ms jitter 20ms` | `latencia_vpn.sh` | Impacto en frecuencia de publicación de sensores |
| **3. Pérdida de Paquetes** | `loss 8%` | `perdida_paquetes.sh` | Comportamiento ante pérdidas |
| **4. Enlace Limitado** | `rate 512kbit delay 50ms` | `enlace_limitado.sh` | Throughput máximo, cuellos de botella |
| **5. Falla de Nodo** | Matar contenedor edge | `falla_edge.sh` | Tiempo de detección y recuperación |

---

## Escenarios Disponibles

### 1. Baseline - Red Normal
Restaura la interfaz a su estado normal sin degradación.

```bash
sudo ./netem/baseline_vpn.sh [interfaz]
# Ejemplo:
sudo ./netem/baseline_vpn.sh wg0
```

**Salida esperada:**
```
✓ Interfaz wg0 restaurada a estado normal
```

---

### 2. Latencia IoT - Simular Red Celular/WiFi Congestionada
Aplica **80ms de latencia + 20ms de jitter** (Documento B).

```bash
sudo ./netem/latencia_vpn.sh [interfaz]
# Ejemplo:
sudo ./netem/latencia_vpn.sh wg0
```

**Comando tc equivalente:**
```bash
sudo tc qdisc add dev wg0 root netem delay 80ms 20ms distribution normal
```

**Salida esperada (ping):**
```
64 bytes from 10.10.10.1: icmp_seq=1 ttl=64 time=82.3 ms
64 bytes from 10.10.10.1: icmp_seq=2 ttl=64 time=95.7 ms
64 bytes from 10.10.10.1: icmp_seq=3 ttl=64 time=78.1 ms
# ~80ms ± 20ms de variación
```

---

### 3. Pérdida de Paquetes - Simular Interferencias
Aplica **8% de pérdida de paquetes** (Documento B).

```bash
sudo ./netem/perdida_paquetes.sh [interfaz]
# Ejemplo:
sudo ./netem/perdida_paquetes.sh wg0
```

**Comando tc equivalente:**
```bash
sudo tc qdisc add dev wg0 root netem loss 8%
```

**Salida esperada (ping):**
```
64 bytes from 10.10.10.1: icmp_seq=1 ttl=64 time=5.2 ms
64 bytes from 10.10.10.1: icmp_seq=2 ttl=64 time=5.1 ms
# Pérdida esperada: ~8% de los paquetes
--- 10.10.10.1 ping statistics ---
10 packets transmitted, 9 received, 10% packet loss
```

---

### 4. Enlace Limitado - Simular 3G/4G Limitado
Limita el ancho de banda a **512 kbps con 50ms de delay** (Documento B).

```bash
sudo ./netem/enlace_limitado.sh [interfaz]
# Ejemplo:
sudo ./netem/enlace_limitado.sh wg0
```

**Comando tc equivalente:**
```bash
sudo tc qdisc add dev wg0 root netem rate 512kbit delay 50ms
```

**Salida esperada (iperf3):**
```
[ ID] Interval           Transfer     Bitrate
[  5]   0.00-10.00  sec   640 KBytes   512 Kbits/sec
# Throughput limitado a ~512 kbps
```

---

### 5. Falla de Nodo Edge - Simulación de Caída
Mata el contenedor `edge-processor` para simular falla abrupta.

```bash
# Falla inmediata:
./netem/falla_edge.sh

# Falla después de N segundos (para observar detección):
./netem/falla_edge.sh -d 30
```

**Comando equivalente:**
```bash
docker kill edge-processor
```

**Salida esperada en Coordinator:**
```
[ALERTA] Falla detectada en edge-gabo. Sin señal por 10302 ms.
[RECUPERACIÓN] El nodo edge-gabo ha vuelto a conectarse.
```

---

## 6.2 Consideraciones Técnicas Obligatorias (Documento B)

### 6.2.1 Interfaz de Aplicación

Las reglas tc netem deben aplicarse sobre la interfaz correcta:

#### Opción A: Interfaz Física (eth0/ens33)
```bash
# Detectar interfaz física
ip route | grep default
# Salida: default via 192.168.1.1 dev ens33

# Aplicar netem
sudo tc qdisc add dev ens33 root netem delay 80ms 20ms
```

**Justificación:** Afecta todo el tráfico de red, incluyendo la VPN. Simula condiciones reales de red del host físico.

#### Opción B: Interfaz WireGuard (wg0) ⭐ Recomendado
```bash
# Aplicar netem solo en la VPN
sudo tc qdisc add dev wg0 root netem delay 80ms 20ms
```

**Justificación:** Solo afecta tráfico del cluster a través de la VPN. No interfiere con otras aplicaciones ni conexiones SSH.

** Nota:** NetEm debe aplicarse **después** de que `wg0` esté activa (`sudo wg-quick up wg0`).

### 6.2.2 MTU de WireGuard y Fragmentación

WireGuard usa **MTU de 1420 bytes** por defecto (vs 1500 bytes de Ethernet estándar).

**Verificar MTU:**
```bash
ip addr show wg0
# mtu 1420
```

**Prueba de fragmentación:**
```bash
# Ping con DF (Don't Fragment) y tamaño grande
ping -M do -s 1400 10.10.10.1

# Si falla, probar con tamaño menor
ping -M do -s 1300 10.10.10.1
```

**Consideración:** Con pérdida de paquetes (`loss 8%`), paquetes grandes pueden fragmentarse o perderse. El sistema debe manejar retransmisiones.

### 6.2.3 Comandos Exactos para Activar/Desactivar

#### Activar Escenarios

```bash
# 1. Baseline (restaurar)
sudo tc qdisc del dev wg0 root 2>/dev/null || true

# 2. Latencia IoT
sudo tc qdisc add dev wg0 root netem delay 80ms 20ms

# 3. Pérdida de Paquetes
sudo tc qdisc add dev wg0 root netem loss 8%

# 4. Enlace Limitado
sudo tc qdisc add dev wg0 root netem rate 512kbit delay 50ms

# 5. Falla de Nodo (simulación)
docker kill edge-processor
```

#### Desactivar (Volver a Baseline)

```bash
# Eliminar reglas netem
sudo tc qdisc del dev wg0 root 2>/dev/null || true

# O usar el script
sudo ./netem/baseline_vpn.sh wg0
```

---

## Validación con iperf3 (Obligatorio)

Cada escenario debe validarse con `iperf3` antes de ejecutar el sistema distribuido.

### Preparación

**En el Coordinator (Hub - 10.10.10.1):**
```bash
iperf3 -s
```

**En el Edge (10.10.10.2):**
```bash
# Instalar iperf3 si es necesario
sudo apt install iperf3

# Verificar conectividad VPN
ping 10.10.10.1
```

### Escenario 1: Baseline (Sin degradación)

```bash
# Limpiar reglas
sudo ./netem/baseline_vpn.sh wg0

# Medir
iperf3 -c 10.10.10.1 -t 10

# Captura esperada:
# [ ID] Interval           Transfer     Bitrate
# [  5]   0.00-10.00  sec  1.10 GBytes   943 Mbits/sec  (valores altos)
```

### Escenario 2: Latencia IoT

```bash
sudo ./netem/latencia_vpn.sh wg0
iperf3 -c 10.10.10.1 -t 10

# Captura esperada:
# [  5]   0.00-10.00  sec  950 MBytes   797 Mbits/sec  (throughput similar, RTT ~80ms)
```

### Escenario 3: Pérdida de Paquetes

```bash
sudo ./netem/perdida_paquetes.sh wg0
iperf3 -c 10.10.10.1 -t 10

# Captura esperada:
# [  5]   0.00-10.00  sec  450 MBytes   377 Mbits/sec  (throughput reducido por retransmisiones)
```

### Escenario 4: Enlace Limitado

```bash
sudo ./netem/enlace_limitado.sh wg0
iperf3 -c 10.10.10.1 -t 30

# Captura esperada:
# [  5]   0.00-30.00  sec  1.88 MBytes   512 Kbits/sec  (limitado exacto)
```

---

## Resumen de Scripts

| Script | Propósito | Uso |
|--------|-----------|-----|
| `baseline_vpn.sh` | Restaurar red normal | `sudo ./baseline_vpn.sh wg0` |
| `latencia_vpn.sh` | 80ms + 20ms jitter | `sudo ./latencia_vpn.sh wg0` |
| `perdida_paquetes.sh` | 8% packet loss | `sudo ./perdida_paquetes.sh wg0` |
| `enlace_limitado.sh` | 512kbit + 50ms | `sudo ./enlace_limitado.sh wg0` |
| `falla_edge.sh` | Simular caída de edge | `./falla_edge.sh [-d segundos]` |

---

## Verificación Rápida

```bash
# 1. Verificar estado actual
tc qdisc show dev wg0

# 2. Limpiar todas las reglas
sudo tc qdisc del dev wg0 root

# 3. Aplicar escenario
sudo ./netem/latencia_vpn.sh wg0

# 4. Validar con ping
ping 10.10.10.1

# 5. Validar con iperf3 (requiere servidor)
iperf3 -c 10.10.10.1
```

