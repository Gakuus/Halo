# ADR-003: Tokio como runtime asíncrono

**Estado:** Aceptado

**Contexto:** El sistema necesita manejar cientos de conexiones WebSocket y señalización concurrente.

**Decisión:** Tokio.

**Opciones consideradas:**
- **async-std:** Menos ecosistema y adopción
- **Smol:** Ligero pero menos soporte
- **Tokio:** Runtime más maduro, mayor adopción, mejor ecosistema

**Consecuencias:**
- Scheduler work-stealing para alta concurrencia
- Ecosistema amplio (Tower, Hyper, Axum, sqlx)
- Soporte para I/O asíncrona, timers, canales
