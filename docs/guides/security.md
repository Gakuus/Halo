# Seguridad

## JWT (JSON Web Tokens)

### Configuración

| Parámetro | Valor |
|-----------|-------|
| Algoritmo | HS256 (HMAC-SHA256) o ES256 (ECDSA-P256) |
| Access token expira | 15 minutos |
| Refresh token expira | 7 días |
| Secret key | 256 bits mínimo, generada con `openssl rand -hex 32` |
| `jti` | UUID v7 único por sesión |

### Claims del Access Token

```json
{
  "sub": "user_id (uuid)",
  "jti": "session_id (uuid)",
  "exp": 1234567890,
  "iat": 1234567000,
  "iss": "halo-server",
  "username": "string"
}
```

### Claims del Refresh Token

```json
{
  "sub": "user_id (uuid)",
  "jti": "refresh_session_id (uuid)",
  "exp": 1234567890 + 7dias,
  "iat": 1234567000,
  "iss": "halo-server",
  "type": "refresh"
}
```

### Validación

- Verificar firma
- Verificar expiración (rechazar si expirado)
- Verificar que `jti` existe en BD y sesión está activa
- No confiar en claims del cliente sin verificar

### Rotación de secretos

- Rotación manual del secret key (con soporte para múltiples keys en validación)
- Al rotar, todos los tokens existentes se invalidan

---

## Expiración y Refresh

### Flujo

1. Access token expira → cliente usa refresh token
2. Servidor valida refresh token → emite nuevo access token + nuevo refresh token
3. El refresh token anterior se invalida (rotación)
4. Si refresh token expira → el usuario debe hacer login de nuevo

### Renovación automática

- El cliente Tauri renueva automáticamente 5 minutos antes de expirar
- Si falla el refresh, se cierra sesión y se pide login

---

## Hash de Contraseñas

| Parámetro | Valor |
|-----------|-------|
| Algoritmo | argon2id |
| Versión | 1.3 |
| Salt | 16 bytes aleatorios |
| Time cost | 3 |
| Memory cost | 64 MB |
| Parallelism | 4 |
| Hash length | 32 bytes |

### Validación

- Comparación en tiempo constante (no revelar si user existe)
- Mensaje de error genérico: "Invalid credentials" (sin revelar si el usuario existe)

---

## Validaciones

### Lado servidor (siempre)

| Campo | Validación |
|-------|-----------|
| username | 3-30 chars, `[a-zA-Z0-9_]` |
| email | RFC 5322 |
| password | 8-128 chars |
| group name | 1-50 chars, cualquier carácter imprimible |
| message size | Máximo 64 KB (cifrado) |

### Lado cliente (UX)

- Validaciones en tiempo real mientras el usuario escribe
- Las validaciones del servidor son las que cuentan

---

## Rate Limiting

| Endpoint | Límite | Ventana |
|----------|--------|---------|
| POST /auth/login | 5 intentos | 1 minuto |
| POST /auth/register | 3 intentos | 1 hora |
| GET /users/search | 30 requests | 1 minuto |
| WebSocket mensajes | 100 mensajes | 1 minuto |
| POST /auth/refresh | 10 requests | 1 minuto |

### Implementación

- Token bucket por IP y por user_id
- En producción, usar Redis como backend de rate limiter
- En desarrollo, rate limiter opcional o con límites más altos
- Headers de rate limit en respuestas: `X-RateLimit-Remaining`, `X-RateLimit-Reset`

---

## Protección frente a spam

- Rate limiting por usuario (ver arriba)
- Detección de mensajes duplicados (mismo `message_id` rechazado)
- Límite de grupos creados por usuario (máximo 50)
- Límite de miembros por grupo (máximo 50)
- Marcado de cuentas sospechosas (futuro: captcha en registro)

---

## Heartbeat

| Parámetro | Valor |
|-----------|-------|
| Intervalo de envío (cliente) | 15s (P2P), 30s (WS) |
| Timeout sin pong | 2x intervalo |
| Acción al expirar | Marcar sesión como `Reconnecting` |
| Timeout de reconexión | 30 segundos |
| Después de timeout | Marcar sesión como `Expired` |

---

## Reconexión

### Seguridad

- La reconexión requiere nuevo JWT válido
- Si la sesión anterior está en `Reconnecting`, se reactiva
- Si la sesión expiró, se crea nueva (y se revoca la anterior)
- Las conexiones P2P se restablecen con ICE restart

### Protección contra hijacking

- El token de reconexión debe incluir el `session_id` original
- Verificar que la IP no cambió drásticamente (opcional)
- El servidor solo permite reconectar a la misma sesión dentro de la ventana de reconexión

---

## WebSocket

- Conexiones WS autenticadas mediante token JWT en query param
- No almacenar tokens en localStorage (Tauri: keychain)
- Cerrar conexión si el token expira durante la sesión
- No reenviar mensajes a conexiones no autenticadas
- Validar que el remitente de un mensaje de señalización es quien dice ser

---

## Cifrado en tránsito

### Producción
- HTTPS/WSS obligatorio
- Certificados TLS vía Let's Encrypt
- TURN server con TLS/DTLS

### Desarrollo
- HTTP/WS sin TLS (localhost)
- TURN server opcional

---

## E2EE (privacidad de mensajes)

Documentado en detalle en `docs/adr/009-e2ee.md` y `docs/architecture/p2p-mesh-architecture.md`.

Resumen:
- XChaCha20-Poly1305 para cifrado simétrico
- X25519 ECDH para intercambio de claves
- Ed25519 para firmas
- Doble Rachet para PFS
- Claves almacenadas en keychain del SO
- El servidor NO tiene acceso a las claves
- Sin respaldo (claves perdidas = historial perdido)
