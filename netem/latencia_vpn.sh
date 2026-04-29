#!/bin/bash
# Escenario Latencia IoT - delay 80ms jitter 20ms (Documento B)
# Uso: sudo ./latencia_vpn.sh [interfaz]

YELLOW='\033[1;33m'
NC='\033[0m'

INTERFACE=${1:-wg0}

if ! ip link show "$INTERFACE" &> /dev/null; then
    echo "Error: Interfaz '$INTERFACE' no existe"
    exit 1
fi

tc qdisc del dev "$INTERFACE" root 2>/dev/null || true
tc qdisc add dev "$INTERFACE" root netem delay 80ms 20ms distribution normal

echo -e "${YELLOW}✓ Latencia 80ms ±20ms aplicada en $INTERFACE${NC}"
echo "  ping 10.10.10.1  # Verificar"
