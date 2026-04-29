#!/bin/bash
# Baseline - Restaura red normal. Uso: sudo ./baseline_vpn.sh [interfaz]

GREEN='\033[0;32m'
NC='\033[0m'

INTERFACE=${1:-wg0}

if ! ip link show "$INTERFACE" &> /dev/null; then
    INTERFACE=$(ip route | grep default | head -n1 | awk '{print $5}')
fi

tc qdisc del dev "$INTERFACE" root 2>/dev/null || true
echo -e "${GREEN}✓ Red restaurada en $INTERFACE${NC}"
