#!/bin/bash
# Simula red IoT real con latencia y jitter en la interfaz de red del host
# Uso: sudo ./latencia_vpn.sh [interfaz]
# Ejemplo: sudo ./latencia_vpn.sh eth0

set -e

# Colores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuración de latencia
LATENCY="80ms"
JITTER="20ms"
DISTRIBUTION="normal"

# Detectar interfaz si no se proporciona
if [ -z "$1" ]; then
    echo -e "${YELLOW}No se especificó interfaz. Detectando automáticamente...${NC}"

    # Intentar detectar interfaz principal (conectada a internet)
    if command -v ip &> /dev/null; then
        # Linux con iproute2
        INTERFACE=$(ip route | grep default | head -n1 | awk '{print $5}')
    elif command -v route &> /dev/null; then
        # macOS o Linux antiguo
        INTERFACE=$(route -n get default 2>/dev/null | grep interface | awk '{print $2}')
        if [ -z "$INTERFACE" ]; then
            INTERFACE=$(netstat -rn | grep default | head -n1 | awk '{print $6}')
        fi
    fi

    # Si tenemos wg0 activa, sugerirla
    if ip link show wg0 &> /dev/null; then
        echo -e "${YELLOW}WireGuard detectado (wg0). ¿Deseas usar wg0? (s/n)${NC}"
        read -r response
        if [ "$response" = "s" ] || [ "$response" = "S" ]; then
            INTERFACE="wg0"
        fi
    fi

    if [ -z "$INTERFACE" ]; then
        echo -e "${RED}Error: No se pudo detectar interfaz automáticamente.${NC}"
        echo "Uso: sudo $0 <interfaz>"
        echo "Ejemplo: sudo $0 eth0"
        exit 1
    fi

    echo -e "${GREEN}Interfaz detectada: $INTERFACE${NC}"
else
    INTERFACE="$1"
    echo -e "${GREEN}Usando interfaz especificada: $INTERFACE${NC}"
fi

# Verificar que tc esté disponible
if ! command -v tc &> /dev/null; then
    echo -e "${RED}Error: 'tc' (traffic control) no está instalado.${NC}"
    echo "Instalar en Ubuntu/Debian: sudo apt-get install iproute2"
    echo "Instalar en macOS: tc viene con el sistema (a veces)"
    exit 1
fi

# Verificar permisos de root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: Se requieren permisos de root (sudo).${NC}"
    exit 1
fi

# Verificar que la interfaz existe
if ! ip link show "$INTERFACE" &> /dev/null; then
    echo -e "${RED}Error: La interfaz '$INTERFACE' no existe.${NC}"
    echo "Interfaces disponibles:"
    ip link show | grep -E "^[0-9]+:" | awk -F: '{print $2}' | sed 's/^ //'
    exit 1
fi

echo -e "${YELLOW}Aplicando netem: ${LATENCY} delay ±${JITTER} en ${INTERFACE}${NC}"

# Limpiar reglas existentes (si las hay)
tc qdisc del dev "$INTERFACE" root 2>/dev/null || true

# Aplicar netem
if tc qdisc add dev "$INTERFACE" root netem delay "$LATENCY" "$JITTER" distribution "$DISTRIBUTION"; then
    echo -e "${GREEN}✓ Latencia IoT aplicada exitosamente${NC}"
    echo -e "  Interfaz: $INTERFACE"
    echo -e "  Latencia: $LATENCY"
    echo -e "  Jitter: ±$JITTER"
    echo ""
    echo -e "${YELLOW}Para verificar:${NC}"
    echo "  ping 10.10.10.1  # (desde edge hacia hub)"
    echo ""
    echo -e "${YELLOW}Para remover:${NC}"
    echo "  sudo ./baseline_vpn.sh $INTERFACE"
else
    echo -e "${RED}✗ Error al aplicar netem${NC}"
    exit 1
fi

# Mostrar reglas actuales
echo ""
echo "Reglas actuales de tc:"
tc qdisc show dev "$INTERFACE"
