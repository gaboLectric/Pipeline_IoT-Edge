#!/bin/bash
# Enlace Limitado - rate 512kbit delay 50ms (Documento B). Uso: sudo ./enlace_limitado.sh [interfaz]

BLUE='\033[0;34m'
NC='\033[0m'

INTERFACE=${1:-wg0}

if ! ip link show "$INTERFACE" &> /dev/null; then
    INTERFACE=$(ip route | grep default | head -n1 | awk '{print $5}')
fi

tc qdisc del dev "$INTERFACE" root 2>/dev/null || true
tc qdisc add dev "$INTERFACE" root netem rate 512kbit delay 50ms

echo -e "${BLUE}✓ Enlace limitado 512kbit+50ms en $INTERFACE${NC}"
echo "  iperf3 -c 10.10.10.1  # Verificar throughput ~512kbps"
