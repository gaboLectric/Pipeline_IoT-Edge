#!/bin/bash

# Script de Limpieza: Restaurar condiciones normales de red

# ANSI Color Codes
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

INTERFACE="eth0"

printf "${BLUE}[INFO]${NC} Eliminando todas las reglas de control de tráfico en $INTERFACE...\n"

tc qdisc del dev $INTERFACE root 2>/dev/null

if [ $? -eq 0 ]; then
    printf "${GREEN}[SUCCESS]${NC} Red restaurada a condiciones normales (0ms latencia extra).\n"
else
    printf "${YELLOW}[WARN]${NC} La interfaz ya estaba limpia o no fue encontrada.\n"
fi
