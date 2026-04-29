#!/bin/bash
# Script de despliegue para el Nodo Coordinator (Hub)
# Ejecutar en la máquina designada como Hub (10.10.10.1)

set -e

# Colores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Despliegue: Coordinator (Hub)         ${NC}"
echo -e "${BLUE}  IP esperada: 10.10.10.1               ${NC}"
echo -e "${BLUE}========================================${NC}"
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

# Verificar que WireGuard está configurado
if ! ip link show wg0 &> /dev/null; then
    echo -e "${YELLOW}Advertencia: Interfaz wg0 no encontrada${NC}"
    echo "Asegúrate de que WireGuard esté configurado y activo."
    echo "Ver: vpn/README.md"
    echo ""
    read -p "¿Continuar de todos modos? (s/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Ss]$ ]]; then
        exit 1
    fi
fi

# Verificar IP
CURRENT_IP=$(ip addr show wg0 2>/dev/null | grep "inet " | awk '{print $2}' | cut -d/ -f1)
if [ "$CURRENT_IP" != "10.10.10.1" ]; then
    echo -e "${YELLOW}Advertencia: La IP de wg0 es ${CURRENT_IP}, se esperaba 10.10.10.1${NC}"
    echo ""
    read -p "¿Continuar de todos modos? (s/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Ss]$ ]]; then
        exit 1
    fi
fi

# Detener contenedores existentes si los hay
echo -e "${YELLOW}Deteniendo contenedores existentes...${NC}"
docker-compose -f docker/docker-compose.coordinator.yml down 2>/dev/null || true

# Construir y levantar
echo -e "${YELLOW}Construyendo e iniciando Coordinator...${NC}"
cd docker
docker-compose -f docker-compose.coordinator.yml up --build -d

echo ""
echo -e "${GREEN}✓ Coordinator desplegado exitosamente${NC}"
echo ""
echo "Estado del servicio:"
docker-compose -f docker-compose.coordinator.yml ps

echo ""
echo -e "${BLUE}Logs (Ctrl+C para salir):${NC}"
docker-compose -f docker-compose.coordinator.yml logs -f
