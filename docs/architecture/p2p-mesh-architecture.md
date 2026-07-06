# Arquitectura de Red P2P Mesh con WebRTC

## Visión general

Halo utiliza una arquitectura híbrida: servidor central para funciones de coordinación + red P2P mesh para el intercambio de mensajes. Esto combina la seguridad de lo descentralizado con la practicidad de un servidor de señalización.

---

## Componentes de red

### Servidor Central (Rust/Axum)

**Funciones:**
- Registro y autenticación de usuarios (REST)
- Emisión y validación de JWT
- Descubrimiento de usuarios (who is online?)
- Señalización WebRTC (intercambio de SDP + ICE candidates)
- Gestión de presencia (conectados/desconectados)
- Relay TURN (solo cuando no es posible P2P directo)
- Health checks y métricas

**NO hace:**
- Almacenar mensajes
- Cifrar/descifrar contenido
- Intermediar en la comunicación de mensajes (salvo relay TURN forzado)

### Cliente (Tauri)

**Funciones:**
- Autenticación contra servidor central
- Conexión WebRTC directa con otros peers
- Cifrado/descifrado E2EE de mensajes
- Almacenamiento local del historial
- Gestión de claves E2EE
- Mesh de grupos (conexión directa con cada miembro)

---

## Protocolo de señalización

### 1. Conexión inicial

```
Cliente A                     Servidor                   Cliente B
    │                            │                          │
    │──── POST /auth/login ─────►│                          │
    │◄──── { jwt } ─────────────│                          │
    │                            │                          │
    │──── POST /auth/register ──►│                          │
    │◄──── { jwt } ─────────────│                          │
```

### 2. Presencia

```
Cliente A                     Servidor                   Cliente B
    │                            │                          │
    │──── WS /ws?token=jwt ────►│                          │
    │◄─── { type: "online" } ───│────► { type: "online" } │
    │                            │       (broadcast a      │
    │                            │        contactos)       │
    │◄─── { type: "user_list" } │                          │
    │       [users online]      │                          │
```

### 3. Señalización WebRTC (1:1)

```
Cliente A                     Servidor                   Cliente B
    │                            │                          │
    │──── { type: "call" } ────►│                          │
    │       target: B           │                          │
    │                            │──── { type: "offer" } ──►│
    │◄─── { type: "ringing" } ◄─│                          │
    │                            │                          │
    │                            │◄─── { type: "accept" } ─│
    │◄─── { type: "offer" } ◄───│                          │
    │       sdp: ...            │                          │
    │                            │                          │
    │──── { type: "answer" } ──►│                          │
    │       sdp: ...            │──── { type: "answer" } ──►│
    │                            │       sdp: ...           │
    │◄─── { type: "ice" } ◄─────│◄──── { type: "ice" } ───│
    │       candidate: ...      │       candidate: ...      │
    │──── { type: "ice" } ────►│──── { type: "ice" } ────►│
    │                            │                          │
    │══════ WebRTC Data Channel ══════════════════════════►│
    │          (P2P directo)                                │
```

### 4. Grupo Mesh

```
Para un grupo con N miembros, cada cliente mantiene
(N-1) conexiones WebRTC directas.

                    ┌──────────┐
           ┌───────│ Cliente A │────────┐
           │       └──────────┘        │
           ▼                           ▼
     ┌──────────┐               ┌──────────┐
     │ Cliente B │◄────...────►│ Cliente C │
     └──────────┘               └──────────┘

La señalización para cada par sigue el mismo protocolo 1:1.
```

---

## WebRTC Data Channel

### Configuración

| Parámetro | Valor |
|-----------|-------|
| ICE Servers | Servidor TURN propio + STUN público |
| Data Channel Protocol | Reliable (SCTP) |
| Data Channel Label | "halo-messages" |
| Orden garantizado | Sí (reliable mode) |

### Mensajes sobre Data Channel

Una vez establecido el canal, los mensajes siguen este formato (YA CIFRADOS E2EE):

```json
{
  "type": "message",
  "payload": "<ciphertext_base64>",
  "iv": "<iv_base64>",
  "salt": "<salt_base64>",
  "sender": "<user_id>",
  "conversation_id": "<uuid>",
  "timestamp": 1234567890,
  "signature": "<hmac_base64>"
}
```

### Tipos de mensajes

| Tipo | Descripción |
|------|-------------|
| `message` | Mensaje de texto cifrado |
| `typing` | Indicador de escritura |
| `read_receipt` | Confirmación de lectura |
| `delivery_receipt` | Confirmación de entrega |
| `key_exchange` | Intercambio de claves E2EE |
| `ping` | Heartbeat P2P |
| `pong` | Respuesta heartbeat |

---

## E2EE (Cifrado extremo a extremo)

### Esquema

- Algoritmo: **XChaCha20-Poly1305** (clave simétrica)
- Intercambio de claves: **X25519** (Curve25519 ECDH)
- Perfect Forward Secrecy: **Doble Rachet** (inspirado en Signal)
- Firma: **Ed25519** (autenticación del remitente)

### Flujo de establecimiento de claves

```
1. Cada cliente genera un par de claves (identity_key)
   - identity_public_key se registra en el servidor
   - identity_private_key en keychain local

2. Para iniciar conversación 1:1:
   - Cliente A solicita public_key de B al servidor
   - A genera ephemeral key pair
   - A realiza ECDH con su ephemeral + public_key de B
   - Se deriva clave compartida (shared_secret)

3. Para grupos:
   - Se usa un esquema de clave de grupo (Sender Keys)
   - Cada miembro tiene una clave de grupo
   - Los mensajes se cifran con la clave de grupo
   - La clave de grupo se distribuye cifrada con la clave 1:1 de cada miembro
```

### Almacenamiento de claves (cliente)

- Las claves se almacenan en el keychain del SO mediante `keyring` crate
- No hay respaldo de claves (si se pierden, el historial es irrecuperable)
- Las claves de sesión (ephemeral) se almacenan en memoria

---

## Manejo de desconexión y reconexión

### Desconexión

```
1. Cliente A pierde conexión con Servidor (WS)
2. Servidor marca A como offline → broadcast a contactos
3. Conexiones WebRTC activas de A con peers → hielan (ICE restart)
4. A queda en estado "reconnecting"
```

### Reconexión

```
1. A se reconecta al WS del servidor con nuevo JWT (o refresh)
2. Servidor marca A como online → broadcast a contactos
3. A recibe lista de contactos online
4. A inicia WebRTC con cada contacto online (re-establish)
5. Los peers detectan ICE restart y reestablecen Data Channels
6. Los mensajes offline se sincronizan desde peers (no hay servidor)
```

### Heartbeat

- Cada peer envía `ping` cada 15s sobre el Data Channel
- Si no se recibe `pong` en 30s, se considera desconectado
- El servidor también tiene heartbeat sobre WS cada 30s

---

## TURN Relay

Cuando no es posible P2P directo (NAT simétrico, firewall):

1. El cliente solicita credenciales TURN al servidor
2. El servidor emite credenciales temporales (asociadas al JWT)
3. El tráfico se enruta a través del servidor TURN
4. El mensaje sigue cifrado E2EE (el relay no puede leerlo)

---

## Limitaciones de Mesh

| Aspecto | Límite práctico |
|---------|----------------|
| Conexiones por cliente | ~50-100 conexiones WebRTC |
| Tamaño de grupo mesh | Recomendado < 20 miembros |
| Ancho de banda | O(N²) en grupos grandes |

Para grupos grandes (>20), se podría implementar relay/SFU como alternativa.
