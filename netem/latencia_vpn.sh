#!/bin/bash
# Escenario Latencia IoT - delay 3000ms jitter 1000ms 25% (Documento B)
# Uso: sudo ./latencia_vpn.sh [interfaz]

YELLOW='\033[1;33m'
NC='\033[0m'

INTERFACE=${1:-wg0}

if ! ip link show "$INTERFACE" &> /dev/null; then
    echo "Error: Interfaz '$INTERFACE' no existe"
    exit 1
fi

tc qdisc del dev "$INTERFACE" root 2>/dev/null || true
tc qdisc add dev "$INTERFACE" root netem delay 3000ms 1000ms 25%

echo -e "${YELLOW}✓ Latencia 3000ms ±1000ms (25% corr) aplicada en $INTERFACE${NC}"
echo "  ping 10.10.10.1  # Verificar"
