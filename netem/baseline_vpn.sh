#!/bin/bash
# Restaura la interfaz de red a su estado normal (sin netem)
# Uso: sudo ./baseline_vpn.sh [interfaz]
# Ejemplo: sudo ./baseline_vpn.sh eth0

set -e

# Colores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Detectar interfaz si no se proporciona
if [ -z "$1" ]; then
    echo -e "${YELLOW}No se especificó interfaz. Detectando automáticamente...${NC}"

    if command -v ip &> /dev/null; then
        INTERFACE=$(ip route | grep default | head -n1 | awk '{print $5}')
    elif command -v route &> /dev/null; then
        INTERFACE=$(route -n get default 2>/dev/null | grep interface | awk '{print $2}')
        if [ -z "$INTERFACE" ]; then
            INTERFACE=$(netstat -rn | grep default | head -n1 | awk '{print $6}')
        fi
    fi

    if [ -z "$INTERFACE" ]; then
        echo -e "${RED}Error: No se pudo detectar interfaz automáticamente.${NC}"
        echo "Uso: sudo $0 <interfaz>"
        exit 1
    fi

    echo -e "${GREEN}Interfaz detectada: $INTERFACE${NC}"
else
    INTERFACE="$1"
    echo -e "${GREEN}Usando interfaz especificada: $INTERFACE${NC}"
fi

# Verificar permisos de root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: Se requieren permisos de root (sudo).${NC}"
    exit 1
fi

# Verificar que tc esté disponible
if ! command -v tc &> /dev/null; then
    echo -e "${RED}Advertencia: 'tc' no está instalado.${NC}"
    exit 1
fi

echo -e "${YELLOW}Limpiando reglas de netem de ${INTERFACE}...${NC}"

# Intentar eliminar reglas de netem
if tc qdisc del dev "$INTERFACE" root 2>/dev/null; then
    echo -e "${GREEN}✓ Reglas de netem removidas exitosamente${NC}"
else
    echo -e "${YELLOW}No había reglas de netem activas en ${INTERFACE}${NC}"
fi

# Mostrar estado actual
echo ""
echo "Estado actual de tc en ${INTERFACE}:"
tc qdisc show dev "$INTERFACE" || echo "  (no hay reglas configuradas)"

echo ""
echo -e "${GREEN}La red ha sido restaurada a su estado normal.${NC}"
