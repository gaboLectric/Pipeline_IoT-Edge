# Post-Mortem Técnico - Pipeline IoT Edge

## 1. Falla: Latencia no simulada correctamente al inicio
- Causa raíz: configuración incompleta de tc netem en nodos edge.
- Impacto: el sistema no reflejaba condiciones reales de red distribuida.
- Solución: se integraron scripts estandarizados en /netem con parámetros fijos de latencia y pérdida.

---

## 2. Falla: Desincronización entre sensores y coordinator
- Causa raíz: ausencia de manejo de timestamps consistentes.
- Impacto: datos duplicados o fuera de orden.
- Solución: se agregó lógica de ordenamiento y validación temporal en coordinator.

---

## 3. Falla: Variables de entorno inconsistentes en despliegue Docker
- Causa raíz: falta de centralización del archivo .env.
- Impacto: fallos intermitentes en containers.
- Solución: se implementó carga automática de variables desde docker/.env.
