#!/bin/bash
# Script de despliegue para Nodos Edge + Sensores
# Ejecutar en cada máquina del equipo (no en el Hub)

set -e

# Colores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Despliegue: Edge + Sensores           ${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Cargar variables desde .env si existe en la carpeta docker
if [ -f "docker/.env" ]; then
    echo -e "${GREEN}✓ Cargando configuración desde docker/.env${NC}"
    export $(grep -v '^#' docker/.env | xargs)
fi

# Variables requeridas
if [ -z "$EDGE_ID" ]; then
    echo -e "${YELLOW}Variable EDGE_ID no definida.${NC}"
    read -p "Ingresa un identificador para este Edge (ej: edge-rogelio): " EDGE_ID
    export EDGE_ID
fi

if [ -z "$COORDINATOR_IP" ]; then
    echo -e "${YELLOW}Variable COORDINATOR_IP no definida.${NC}"
    read -p "Ingresa la IP del Coordinator (default: 10.10.10.1): " COORDINATOR_IP
    COORDINATOR_IP=${COORDINATOR_IP:-10.10.10.1}
    export COORDINATOR_IP
fi

echo -e "${BLUE}Configuración:${NC}"
echo "  EDGE_ID: $EDGE_ID"
echo "  COORDINATOR_IP: $COORDINATOR_IP"
echo ""

# Verificar prerequisitos
echo -e "${YELLOW}Verificando prerequisitos...${NC}"

if ! command -v docker &> /dev/null; then
    echo -e "${RED}Error: Docker no está instalado${NC}"
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}Error: Docker Compose no está instalado${NC}"
    exit 1
fi

# Verificar conectividad VPN
echo -e "${YELLOW}Verificando conectividad VPN con $COORDINATOR_IP...${NC}"
if ! ping -c 1 -W 3 "$COORDINATOR_IP" &> /dev/null; then
    echo -e "${RED}Error: No se puede alcanzar el Coordinator en $COORDINATOR_IP${NC}"
    echo "Verifica que:"
    echo "  1. WireGuard está activo (sudo wg-quick up wg0)"
    echo "  2. La VPN está configurada correctamente"
    echo "  3. El Coordinator está corriendo en el Hub"
    exit 1
fi

echo -e "${GREEN}✓ Conectividad VPN verificada${NC}"

# Verificar que el Coordinator responde
echo -e "${YELLOW}Verificando que Coordinator responde en puerto 3000...${NC}"
if ! curl -s "http://$COORDINATOR_IP:3000" &> /dev/null; then
    echo -e "${YELLOW}Advertencia: El Coordinator no responde en puerto 3000${NC}"
    echo "Es posible que aún esté iniciándose."
    echo ""
    read -p "¿Continuar de todos modos? (s/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Ss]$ ]]; then
        exit 1
    fi
else
    echo -e "${GREEN}✓ Coordinator responde correctamente${NC}"
fi

# Detener contenedores existentes si los hay
echo -e "${YELLOW}Deteniendo contenedores existentes...${NC}"
docker-compose -f docker/docker-compose.edge.yml down 2>/dev/null || true

# Construir y levantar
echo -e "${YELLOW}Construyendo e iniciando Edge + Sensores...${NC}"
cd docker
# Docker Compose cargará automáticamente el archivo .env si existe
docker-compose -f docker-compose.edge.yml up --build -d

echo ""
echo -e "${GREEN}✓ Edge y Sensores desplegados exitosamente${NC}"
echo ""
echo "Estado de los servicios:"
docker-compose -f docker-compose.edge.yml ps

echo ""
echo -e "${BLUE}Próximos pasos:${NC}"
echo "  1. Verificar logs del Edge: docker logs edge-processor"
echo "  2. Verificar heartbeats llegan al Coordinator"
echo "  3. Aplicar netem para pruebas: sudo ./netem/latencia_vpn.sh"
echo ""
echo -e "${BLUE}Logs del Edge (Ctrl+C para salir):${NC}"
docker-compose -f docker-compose.edge.yml logs -f edge
