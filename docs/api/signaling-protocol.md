# Protocolo de Señalización WebRTC

## Conexión WebSocket

```
Endpoint: ws://localhost:8080/api/v1/ws
Query param: ?token=<jwt_token>
```

### Establecimiento

1. Cliente conecta al WebSocket con JWT en query param
2. Servidor valida JWT
3. Si válido: asocia la conexión a la sesión del usuario
4. Si inválido: cierra conexión con código 4001

### Cierre

- El servidor envía `{ "type": "error", "code": "...", "message": "..." }` antes de cerrar
- Códigos de cierre WS: 4001 (no autenticado), 4002 (sesión revocada), 4003 (timeout)

---

## Formato de mensajes

Todos los mensajes siguen este formato JSON:

```json
{
  "type": "string",
  "payload": {},
  "timestamp": "i64 (unix ms)"
}
```

---

## Eventos de Presencia

### Servidor → Cliente

#### `welcome`
Enviado al conectar exitosamente.

```json
{
  "type": "welcome",
  "payload": {
    "user_id": "uuid",
    "session_id": "uuid",
    "online_users": [
      {"user_id": "uuid", "username": "string"}
    ]
  },
  "timestamp": 1234567890
}
```

#### `user_online`
Un contacto se conectó.

```json
{
  "type": "user_online",
  "payload": {
    "user_id": "uuid",
    "username": "string"
  },
  "timestamp": 1234567890
}
```

#### `user_offline`
Un contacto se desconectó.

```json
{
  "type": "user_offline",
  "payload": {
    "user_id": "uuid",
    "username": "string"
  },
  "timestamp": 1234567890
}
```

---

## Eventos de Señalización WebRTC

### Cliente → Servidor

#### `signal_offer`
Iniciar llamada/chat con otro usuario.

```json
{
  "type": "signal_offer",
  "payload": {
    "target_user_id": "uuid",
    "sdp": "string (SDP offer)",
    "conversation_type": "direct"
  },
  "timestamp": 1234567890
}
```

#### `signal_answer`
Responder a una oferta.

```json
{
  "type": "signal_answer",
  "payload": {
    "target_user_id": "uuid",
    "sdp": "string (SDP answer)"
  },
  "timestamp": 1234567890
}
```

#### `signal_ice_candidate`
Enviar ICE candidate durante el establecimiento.

```json
{
  "type": "signal_ice_candidate",
  "payload": {
    "target_user_id": "uuid",
    "candidate": "string (ICE candidate)",
    "sdp_mid": "string",
    "sdp_mline_index": 0
  },
  "timestamp": 1234567890
}
```

#### `signal_accept`
Aceptar una invitación de chat.

```json
{
  "type": "signal_accept",
  "payload": {
    "target_user_id": "uuid"
  },
  "timestamp": 1234567890
}
```

#### `signal_reject`
Rechazar una invitación.

```json
{
  "type": "signal_reject",
  "payload": {
    "target_user_id": "uuid",
    "reason": "busy | declined"
  },
  "timestamp": 1234567890
}
```

#### `signal_end`
Finalizar una conexión WebRTC.

```json
{
  "type": "signal_end",
  "payload": {
    "target_user_id": "uuid"
  },
  "timestamp": 1234567890
}
```

### Servidor → Cliente

#### `incoming_offer`
Nueva oferta de otro usuario.

```json
{
  "type": "incoming_offer",
  "payload": {
    "sender_user_id": "uuid",
    "sender_username": "string",
    "sdp": "string (SDP offer)",
    "conversation_type": "direct"
  },
  "timestamp": 1234567890
}
```

#### `incoming_answer`
Respuesta a la oferta enviada.

```json
{
  "type": "incoming_answer",
  "payload": {
    "sender_user_id": "uuid",
    "sdp": "string (SDP answer)"
  },
  "timestamp": 1234567890
}
```

#### `incoming_ice_candidate`
ICE candidate del otro peer.

```json
{
  "type": "incoming_ice_candidate",
  "payload": {
    "sender_user_id": "uuid",
    "candidate": "string",
    "sdp_mid": "string",
    "sdp_mline_index": 0
  },
  "timestamp": 1234567890
}
```

