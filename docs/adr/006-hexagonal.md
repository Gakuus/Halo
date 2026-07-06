# ADR-006: Arquitectura Hexagonal (Ports & Adapters)

**Estado:** Aceptado

**Contexto:** Necesitamos una arquitectura que aisle el dominio de la infraestructura, permita testear lógica de negocio sin base de datos y facilite cambios tecnológicos.

**Decisión:** Hexagonal Architecture.

**Opciones consideradas:**
- **Arquitectura en capas tradicional:** Mayor acoplamiento, dominio depende de infraestructura
- **Clean Architecture:** Muy similar a Hexagonal, mayor formalismo
- **Hexagonal:** Equilibrio entre formalismo y pragmatismo

**Consecuencias:**
- El dominio es puramente Rust, testeable sin infraestructura
- Cambiar de BD requiere solo un nuevo adaptador
- Mayor boilerplate inicial (interfaces/implementaciones)
- Dependencias apuntan hacia adentro
