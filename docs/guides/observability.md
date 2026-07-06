# Observabilidad

## Principios

1. Todo evento importante debe dejar traza
2. Los errores deben ser rastreables hasta su origen
3. El sistema debe exponer su estado interno para diagnóstico

---

## Tracing

Usamos el crate `tracing` con `tracing-subscriber` para la capa de observabilidad.

### Instrumentación

```rust
// Toda función asíncrona importante se instrumenta
#[tracing::instrument(skip(self))]
async fn execute(&self, cmd: LoginCommand) -> Result<LoginResponse, DomainError> {
    // ...
}
```

### Fields estándar

| Field | Cuándo |
|-------|--------|
| `user_id` | En operaciones autenticadas |
| `session_id` | En operaciones de sesión |
| `conversation_id` | En operaciones de mensajería |
| `error` | En errores |
| `duration_ms` | En operaciones lentas |
| `peer_id` | En comunicaciones WebRTC |

### Niveles

| Evento | Nivel |
|--------|-------|
| Inicio/fin de request HTTP | `info` |
| Conexión/desconexión WS | `info` |
| Login/registro | `info` |
| Error de negocio esperado | `warn` |
| Error de infraestructura | `error` |
| Señalización WebRTC | `debug` |
| Heartbeat | `trace` |

---

## Logging

### Formato (desarrollo)

```
2026-07-06T10:30:00.123Z  INFO halo::auth: User logged in user_id=abc-123 session_id=xyz-789
2026-07-06T10:30:01.456Z ERROR halo::ws: Connection error user_id=abc-123 error="session expired"
```

### Formato (producción)

JSON estructurado para mejor integración con sistemas de logging:

```json
{
  "timestamp": "2026-07-06T10:30:00.123Z",
  "level": "INFO",
  "target": "halo::auth",
  "message": "User logged in",
  "fields": {
    "user_id": "abc-123",
    "session_id": "xyz-789"
  }
}
```

### Outputs

| Entorno | Output |
|---------|--------|
| Desarrollo | stderr con formato humano (tracing-subscriber) |
| Producción | stdout JSON (tracing-bunyan-formatter o similar) |
| Tests | Output silenciado, capturar con `tracing-test` |

---

## Errores

### Estrategia

- Todos los errores que cruzan capas se loggean con `error!` o `warn!`
- Los errores esperados (validación, credenciales inválidas) se loggean como `warn`
- Los errores inesperados (BD caída, pánico) se loggean como `error`
- Los pánicos se capturan con `catch_unwind` en los handlers

### Error tracking (futuro)

- Integración con Sentry o similar para producción
- Capturar pánicos y errores no manejados
- Contexto: user_id, session_id, request_id

---

## Métricas

### Endpoint de métricas

```
GET /metrics
```

Expuesto en puerto separado (ej: 9090) para no mezclar con API pública.

### Métricas a exponer

| Métrica | Tipo | Descripción |
|---------|------|-------------|
| `http_requests_total` | Counter | Total de requests HTTP por endpoint y status |
| `http_request_duration_ms` | Histogram | Duración de requests HTTP |
| `ws_connections_active` | Gauge | Conexiones WebSocket activas |
| `ws_connections_total` | Counter | Conexiones WebSocket totales |
| `users_online` | Gauge | Usuarios con sesión activa |
| `messages_relayed_total` | Counter | Mensajes de señalización reenviados |
| `db_queries_total` | Counter | Queries a BD por tipo |
| `db_query_duration_ms` | Histogram | Duración de queries |
| `turn_credentials_issued` | Counter | Credenciales TURN emitidas |
| `webrtc_offers_sent` | Counter | Ofertas WebRTC enviadas |
| `webrtc_answers_sent` | Counter | Respuestas WebRTC enviadas |

### Implementación

- Usar `metrics` crate + `metrics-exporter-prometheus`
- Las métricas se registran en los adaptadores
- Dashboard en Grafana (futuro)

---

## Health Checks

### Endpoint

```
GET /health
```

**Response (200):**
```json
{
  "status": "ok",
  "version": "1.0.0",
  "uptime_seconds": 3600,
  "checks": {
    "database": {
      "status": "ok",
      "latency_ms": 2
    },
    "database_migrations": {
      "status": "ok",
      "version": 5
    }
  }
}
```

### Checks implementados

| Check | Descripción |
|-------|-------------|
| `database` | Conexión a PostgreSQL (SELECT 1) |
| `database_migrations` | Migraciones al día |

### Futuros checks

- `turn_server` → latencia con servidor TURN
- `ws_connections` → sanity check de conexiones activas

---

## Request ID

Cada request HTTP/WS recibe un `request_id` (UUID v7) que se propaga en:

- Logs
- Tracing spans
- Respuestas de error
- Headers de respuesta (`X-Request-Id`)
