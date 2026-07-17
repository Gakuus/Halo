# ADR 012: Diseño de la API REST

- **Estado:** Aceptado
- **Fecha:** 2026-07-16
- **Driver:** @equipo

## Contexto

Al implementar la capa HTTP (Fase 7 del roadmap), es necesario definir decisiones de diseño transversales: estructura de rutas, manejo de estado, formato de respuestas, autenticación, paginación, etc.

## Decisiones

### 1. Estructura del router

Se usa `Router::new().nest()` por recurso, con módulos separados:

```
src/adapters/api/
  routes/
    mod.rs    → fn create_router() -> Router
    auth.rs   → /api/v1/auth/*
    users.rs  → /api/v1/users/*
    groups.rs → /api/v1/groups/*
    ws.rs     → /api/v1/ws (WebSocket)
  middleware/
    mod.rs
    auth.rs   → JWT validation layer
  error.rs    → ApiError + IntoResponse
  response.rs → helpers de respuesta
```

### 2. Estado compartido (App State)

```rust
struct AppState {
    db_pool: PgPool,
    user_repo: PostgresUserRepository,
    session_repo: PostgresSessionRepository,
    conversation_repo: PostgresConversationRepository,
    group_repo: PostgresGroupRepository,
    jwt_secret: String,
    jwt_expiration: i64,
    jwt_refresh_expiration: i64,
}
```

Se pasa con `Router::with_state()`. Los repositorios son tipos concretos (no trait objects) por simplicidad. Para tests se mockea a nivel de servicio/caso de uso.

### 3. Manejo de errores HTTP

Se define `ApiError` con `StatusCode`, `error_code` y `message`, e implementa `IntoResponse`. Un `From<DomainError>` convierte errores de dominio a API errors:

```rust
struct ApiError {
    status: StatusCode,
    code: ErrorCode,      // enum serializado como string
    message: String,
}
```

Los handlers devuelven `Result<Json<T>, ApiError>`.

### 4. Validación de requests

Se usa un extractor custom `Json<ValidatedBody<T>>` que deserializa y valida en un paso, devolviendo `ApiError::Validation` si falla.

### 5. Autenticación JWT

Se implementa como un **Layer** de Axum (`tower::Layer`) que:
1. Extrae el header `Authorization: Bearer <token>`
2. Valida el JWT (firma, expiración, `jti` activo en BD)
3. Inserta `Claims` en `Request::extensions()`
4. Los handlers acceden via `Extension<Claims>`

Endpoints públicos llevan `#[utoipa::skip]` semántico (sin layer). Alternativa: aplicar el layer por grupo de rutas.

### 6. Formato de respuestas JSON

**Sin envoltorio unificado.** Respuestas directas según el spec:

```
// Éxito: response body directo
{ "user_id": "...", "username": "...", "token": {...} }

// Error: objeto error
{ "error": { "code": "INVALID_CREDENTIALS", "message": "..." } }
```

### 7. Paginación

**Cursor-based**, no page/limit.

```
GET /users/search?cursor=<last_id>&limit=20
→ {
    "data": [...],
    "next_cursor": "uuid",
    "has_more": true
  }
```

Usa `cursor` (UUID del último elemento) + `limit` (default 20, max 100).

### 8. Graceful shutdown

Se capturan `SIGINT` (ctrl+c) y `SIGTERM` con `tokio::signal`. Al recibir señal:
1. Dejar de aceptar nuevas conexiones
2. Esperar hasta 30 segundos a que las conexiones activas terminen
3. Forzar cierre después del timeout

### 9. CORS

Configurable desde archivo de configuración. Por defecto:

| Origen | Propósito |
|--------|-----------|
| `http://localhost:5173` | Vite dev server (frontend Tauri) |
| `tauri://localhost` | Tauri webview en desarrollo |
| `https://app.halo.app` | Producción |

Métodos: `GET, POST, PATCH, DELETE`
Headers: `Authorization, Content-Type, X-Halo-WS-Token`

### 10. Rate limiting

Implementación **in-memory con token bucket** usando `tokio` (sin Redis por ahora). Límites por endpoint (ver `docs/guides/security.md`). Migrar a Redis cuando haya múltiples instancias.

## Consecuencias

### Positivas
- Código modular y fácil de navegar
- Error handling centralizado y consistente
- Paginación eficiente (evita `OFFSET`)
- Fácil escalar a Redis rate limiting después
- Sin overhead de serialización de envoltorio

### Negativas
- Sin envoltorio unificado: el cliente debe checkear `error` vs campos directos
- Cursor-based es menos intuitivo que page/limit para algunos clientes
- Rate limiting in-memory se pierde al reiniciar el servidor
- Layer de auth no es obvio para handlers nuevos (hay que saber que existe)

## Referencias

- [ADR 004: JWT](004-jwt.md)
- [ADR 002: Axum](002-axum.md)
- [REST API spec](../api/rest-api.md)
