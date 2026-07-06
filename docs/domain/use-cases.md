# Casos de Uso

Cada caso de uso se implementa como un struct en la capa de aplicación que recibe puertos (traits) por inyección de dependencias.

---

## Actor: Usuario no autenticado

### UC-01: Registrar usuario

**Descripción:** Un nuevo usuario crea una cuenta en el sistema.

**Entrada:**
- `username: String`
- `email: String`
- `password: String`
- `identity_public_key: [u8; 32]`

**Flujo:**
1. Validar formato de username, email y password
2. Verificar que username y email no existan
3. Hashear password con argon2id
4. Crear entidad `User`
5. Persistir usuario vía `UserRepository`
6. Generar JWT vía `AuthPort`
7. Crear sesión vía `SessionRepository`
8. Retornar `JwtToken` + datos del usuario

**Errores:** `DuplicateUsername`, `DuplicateEmail`, `InvalidUsername`, `InvalidEmail`, `WeakPassword`

### UC-02: Iniciar sesión

**Descripción:** Un usuario existente se autentica.

**Entrada:**
- `username: String` o `email: String`
- `password: String`

**Flujo:**
1. Buscar usuario por username/email
2. Verificar password contra hash (argon2id)
3. Revocar sesiones activas anteriores (si existen)
4. Generar nuevo JWT
5. Crear nueva sesión
6. Notificar a contactos sobre presencia online
7. Retornar `JwtToken` + datos del usuario

**Errores:** `InvalidCredentials`, `UserNotFound`

---

## Actor: Usuario autenticado (vía JWT)

### UC-03: Validar JWT

**Descripción:** Verifica que un token JWT sea válido y la sesión esté activa.

**Entrada:**
- `token: String`

**Flujo:**
1. Verificar firma y expiración del JWT
2. Buscar sesión asociada al `jti`
3. Verificar que la sesión esté activa
4. Retornar `UserId` y datos de sesión

**Errores:** `InvalidToken`, `SessionExpired`, `SessionRevoked`

### UC-04: Refrescar token

**Descripción:** Renueva un JWT próximo a expirar.

**Entrada:**
- `refresh_token: String`

**Flujo:**
1. Validar refresh token
2. Revocar sesión anterior
3. Crear nuevo JWT y sesión
4. Retornar nuevos tokens

**Errores:** `InvalidToken`, `SessionExpired`

### UC-05: Cerrar sesión

**Descripción:** Finaliza la sesión activa del usuario.

**Entrada:**
- `session_id: SessionId`

**Flujo:**
1. Revocar sesión
2. Notificar a contactos sobre presencia offline
3. Limpiar recursos asociados

**Errores:** `SessionNotFound`

---

### UC-06: Conectar WebSocket (señalización)

**Descripción:** Establece conexión WebSocket para señalización WebRTC.

**Entrada:**
- `jwt_token: String`

**Flujo:**
1. Validar JWT (UC-03)
2. Asociar conexión WS a la sesión
3. Registrar en presencia
4. Iniciar heartbeat
5. Enviar lista de usuarios online al cliente
6. Broadcast de "online" a contactos del usuario

**Errores:** `InvalidToken`, `SessionExpired`

### UC-07: Desconectar WebSocket

**Descripción:** Maneja la desconexión voluntaria o por timeout.

**Entrada:**
- `session_id: SessionId`

**Flujo:**
1. Marcar sesión como `Reconnecting`
2. Esperar reconexión (timeout configurable: 30s)
3. Si no reconecta, marcar como `Expired`
4. Notificar a contactos sobre offline

### UC-08: Reconectar WebSocket

**Descripción:** Reconexión tras pérdida de conexión.

**Entrada:**
- `jwt_token: String`
- `session_id: SessionId` (anterior)

**Flujo:**
1. Validar JWT
2. Buscar sesión anterior
3. Si está en `Reconnecting`, reactivar
4. Si expiró, crear nueva sesión (y revocar)
5. Reanudar presencia online

---

### UC-09: Iniciar llamada WebRTC (1:1)

**Descripción:** Un usuario inicia el establecimiento de un canal P2P con otro.

**Entrada:**
- `target_user_id: UserId`
- `session_id: SessionId`

**Flujo:**
1. Verificar que target_user existe y está online
2. Crear oferta SDP en el cliente (señalización)
3. Reenviar oferta al target vía su WS
4. Esperar respuesta
5. Reenviar answer y ICE candidates

**Errores:** `UserNotFound`, `NotOnline`, `SessionExpired`

