#!/bin/bash
# Falla de Nodo Edge - Mata contenedor edge-processor (Documento B)
# Uso: ./falla_edge.sh [-d segundos]

MAGENTA='\033[0;35m'
NC='\033[0m'

DELAY=0
while getopts "d:" opt; do
    case $opt in
        d) DELAY=$OPTARG ;;
        *) echo "Uso: $0 [-d segundos]" && exit 1 ;;
    esac
done

if [ "$DELAY" -gt 0 ]; then
    echo "Esperando ${DELAY}s antes de matar edge..."
    sleep "$DELAY"
fi

if ! docker ps | grep -q "edge-processor"; then
    echo "Error: edge-processor no está corriendo"
    exit 1
fi

docker kill edge-processor
echo -e "${MAGENTA}✓ edge-processor detenido. Timeout detection: ~10s en Coordinator${NC}"
