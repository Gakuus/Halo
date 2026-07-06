# ADR-001: Rust como lenguaje principal

**Estado:** Aceptado

**Contexto:** Necesitamos un lenguaje para el servidor central y el frontend nativo que ofrezca rendimiento, seguridad de memoria y buen soporte para concurrencia.

**Decisión:** Rust.

**Opciones consideradas:**
- **Go:** Bueno para redes pero sin E2EE nativo, sin gestión de memoria explícita
- **Node.js:** Rendimiento inferior, sin tipado fuerte nativo
- **Rust:** Rendimiento nativo, seguridad de memoria en compilación, ecosistema asíncrono maduro (Tokio)

**Consecuencias:**
- Curva de aprendizaje más alta
- Compilación más lenta
- Seguridad de memoria en tiempo de compilación
- Ecosistema excelente para redes y criptografía
- Mismo lenguaje para backend y frontend (Tauri)