### UC-10: Aceptar llamada WebRTC

**Descripción:** El destinatario acepta el establecimiento P2P.

**Entrada:**
- `sender_user_id: UserId`
- `sdp_answer: String`

### UC-11: Rechazar llamada WebRTC

**Descripción:** El destinatario rechaza el establecimiento P2P.

### UC-12: Intercambiar ICE candidates

**Descripción:** Durante el establecimiento WebRTC, los peers intercambian candidates.

---

### UC-13: Enviar mensaje 1:1

**Descripción:** El usuario envía un mensaje cifrado a otro usuario.

**Precondición:** Canal WebRTC Data Channel establecido entre ambos.

**Entrada (ya cifrada por el cliente):**
- `conversation_id: ConversationId`
- `ciphertext: Vec<u8>`
- `iv: [u8; 24]`
- `salt: [u8; 32]`
- `signature: [u8; 64]`
- `reply_to: Option<MessageId>`

**Flujo:**
1. Validar que ambos son participantes de la conversación
2. Reenviar payload cifrado al destinatario por su Data Channel
3. Esperar confirmación de entrega
4. Actualizar estado a `Delivered`

**Errores:** `NotParticipant`, `ConversationNotFound`

### UC-14: Recibir mensaje 1:1

**Descripción:** El destinatario recibe un mensaje por su Data Channel.

**Flujo:**
1. Recibir payload cifrado
2. Enviar confirmación de entrega (`delivery_receipt`)
3. Descifrar localmente con clave E2EE
4. Almacenar en historial local
5. Notificar a la UI

### UC-15: Enviar mensaje a grupo

**Descripción:** Envía un mensaje cifrado a todos los miembros del grupo.

**Precondición:** Data Channel establecido con cada miembro del grupo.

**Flujo:**
1. Cifrar con clave de grupo
2. Enviar a cada miembro por su Data Channel respectivo
3. Cada miembro descifra con clave de grupo

### UC-16: Crear grupo

**Descripción:** Un usuario crea un grupo con otros usuarios.

**Entrada:**
- `name: String`
- `member_ids: Vec<UserId>`

**Flujo:**
1. Validar nombre del grupo
2. Verificar que todos los miembros existen
3. Crear entidad `Group` con owner = current_user
4. Generar clave de grupo
5. Cifrar clave de grupo con clave 1:1 de cada miembro
6. Persistir grupo
7. Iniciar señalización WebRTC con todos los miembros

### UC-17: Añadir miembro a grupo

**Descripción:** El owner/admin añade un nuevo miembro.

**Flujo:**
1. Verificar permisos (owner/admin)
2. Generar nueva clave de grupo (key rotation)
3. Distribuir nueva clave cifrada a todos los miembros

### UC-18: Abandonar grupo

**Descripción:** Un miembro sale del grupo.

### UC-19: Eliminar grupo

**Descripción:** El owner elimina el grupo.

---

### UC-20: Enviar indicador de escritura

**Descripción:** Notifica que el usuario está escribiendo.

**Entrada:**
- `conversation_id: ConversationId`

**Flujo:**
1. Enviar `{ type: "typing" }` por Data Channel

### UC-21: Enviar confirmación de lectura

**Descripción:** Notifica que el usuario leyó un mensaje.

**Entrada:**
- `message_id: MessageId`

### UC-22: Enviar confirmación de entrega

**Descripción:** Notifica que el mensaje llegó al destinatario.

**Entrada:**
- `message_id: MessageId`

---

### UC-23: Buscar usuarios

**Descripción:** Busca usuarios por username o email.

**Entrada:**
- `query: String`

### UC-24: Listar usuarios online

**Descripción:** Obtiene la lista de contactos con sesión activa.

### UC-25: Obtener perfil de usuario

**Descripción:** Obtiene datos públicos de un usuario (username, clave pública).

---

### UC-26: Intercambio de claves E2EE (X3DH)

**Descripción:** Establece una clave compartida entre dos usuarios.

**Flujo:**
1. Solicitar `identity_public_key` del otro usuario al servidor
2. Generar ephemeral key pair
3. Realizar ECDH
4. Derivar shared_secret
5. Almacenar localmente

### UC-27: Rotación de clave de grupo

**Descripción:** Renueva la clave compartida del grupo.

**Flujo:**
1. Generar nueva clave de grupo
2. Cifrar con clave 1:1 de cada miembro
3. Distribuir vía Data Channel a cada miembro
