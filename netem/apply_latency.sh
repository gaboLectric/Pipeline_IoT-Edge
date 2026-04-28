#!/bin/bash

# Escenario: Simulación de Latencia de Red Estable
# Objetivo: Incrementar la latencia en 140ms para observar el impacto en las métricas del Coordinador.

# NOTA: Este script debe ejecutarse dentro de un contenedor Docker con privilegios NET_ADMIN
# o en la máquina host de Linux.

# ANSI Color Codes
BLUE='\033[0;34m'
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

INTERFACE="eth0"
DELAY="140ms"

printf "${BLUE}[INFO]${NC} Aplicando latencia de $DELAY a la interfaz $INTERFACE...\n"

# Eliminar reglas previas si existen para evitar errores
tc qdisc del dev $INTERFACE root 2>/dev/null

# Aplicar la nueva regla de latencia
tc qdisc add dev $INTERFACE root netem delay $DELAY

if [ $? -eq 0 ]; then
    printf "${GREEN}[SUCCESS]${NC} Latencia de $DELAY aplicada correctamente.\n"
    printf "${BLUE}[NOTE]${NC} Revisa el endpoint /status del Coordinador para validar el impacto.\n"
else
    printf "${RED}[ERROR]${NC} Fallo al aplicar latencia. Verifica privilegios (NET_ADMIN) e interfaz.\n"
fi
