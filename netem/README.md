# Guía de Simulación y Validación de Red (tc netem)

Esta carpeta contiene los scripts necesarios para simular condiciones adversas de red en el pipeline IoT/Edge.

## Escenarios Disponibles

1.  `baseline.sh`: Restaura la red a su estado normal (limpia todas las reglas).
2.  `apply_latency.sh`: Inyecta una latencia fija de **140ms**. Útil para pruebas de estrés.
3.  `latencia_iot.sh`: Simula una red IoT real con **80ms de retraso y 20ms de jitter**.

## Validación con iperf3 (OBLIGATORIO)

Para evidenciar que los scripts están funcionando, se debe utilizar la herramienta `iperf3` entre dos contenedores.

### Paso 1: Preparar el Servidor
En el contenedor del **Coordinador**, inicia el servidor de iperf3:
```bash
docker exec -it coordinator iperf3 -s
```

### Paso 2: Aplicar Escenario
En el contenedor del **Edge**, aplica uno de los scripts:
```bash
docker exec -it edge-processor ./netem/latencia_iot.sh
```

### Paso 3: Ejecutar el Cliente y Capturar Evidencia
Desde el contenedor del **Edge**, lanza la prueba hacia el coordinador:
```bash
docker exec -it edge-processor iperf3 -c coordinator
```

### Qué observar en la captura:
*   **Sin script:** La latencia debe ser mínima (< 1ms en red local de Docker).
*   **Con latencia_iot.sh:** El reporte de iperf3 mostrará tiempos de transferencia consistentes con el retraso de 80ms aplicado.
*   **Jitter:** Al usar `ping coordinator`, verás que los tiempos varían entre ráfagas debido al jitter de 20ms.
