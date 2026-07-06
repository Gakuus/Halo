# Modelo de Dominio

## Principios

- Las entidades son puras: no contienen lógica de infraestructura
- Los identificadores son Value Objects tipados (no Strings sueltos)
- Las reglas de negocio viven en las entidades
- Las invariantes se validan en la construcción

---

## Entidades

### User (Usuario)

Representa una persona registrada en el sistema.

| Atributo | Tipo | Descripción |
|----------|------|-------------|
| id | `UserId` | Identificador único (UUID v7) |
| username | `Username` | Nombre de usuario único |
| email | `Email` | Email válido |
| password_hash | `PasswordHash` | Hash de la contraseña (argon2) |
| identity_public_key | `Ed25519PublicKey` | Clave pública E2EE del usuario |
| created_at | `Timestamp` | Fecha de registro |
| updated_at | `Timestamp` | Fecha de última modificación |

**Reglas de negocio:**
- El username debe ser único
- El email debe ser único
- El username debe tener entre 3-30 caracteres alfanuméricos
- El email debe tener formato válido
- La password debe tener mínimo 8 caracteres

### Session (Sesión)

Representa una conexión activa de un usuario.

| Atributo | Tipo | Descripción |
|----------|------|-------------|
| id | `SessionId` | Identificador único (UUID v7) |
| user_id | `UserId` | Usuario al que pertenece |
| jwt_id | `JwtId` | ID único del JWT asociado |
| status | `SessionStatus` | `Active`, `Expired`, `Revoked`, `Reconnecting` |
| connected_at | `Timestamp` | Inicio de la sesión |
| last_heartbeat | `Timestamp` | Última señal de vida |
| ip_address | `IpAddress` | Dirección IP de conexión |
| user_agent | `UserAgent` | Información del cliente |

**Reglas de negocio:**
- Un usuario solo puede tener UNA sesión activa
- Al crear nueva sesión, la anterior se revoca
- La sesión expira si no hay heartbeat en 60s

### Message (Mensaje)

Representa un mensaje intercambiado entre usuarios (ya cifrado E2EE).

| Atributo | Tipo | Descripción |
|----------|------|-------------|
| id | `MessageId` | Identificador único (UUID v7) |
| conversation_id | `ConversationId` | Conversación a la que pertenece |
| sender_id | `UserId` | Remitente |
| ciphertext | `Ciphertext` | Contenido cifrado (XChaCha20-Poly1305) |
| iv | `Iv` | Vector de inicialización |
| salt | `Salt` | Salt para derivación de clave |
| signature | `Signature` | Firma Ed25519 del remitente |
| status | `MessageStatus` | `Sent`, `Delivered`, `Read`, `Failed` |
| timestamp | `Timestamp` | Momento de envío (reloj del remitente) |
| reply_to | `Option<MessageId>` | Mensaje al que responde (opcional) |

**Reglas de negocio:**
- El timestamp lo genera el remitente (no confiar en reloj del servidor)
- El mensaje nunca se almacena descifrado en el servidor
- La firma verifica autenticidad e integridad

### Conversation (Conversación)

Representa un canal de comunicación entre dos usuarios (1:1).

| Atributo | Tipo | Descripción |
|----------|------|-------------|
| id | `ConversationId` | Identificador único (UUID v7) |
| participant_a | `UserId` | Primer participante |
| participant_b | `UserId` | Segundo participante |
| created_at | `Timestamp` | Fecha de creación |
| last_message_at | `Option<Timestamp>` | Último mensaje |
| is_active | `bool` | Conversación activa |

**Reglas de negocio:**
- Los participantes se ordenan (lexicográfico) para evitar duplicados
- No puede haber dos conversaciones 1:1 con los mismos participantes
- participants[0] != participants[1]

### Group (Grupo)

Representa un grupo de多名 usuarios.

