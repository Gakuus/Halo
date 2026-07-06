# API REST

## Versionado

La API se versiona mediante prefijo en la URL: `/api/v1/`.

Cuando los cambios sean incompatibles, se incrementa la versión.

---

## Base URL

```
http://localhost:8080/api/v1
```

## Autenticación

La mayoría de endpoints requieren header:

```
Authorization: Bearer <jwt_token>
```

### Respuestas de error comunes

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Descripción legible",
    "details": {}
  }
}
```

| Código HTTP | Error code | Descripción |
|-------------|------------|-------------|
| 400 | `VALIDATION_ERROR` | Datos de entrada inválidos |
| 401 | `UNAUTHORIZED` | Token faltante o inválido |
| 403 | `FORBIDDEN` | Sin permisos |
| 404 | `NOT_FOUND` | Recurso no encontrado |
| 409 | `CONFLICT` | Conflicto (ej: username duplicado) |
| 429 | `RATE_LIMITED` | Demasiadas solicitudes |
| 500 | `INTERNAL_ERROR` | Error interno del servidor |

---

## Endpoints

### Autenticación

#### POST /auth/register

Registrar un nuevo usuario.

**Request:**
```json
{
  "username": "string (3-30 chars, alfanumérico)",
  "email": "string (email válido)",
  "password": "string (min 8 chars)",
  "identity_public_key": "hex string (32 bytes Ed25519)"
}
```

**Response (201):**
```json
{
  "user_id": "uuid",
  "username": "string",
  "token": {
    "access_token": "jwt_string",
    "refresh_token": "jwt_string",
    "expires_in": 900
  }
}
```

**Errores:** 400 (VALIDATION_ERROR), 409 (CONFLICT)

#### POST /auth/login

Iniciar sesión.

**Request:**
```json
{
  "login": "string (username o email)",
  "password": "string"
}
```

**Response (200):**
```json
{
  "user_id": "uuid",
  "username": "string",
  "token": {
    "access_token": "jwt_string",
    "refresh_token": "jwt_string",
    "expires_in": 900
  }
}
```

**Errores:** 401 (UNAUTHORIZED)

#### POST /auth/refresh

Refrescar token de acceso.

**Request:**
```json
{
  "refresh_token": "jwt_string"
}
```

**Response (200):**
```json
{
  "access_token": "jwt_string",
  "refresh_token": "jwt_string",
  "expires_in": 900
}
```

**Errores:** 401 (UNAUTHORIZED)

#### POST /auth/logout

Cerrar sesión. Requiere autenticación.

**Response (200):**
```json
{
  "message": "logged_out"
}
```

---

### Usuarios

Todos requieren autenticación.

#### GET /users/search?q={query}

Buscar usuarios por username o email.

**Response (200):**
```json
{
  "users": [
    {
      "user_id": "uuid",
      "username": "string"
    }
  ]
}
```

#### GET /users/online

Lista de usuarios actualmente en línea.

**Response (200):**
```json
{
  "online_users": [
    {
      "user_id": "uuid",
      "username": "string",
      "online_since": "iso8601"
    }
  ]
}
```

#### GET /users/:id

Obtener perfil público de un usuario.

**Response (200):**
```json
{
  "user_id": "uuid",
  "username": "string",
  "identity_public_key": "hex string",
  "created_at": "iso8601"
}
```

**Errores:** 404 (NOT_FOUND)

#### GET /users/:id/public-key

Obtener clave pública Ed25519 de un usuario (para E2EE).

**Response (200):**
```json
{
  "user_id": "uuid",
  "identity_public_key": "hex string"
}
```

**Errores:** 404 (NOT_FOUND)

---

### Grupos

Todos requieren autenticación.

#### POST /groups

Crear un grupo.

**Request:**
```json
{
  "name": "string (1-50 chars)",
  "member_ids": ["uuid", "uuid"]
}
```

**Response (201):**
```json
{
  "group_id": "uuid",
  "name": "string",
  "owner_id": "uuid",
  "members": [
    {"user_id": "uuid", "role": "owner"},
    {"user_id": "uuid", "role": "member"}
  ],
  "created_at": "iso8601"
}
```

#### GET /groups/:id

Obtener información del grupo.

**Response (200):**
```json
{
  "group_id": "uuid",
  "name": "string",
  "owner_id": "uuid",
  "members": [
    {"user_id": "uuid", "username": "string", "role": "string"}
  ],
  "created_at": "iso8601"
}
```

#### POST /groups/:id/members

Añadir miembros al grupo (solo owner/admin).

**Request:**
```json
{
  "user_ids": ["uuid", "uuid"]
}
```

**Response (200):**
```json
{
  "group_id": "uuid",
  "members_added": 2
}
```

#### DELETE /groups/:id/members/:user_id

Eliminar miembro del grupo (solo owner/admin).

**Response (200):** No content

#### DELETE /groups/:id

Eliminar grupo (solo owner).

**Response (200):** No content

---

## Códigos de error detallados

| Código | HTTP | Causa |
|--------|------|-------|
| `INVALID_CREDENTIALS` | 401 | Usuario/contraseña incorrectos |
| `TOKEN_EXPIRED` | 401 | JWT expirado |
| `TOKEN_INVALID` | 401 | JWT inválido o malformado |
| `SESSION_EXPIRED` | 401 | Sesión revocada o expirada |
| `USERNAME_TAKEN` | 409 | Username ya existe |
| `EMAIL_TAKEN` | 409 | Email ya registrado |
| `USER_NOT_FOUND` | 404 | Usuario no existe |
| `GROUP_NOT_FOUND` | 404 | Grupo no existe |
| `NOT_GROUP_MEMBER` | 403 | No eres miembro del grupo |
| `INSUFFICIENT_PERMISSIONS` | 403 | No tienes permisos |
| `GROUP_FULL` | 400 | Grupo alcanzó límite de miembros |
| `RATE_LIMIT_EXCEEDED` | 429 | Límite de peticiones excedido |
| `VALIDATION_ERROR` | 400 | Datos de entrada inválidos |