#### `call_ringing`
El destinatario está siendo notificado.

```json
{
  "type": "call_ringing",
  "payload": {
    "target_user_id": "uuid"
  },
  "timestamp": 1234567890
}
```

#### `call_accepted`
El destinatario aceptó.

```json
{
  "type": "call_accepted",
  "payload": {
    "target_user_id": "uuid",
    "sdp": "string (SDP answer)"
  },
  "timestamp": 1234567890
}
```

#### `call_rejected`
El destinatario rechazó.

```json
{
  "type": "call_rejected",
  "payload": {
    "target_user_id": "uuid",
    "reason": "busy | declined"
  },
  "timestamp": 1234567890
}
```

#### `peer_disconnected`
Un peer se desconectó (limpieza).

```json
{
  "type": "peer_disconnected",
  "payload": {
    "peer_user_id": "uuid"
  },
  "timestamp": 1234567890
}
```

---

## Eventos de Heartbeat

### Cliente → Servidor (cada 30s)

```json
{
  "type": "ping",
  "timestamp": 1234567890
}
```

### Servidor → Cliente

```json
{
  "type": "pong",
  "timestamp": 1234567890
}
```

---

## Errores

### Servidor → Cliente

```json
{
  "type": "error",
  "payload": {
    "code": "string",
    "message": "string",
    "original_type": "string (opcional)"
  },
  "timestamp": 1234567890
}
```

| Código | Descripción |
|--------|-------------|
| `AUTH_FAILED` | Token inválido o expirado |
| `SESSION_EXPIRED` | Sesión revocada |
| `USER_NOT_FOUND` | Usuario destino no existe |
| `USER_OFFLINE` | Usuario destino no está conectado |
| `INVALID_SDP` | SDP malformado |
| `INVALID_ICE` | ICE candidate inválido |
| `RATE_LIMITED` | Demasiados mensajes |
| `INTERNAL_ERROR` | Error interno |

---

## TURN Credentials

### GET /turn/credentials

Obtener credenciales temporales para TURN server.

**Request:** Requiere autenticación.

**Response (200):**
```json
{
  "urls": ["turn:turn.halo.local:3478"],
  "username": "timestamp:user_id",
  "credential": "hmac_sha1_credential",
  "ttl": 3600
}
```

---

## WebRTC Data Channel Protocol

Una vez establecido el Data Channel, los mensajes se serializan así:

### Tipos de mensajes P2P

| type | Dirección | Descripción |
|------|-----------|-------------|
| `message` | bidireccional | Mensaje de texto cifrado |
| `delivery_receipt` | A → B | Confirmación de entrega |
| `read_receipt` | A → B | Confirmación de lectura |
| `typing_start` | A → B | Empezó a escribir |
| `typing_stop` | A → B | Dejó de escribir |
| `key_exchange` | bidireccional | Intercambio de claves E2EE |
| `group_key` | A → todos | Distribución de clave de grupo |
| `ping` | bidireccional | Heartbeat P2P |
| `pong` | bidireccional | Respuesta heartbeat |
| `error` | bidireccional | Error en el canal |

### Ejemplo: Mensaje cifrado

```json
{
  "type": "message",
  "payload": {
    "message_id": "uuid",
    "conversation_id": "uuid",
    "ciphertext": "base64_string",
    "iv": "base64_string (24 bytes)",
    "salt": "base64_string (32 bytes)",
    "signature": "base64_string (64 bytes)",
    "reply_to": "uuid | null",
    "timestamp": 1234567890
  }
}
```

### Ejemplo: Delivery receipt

```json
{
  "type": "delivery_receipt",
  "payload": {
    "message_id": "uuid",
    "timestamp": 1234567890
  }
}
```

### Ejemplo: Read receipt

```json
{
  "type": "read_receipt",
  "payload": {
    "message_id": "uuid",
    "timestamp": 1234567890
  }
}
```

### Ejemplo: Typing indicator

```json
{
  "type": "typing_start",
  "payload": {
    "conversation_id": "uuid"
  }
}
```

```json
{
  "type": "typing_stop",
  "payload": {
    "conversation_id": "uuid"
  }
}
```