| Atributo | Tipo | Descripción |
|----------|------|-------------|
| id | `GroupId` | Identificador único (UUID v7) |
| name | `GroupName` | Nombre del grupo |
| owner_id | `UserId` | Creador del grupo |
| members | `Vec<GroupMember>` | Miembros con roles |
| created_at | `Timestamp` | Fecha de creación |
| group_key | `GroupKey` | Clave compartida del grupo (cifrada) |

### GroupMember (Miembro de Grupo)

| Atributo | Tipo | Descripción |
|----------|------|-------------|
| user_id | `UserId` | Usuario miembro |
| role | `GroupRole` | `Owner`, `Admin`, `Member` |
| joined_at | `Timestamp` | Fecha de ingreso |
| encrypted_group_key | `Ciphertext` | Clave de grupo cifrada con clave 1:1 |

---

## Value Objects

| Value Object | Tipo subyacente | Validación |
|-------------|----------------|------------|
| `UserId` | `Uuid` (v7) | Generado por el sistema |
| `Username` | `String` | 3-30 chars, `[a-zA-Z0-9_]` |
| `Email` | `String` | Formato email RFC 5322 |
| `PasswordHash` | `String` | Hash argon2id |
| `JwtId` | `Uuid` | Generado por el sistema |
| `JwtToken` | `String` | Token JWT firmado |
| `SessionId` | `Uuid` | Generado por el sistema |
| `MessageId` | `Uuid` | Generado por el sistema |
| `ConversationId` | `Uuid` | Generado por el sistema |
| `GroupId` | `Uuid` | Generado por el sistema |
| `Ciphertext` | `Vec<u8>` | Datos cifrados |
| `Iv` | `[u8; 24]` | 24 bytes para XChaCha20 |
| `Salt` | `[u8; 32]` | 32 bytes |
| `Signature` | `[u8; 64]` | Firma Ed25519 |
| `GroupName` | `String` | 1-50 chars |
| `Timestamp` | `i64` | Unix timestamp en milisegundos |
| `IpAddress` | `std::net::IpAddr` | IPv4 o IPv6 |
| `UserAgent` | `String` | Sin validación estricta |

---

## Enums

### SessionStatus
```rust
enum SessionStatus {
    Active,
    Expired,
    Revoked,
    Reconnecting,
}
```

### MessageStatus
```rust
enum MessageStatus {
    Sent,
    Delivered,
    Read,
    Failed,
}
```

### ConversationType
```rust
enum ConversationType {
    Direct,
    Group,
}
```

### GroupRole
```rust
enum GroupRole {
    Owner,
    Admin,
    Member,
}
```

---

## Domain Errors

```rust
enum DomainError {
    // Autenticación
    InvalidCredentials,
    UserNotFound,
    SessionExpired,
    SessionRevoked,
    InvalidToken,

    // Validación
    InvalidUsername(String),
    InvalidEmail(String),
    WeakPassword,
    DuplicateUsername,
    DuplicateEmail,

    // Mensajería
    ConversationNotFound,
    NotParticipant,
    MessageTooLarge,
    InvalidSignature,

    // Grupos
    GroupNotFound,
    NotMember,
    NotOwner,
    MemberAlreadyExists,
    GroupFull,
    InsufficientPermissions,

    // General
    Unauthorized,
    Internal(String),
    NotFound(String),
}
```

---

## Relaciones entre entidades

```
User ───1:N─── Session
  │
  ├───M:N─── Conversation (participants)
  │     │
  │     └───1:N─── Message
  │
  └───1:N─── Group (owner)
        │
        └───M:N─── Member (a través de GroupMember)
              │
              └───1:N─── Message
```

---

## Invariantes del modelo

1. Un `Message` siempre pertenece a una `Conversation` o `Group`
2. Un `Message` solo puede ser creado por un participante de la conversación
3. `User.identity_public_key` debe ser una clave Ed25519 válida
4. La `Session` expira si `now - last_heartbeat > MAX_HEARTBEAT_INTERVAL`
5. Un `Group` debe tener al menos 2 miembros (owner + 1)
6. Un `Username` es inmutable después de creado
7. `Conversation.participant_a < Conversation.participant_b` (lexicográfico)
