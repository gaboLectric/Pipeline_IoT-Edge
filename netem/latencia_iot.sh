#!/bin/bash

# Escenario: Simulación de Latencia de Red IoT (Inestable)
# Objetivo: Aplicar un retraso base de 80ms con una variación (jitter) de 20ms.

# ANSI Color Codes
BLUE='\033[0;34m'
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

INTERFACE="eth0"
DELAY="80ms"
JITTER="20ms"

printf "${BLUE}[INFO]${NC} Configurando latencia IoT: $DELAY (+/- $JITTER jitter) en $INTERFACE...\n"

# Eliminar reglas previas
tc qdisc del dev $INTERFACE root 2>/dev/null

# Aplicar la regla con jitter
tc qdisc add dev $INTERFACE root netem delay $DELAY $JITTER

if [ $? -eq 0 ]; then
    printf "${GREEN}[SUCCESS]${NC} Escenario IoT aplicado: ${DELAY} con ${JITTER} de jitter.\n"
    printf "${BLUE}[NOTE]${NC} Usa 'ping' o 'iperf3' para observar la fluctuación en los tiempos de respuesta.\n"
else
    printf "${RED}[ERROR]${NC} No se pudo aplicar la configuración. Revisa privilegios NET_ADMIN.\n"
fi
