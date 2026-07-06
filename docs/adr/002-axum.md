# ADR-002: Axum como framework HTTP

**Estado:** Aceptado

**Contexto:** Necesitamos un framework HTTP/WebSocket para el servidor de señalización y API REST.

**Decisión:** Axum.

**Opciones consideradas:**
- **Actix-web:** Excelente rendimiento, pero con modelo de concurrencia diferente a Tokio
- **Axum:** Construido sobre Tokio, Tower, Hyper. Misma base que el runtime asíncrono. Mejor integración con el ecosistema Tokio.
- **Warp:** Bueno pero con sintaxis menos clara para proyectos grandes

**Consecuencias:**
- Integración natural con Tokio
- Middleware vía Tower (composabilidad)
- WebSocket integrado (sin dependencias extra)
- Comunidad activa y creciendo
