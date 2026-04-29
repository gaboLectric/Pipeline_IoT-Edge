#!/bin/bash
# Pérdida de Paquetes - loss 8% (Documento B). Uso: sudo ./perdida_paquetes.sh [interfaz]

RED='\033[0;31m'
NC='\033[0m'

INTERFACE=${1:-wg0}

if ! ip link show "$INTERFACE" &> /dev/null; then
    INTERFACE=$(ip route | grep default | head -n1 | awk '{print $5}')
fi

tc qdisc del dev "$INTERFACE" root 2>/dev/null || true
tc qdisc add dev "$INTERFACE" root netem loss 8%

echo -e "${RED}✓ Pérdida 8% aplicada en $INTERFACE${NC}"
echo "  ping -c 10 10.10.10.1  # Verificar (~8% loss)"
