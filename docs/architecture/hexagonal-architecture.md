# Arquitectura Hexagonal (Ports & Adapters)

## Principio fundamental

El dominio nunca depende de infraestructura. Las dependencias apuntan hacia adentro: desde los adaptadores hacia los puertos, y desde los puertos hacia la aplicación/dominio.

```
  Adaptadores (API, DB, WS)
         │    ↓ dependen de
         ▼
     Puertos (interfaces/traits)
         │    ↓ implementados por
         ▼
   Aplicación (Casos de uso)
         │    ↓ usa
         ▼
     Dominio (Entidades puras)
```

---

## Capas

### 1. Dominio (`backend/src/domain/`)

**Responsabilidad:** Contiene las entidades del negocio y las reglas que las gobiernan. Es puro Rust, sin dependencias externas.

**Contiene:**
- Entidades: `User`, `Message`, `Session`, `Conversation`, `Group`
- Value Objects: `UserId`, `MessageId`, `SessionId`, `ConversationId`, `JwtToken`, `Password`, `Username`, `Email`
- Enums: `MessageStatus`, `ConversationType`, `SessionStatus`
- Traits de dominio: `EncryptionService` (interfaz pura)
- Errores de dominio: `DomainError`

**NO contiene:**
- Dependencias externas (tokio, axum, sqlx, etc.)
- Lógica de infraestructura
- Serialización/deserialización

### 2. Aplicación (`backend/src/application/`)

**Responsabilidad:** Orquesta los casos de uso. Coordina las entidades de dominio y los puertos. No conoce implementaciones concretas.

**Contiene:**
- Casos de uso: `LoginUseCase`, `RegisterUseCase`, `SendMessageUseCase`, `CreateGroupUseCase`, etc.
- DTOs de entrada/salida
- Servicios de aplicación

**Dependencias permitidas:**
- Dominio
- Puertos (interfaces)

**Dependencias prohibidas:**
- Infraestructura
- Adaptadores

### 3. Puertos (`backend/src/ports/`)

**Responsabilidad:** Define interfaces (traits) que el núcleo necesita y que los adaptadores implementan.

**Puertos primarios (driven por la aplicación):**
- `UserRepository`: persistencia de usuarios
- `MessageRepository`: persistencia de mensajes
- `SessionRepository`: persistencia de sesiones
- `ConversationRepository`: persistencia de conversaciones
- `GroupRepository`: persistencia de grupos

**Puertos secundarios (conducidos por infraestructura):**
- `AuthPort`: emisión/validación de JWT
- `PresencePort`: notificación de presencia
- `SignalingPort`: intercambio de ofertas SDP / ICE candidates
- `MessageBrokerPort`: encaminamiento de mensajes

### 4. Adaptadores (`backend/src/adapters/`)

**Responsabilidad:** Implementan los puertos. Contienen todo el código de infraestructura.

**Adaptadores primarios (inbound):**
- `ApiAdapter`: handlers HTTP (Axum) para REST
- `WsAdapter`: handler WebSocket para señalización

**Adaptadores secundarios (outbound):**
- `PostgresUserRepository`: implementación con sqlx
- `PostgresMessageRepository`: implementación con sqlx
- `PostgresSessionRepository`: implementación con sqlx
- `JwtAuthAdapter`: implementación con jsonwebtoken
- `TokioPresenceAdapter`: gestión de presencia in-memory
- `WebRtcSignalingAdapter`: manejo de señalización

---

## Reglas de dependencia estrictas

| Capa | Puede depender de | NO puede depender de |
|------|-------------------|---------------------|
| Dominio | Nada externo | Infraestructura, frameworks |
| Aplicación | Dominio, Puertos | Adaptadores, infraestructura |
| Puertos | Dominio | Adaptadores |
| Adaptadores | Puertos, Dominio | Otros adaptadores (preferiblemente) |

---

## Flujo de una petición

```
HTTP Request
    │
    ▼
ApiAdapter (handler Axum)
    │  deserializa request → DTO
    ▼
LoginUseCase (caso de uso)
    │  orquesta: valida credenciales → genera JWT → registra sesión
    ▼
UserRepository (puerto)
    │  interfaz
    ▼
PostgresUserRepository (adaptador)
    │  sqlx query
    ▼
PostgreSQL
    │
    ▼  (respuesta hacia arriba)
ApiAdapter → HTTP Response
```

---

## Inyección de dependencias

Todas las dependencias se inyectan en el `main.rs` mediante costrucción manual (sin framework DI). El `AppState` contiene las implementaciones concretas de los puertos.

```rust
// Pseudocstructura (sin código real)
struct AppState {
    user_repo: Arc<dyn UserRepository>,
    session_repo: Arc<dyn SessionRepository>,
    presence_port: Arc<dyn PresencePort>,
    signaling_port: Arc<dyn SignalingPort>,
    auth_port: Arc<dyn AuthPort>,
}
```

---

## Beneficios de esta arquitectura

1. **Testabilidad:** El dominio y casos de uso se testean sin infraestructura
2. **Aislamiento:** Cambiar de PostgreSQL a MySQL solo implica un nuevo adaptador
3. **Claridad:** Las responsabilidades están explícitas
4. **Mantenibilidad:** Bajo acoplamiento entre capas
5. **Evolución:** Se pueden añadir nuevos adaptadores sin tocar el núcleo
