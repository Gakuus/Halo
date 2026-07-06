# ADR-005: PostgreSQL como base de datos

**Estado:** Aceptado

**Contexto:** Necesitamos una base de datos para usuarios, sesiones, grupos y metadatos de conversaciones.

**Decisión:** PostgreSQL con sqlx.

**Opciones consideradas:**
- **SQLite:** Excelente para un solo nodo, no escala horizontalmente
- **PostgreSQL:** Maduro, escalable, con buena concurrencia y extensiones (pgcrypto, UUID)
- **MySQL:** Menor soporte de características avanzadas

**Consecuencias:**
- Escalabilidad horizontal (replicación, clustering)
- Tipos nativos UUID, JSONB
- `sqlx` proporciona verificación de queries en compilación
- Migraciones con sqlx migrate
- Pool de conexiones con bb8 o deadpool
