# ADR-010: Topología Mesh para grupos

**Estado:** Aceptado

**Contexto:** Necesitamos una topología para grupos que minimice la dependencia del servidor y maximice la privacidad.

**Decisión:** Mesh (cada cliente conecta directamente con todos los demás).

**Opciones consideradas:**
- **Client-Server:** Todo el tráfico pasa por el servidor (menos privado, punto central de fallo)
- **Mesh:** Cada par conecta con todos los demás (máxima privacidad, escalabilidad limitada)
- **Relay/SFU:** Servidor reenvía tráfico cifrado (menos conexiones por cliente, más carga en servidor)

**Consecuencias:**
- O(N²) conexiones totales en el grupo
- O(N-1) conexiones por cliente
- Sin punto central de vigilancia
- Latencia mínima (directa entre peers)
- Límite práctico: ~20 miembros por grupo mesh
- Para grupos grandes, evaluar SFU en el futuro
