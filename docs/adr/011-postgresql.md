# ADR-011: sqlx para acceso a base de datos

**Estado:** Aceptado

**Contexto:** Necesitamos un cliente PostgreSQL con verificación de queries en tiempo de compilación y soporte asíncrono nativo con Tokio.

**Decisión:** sqlx.

**Opciones consideradas:**
- **diesel:** ORM completo, pero más verboso, sin verificación en compilación, synchronously
- **sqlx:** Queries verificadas en compilación, async nativo, sin ORM overhead
- **tokio-postgres:** Directo pero sin verificación en compilación

**Consecuencias:**
- Queries verificadas contra BD real en compilación
- Async nativo con Tokio
- Migraciones integradas (sqlx migrate)
- Mapeo manual de filas a structs (más control, más boilerplate)
