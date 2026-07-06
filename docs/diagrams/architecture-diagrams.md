# Diagramas de Arquitectura

## Diagrama 1: Arquitectura General del Sistema

```
┌──────────────────────────────────────────────────────────────────────┐
│                        Cliente A (Tauri)                            │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    Interfaz Web (HTML/CSS/JS)                │   │
│  ├──────────────────────────────────────────────────────────────┤   │
│  │                    Lógica Rust Nativa                        │   │
│  │  ┌─────────────┐  ┌──────────────┐  ┌────────────────────┐  │   │
│  │  │ E2EE Engine │  │ WebRTC Peer  │  │  Local Storage     │  │   │
│  │  │ (XChaCha20, │  │ (Data Ch.)   │  │  (Historial, keys) │  │   │
│  │  │  X25519)    │  │              │  │                    │  │   │
│  │  └─────────────┘  └──────────────┘  └────────────────────┘  │   │
│  └──────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────────┘
         │                         │
         │ WS (señalización)       │ WebRTC Data Channel (P2P)
         ▼                         ▼
┌──────────────────────────────────────────────────────────────────────┐
│                    Servidor Central (Rust/Axum)                     │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  ┌─────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐ │   │
│  │  │ Auth    │  │   WS     │  │  REST    │  │  Signaling   │ │   │
│  │  │ (JWT)   │  │ Handler  │  │  API     │  │  Manager     │ │   │
│  │  └─────────┘  └──────────┘  └──────────┘  └──────────────┘ │   │
│  │  ┌──────────────────────────────────────────────────────┐   │   │
│  │  │            Hexagonal Core (Domain + Use Cases)       │   │   │
│  │  └──────────────────────────────────────────────────────┘   │   │
│  │  ┌───────────┐  ┌──────────┐  ┌─────────────┐             │   │
│  │  │ Presence  │  │   DB     │  │  TURN Auth  │             │   │
│  │  │ Manager   │  │ Adapt.   │  │  (creds)    │             │   │
│  │  └───────────┘  └──────────┘  └─────────────┘             │   │
│  └──────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────────┘
         │
         │ SQL
         ▼
┌──────────────────────────────────────────────────────────────────────┐
│                     PostgreSQL                                       │
│  ┌──────────┐  ┌──────────┐  ┌────────────┐  ┌──────────┐         │
│  │  users   │  │ sessions │  │  groups    │  │conversat.│         │
│  └──────────┘  └──────────┘  └────────────┘  └──────────┘         │
└──────────────────────────────────────────────────────────────────────┘

         WebRTC P2P (sin servidor)
┌──────────────────────────────────────────────────────────────────────┐
│  Cliente A ══════════════════════════════════════════ Cliente B     │
│            WebRTC Data Channel (cifrado E2EE)                       │
└──────────────────────────────────────────────────────────────────────┘
```

---

## Diagrama 2: Arquitectura Hexagonal (Capas)

```
                   ┌──────────────────────────────┐
                   │        Adaptadores IN         │
                   │  (API HTTP, WebSocket)        │
                   └──────────┬───────────────────┘
                              │ depende de
                              ▼
                   ┌──────────────────────────────┐
                   │        Puertos (Ports)        │
                   │  UserRepository trait         │
                   │  SessionRepository trait      │
                   │  AuthPort trait               │
                   │  PresencePort trait           │
                   │  SignalingPort trait          │
                   └──────────┬───────────────────┘
                              │ implementa
                              ▼
                   ┌──────────────────────────────┐
                   │      Aplicación (Use Cases)   │
                   │  LoginUseCase                │
                   │  RegisterUseCase             │
                   │  SendMessageUseCase          │
                   │  CreateGroupUseCase          │
                   └──────────┬───────────────────┘
                              │ usa
                              ▼
                   ┌──────────────────────────────┐
                   │         Dominio               │
                   │  User, Message, Session       │
                   │  Conversation, Group          │
                   │  Value Objects, Errors        │
                   │  (0 dependencias externas)    │
                   └──────────────────────────────┘
                              ▲
                              │ implementa
                   ┌──────────────────────────────┐
                   │       Adaptadores OUT         │
                   │  PostgresUserRepository       │
                   │  PostgresSessionRepository    │
                   │  JwtAuthAdapter               │
                   │  TokioPresenceAdapter         │
                   └──────────────────────────────┘
```

---

## Diagrama 3: Flujo de Conexión (Login → WebSocket → WebRTC)

```
Cliente A                   Servidor                    Cliente B
    │                          │                          │
    │  ─────────────────────── │ ──────────────────────── │
    │  FASE 1: Autenticación   │                          │
    │  ─────────────────────── │ ──────────────────────── │
    │                          │                          │
    │── POST /auth/login ─────►│                          │
    │  { username, password }  │                          │
    │                          │  Validar credenciales     │
    │                          │  Generar JWT             │
    │                          │  Crear sesión            │
    │◄─── { access_token,  ────│                          │
    │       refresh_token }    │                          │
    │                          │                          │
    │  ─────────────────────── │ ──────────────────────── │
    │  FASE 2: WebSocket       │                          │
    │  ─────────────────────── │ ──────────────────────── │
    │                          │                          │
    │── WS /ws?token=jwt ─────►│                          │
    │                          │  Validar JWT             │
    │                          │  Asociar WS a sesión     │
    │                          │  Registrar presencia     │
    │◄─── { welcome,         ──│                          │
    │       online_users }     │                          │
    │                          │── { user_online: A } ───►│
    │                          │                          │
    │  ─────────────────────── │ ──────────────────────── │
    │  FASE 3: Señalización    │                          │
    │  ─────────────────────── │ ──────────────────────── │
    │                          │                          │
    │  Usuario A inicia chat con B                        │
    │                          │                          │
    │── { signal_offer,     ──►│                          │
    │    target: B, sdp }     │                          │
    │                          │── { incoming_offer, ───►│
    │                          │    from: A, sdp }        │
    │                          │                          │
    │                          │◄─── { signal_answer,  ───│
    │                          │     target: A, sdp }     │
    │◄─── { incoming_answer, ──│                          │
    │    from: B, sdp }       │                          │
    │                          │                          │
    │── { signal_ice,       ──►│                          │
    │    target: B, cand }    │── { incoming_ice } ─────►│
    │◄─── { incoming_ice } ◄───│◄─── { signal_ice } ─────│
    │                          │                          │
    │  ─────────────────────── │ ──────────────────────── │
    │  FASE 4: P2P Data Channel                          │
    │  ─────────────────────── │ ──────────────────────── │
    │                          │                          │
    │═══ WebRTC Data Channel ═══════════════════════════►│
    │═══ (XChaCha20 cifrado) ═══════════════════════════►│
    │═══ (directo, sin server)══════════════════════════►│
```

---

## Diagrama 4: Topología Mesh para Grupos

```
Grupo con 4 miembros:

                    ┌──────────┐
           ┌───────│ Cliente A │───────┐
           │       └──────────┘       │
           │         │    │           │
           │    ┌────┘    └────┐      │
           ▼    ▼              ▼      ▼
     ┌──────────┐         ┌──────────┐
     │ Cliente B │ ◄─────►│ Cliente C │
     └──────────┘         └──────────┘
           │                    │
           │                    │
           ▼                    ▼
     ┌─────────────────────────────────┐
     │          Cliente D              │
     └─────────────────────────────────┘

Conexiones por nodo: N-1 = 3
Conexiones totales: N*(N-1)/2 = 6
```

---

## Diagrama 5: Flujo de Mensaje E2EE

```
Cliente A (Remitente)                     Cliente B (Destinatario)
    │                                            │
    │  1. Escribir mensaje                       │
    │     "Hola"                                 │
    │                                            │
    │  2. Cifrar con clave compartida            │
    │     XChaCha20-Poly1305                     │
    │     ↓                                      │
    │     ciphertext = encrypt("Hola", key, iv)  │
    │     signature = sign(ciphertext, sk)       │
    │                                            │
    │  3. Enviar por WebRTC Data Channel         │
    │  ─────────────────────────────────────────►│
    │     {                                      │
    │       type: "message",                     │
    │       conversation_id: "uuid",             │
    │       ciphertext: "base64...",             │
    │       iv: "base64...",                     │
    │       salt: "base64...",                   │
    │       signature: "base64...",              │
    │       timestamp: 1234567890               │
    │     }                                      │
    │                                            │
    │  4. Delivery receipt                       │
    │◄───────────────────────────────────────────│
    │     { type: "delivery_receipt",            │
    │       message_id: "uuid" }                 │
    │                                            │
    │                                            │  5. Descifrar
    │                                            │     verify(signature, pk)
    │                                            │     text = decrypt(ciphertext, key, iv)
    │                                            │     "Hola" ✓
    │                                            │
    │                                            │  6. Mostrar en UI
    │                                            │
    │  7. Read receipt                           │
    │◄───────────────────────────────────────────│
    │     { type: "read_receipt",                │
    │       message_id: "uuid" }                 │
```

---

## Diagrama 6: Modelo de Dominio (ER)

```
┌─────────────┐       ┌─────────────┐       ┌────────────────┐
│    User     │       │   Session   │       │ Conversation   │
├─────────────┤       ├─────────────┤       ├────────────────┤
│ id (PK)     │──1:N──│ id (PK)     │       │ id (PK)        │
│ username    │       │ user_id(FK) │       │ participant_a  │
│ email       │       │ jwt_id      │       │ participant_b  │
│ password_   │       │ status      │       │ created_at     │
│   hash      │       │ connected_  │       │ last_message_  │
│ identity_   │       │   at        │       │   at           │
│   public_key│       │ last_       │       │ is_active      │
│ created_at  │       │   heartbeat │       └────────────────┘
│ updated_at  │       │ ip_address  │              │
└──────┬──────┘       │ user_agent  │              │ 1:N
       │              └─────────────┘              │
       │ 1:N                    │                  ▼
       │                        │          ┌────────────────┐
       │                        │          │    Message     │
       ▼                        │          ├────────────────┤
┌─────────────┐                 │          │ id (PK)        │
│   Group     │                 │          │ conversation_  │
├─────────────┤                 │          │   id (FK)      │
│ id (PK)     │                 │          │ sender_id (FK) │
│ name        │                 │          │ ciphertext     │
│ owner_id(FK)│                 │          │ iv             │
│ created_at  │                 │          │ salt           │
└──────┬──────┘                 │          │ signature      │
       │                        │          │ status         │
       │ 1:N                    │          │ timestamp      │
       ▼                        │          │ reply_to       │
┌─────────────┐                 │          └────────────────┘
│GroupMember  │                 │
├─────────────┤                 │
│ group_id(FK)│                 │
│ user_id(FK) │                 │
│ role        │                 │
│ joined_at   │                 │
│ encrypted_  │                 │
│   group_key │                 │
└─────────────┘                 │
```

---

## Diagrama 7: Estados de Sesión

```
                  ┌──────────┐
                  │  Active  │
                  └────┬─────┘
                       │
              ┌────────┴────────┐
              │                 │
              ▼                 ▼
        ┌────────────┐   ┌──────────┐
        │Reconnecting│   │ Expired  │
        └──────┬─────┘   └──────────┘
               │
      ┌────────┴────────┐
      │                 │
      ▼                 ▼
  ┌────────┐    ┌──────────┐
  │ Active │    │ Expired  │
  │(recon.) │    │(timeout) │
  └────────┘    └──────────┘
```

---

## Diagrama 8: Seguridad JWT

```
                    ┌──────────────────┐
                    │  Cliente         │
                    │  (Tauri/Web)     │
                    └────────┬─────────┘
                             │
              ┌──────────────┴──────────────┐
              │                             │
              ▼                             ▼
    ┌─────────────────┐          ┌──────────────────┐
    │ POST /auth/     │          │ WS /ws?token=... │
    │ login           │          │                  │
    └────────┬────────┘          └────────┬─────────┘
             │                           │
             ▼                           ▼
    ┌──────────────────────────────────────────┐
    │           Servidor Central               │
    │                                          │
    │  ┌──────────────────────────────────┐    │
    │  │         JWT Validation            │    │
    │  │  1. Verificar firma (HS256)      │    │
    │  │  2. Verificar exp (no expirado)  │    │
    │  │  3. Verificar jti en BD (activo) │    │
    │  │  4. Extraer sub (user_id)        │    │
    │  └──────────────────────────────────┘    │
    │                                          │
    │  ┌──────────────────────────────────┐    │
    │  │       Session Management         │    │
    │  │  1. Buscar sesión por jti        │    │
    │  │  2. Verificar status = Active    │    │
    │  │  3. Actualizar heartbeat         │    │
    │  │  4. Si reconnect: validar        │    │
    │  └──────────────────────────────────┘    │
    └──────────────────────────────────────────┘
```

---

## Diagrama 9: Paquetes/Dependencias del Backend

```
┌──────────────────────────────────────────────────────────────────┐
│                      backend (Cargo workspace)                   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │                     dependencias                         │    │
│  ├──────────────────────────────────────────────────────────┤    │
│  │ dominios: sin dependencias externas                      │    │
│  │ aplicación: dominio + async_trait                        │    │
│  │ puertos: dominio + async_trait                           │    │
│  │ adaptadores: puertos + dominio + axum + sqlx + tokio     │    │
│  │               + jsonwebtoken + serde + tracing + uuid    │    │
│  └──────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Crate externos:                                                 │
│  ├── axum (HTTP + WS)                                            │
│  ├── tokio (async runtime)                                       │
│  ├── sqlx (PostgreSQL)                                           │
│  ├── jsonwebtoken (JWT)                                          │
│  ├── serde / serde_json (serialization)                          │
│  ├── tracing / tracing-subscriber (observabilidad)               │
│  ├── uuid (UUID v7)                                              │
│  ├── argon2 (password hashing)                                   │
│  ├── thiserror (errores)                                         │
│  ├── tower-http (middleware CORS, compression, etc.)             │
│  ├── config (configuración)                                      │
│  └── mockall (testing, dev-dep)                                  │
└──────────────────────────────────────────────────────────────────┘
```

---

## Diagrama 10: Flujo de Despliegue

```
                      GitHub
                         │
                         ▼
              ┌──────────────────────┐
              │   GitHub Actions     │
              │                      │
              │  CI: build + test    │
              │  CD: deploy          │
              └──────────────────────┘
                         │
               ┌─────────┴──────────┐
               │                    │
               ▼                    ▼
      ┌────────────────┐   ┌────────────────┐
      │  Docker Image  │   │  Tauri Build   │
      │  (Backend)     │   │  (AppImage/    │
      │                │   │   .deb/.dmg)   │
      └────────┬───────┘   └────────────────┘
               │
               ▼
      ┌────────────────┐
      │   Servidor      │
      │                 │
      │  ┌───────────┐  │
      │  │ Backend   │  │
      │  │ (Axum)    │  │
      │  ├───────────┤  │
      │  │ PostgreSQL│  │
      │  ├───────────┤  │
      │  │ Coturn    │  │
      │  │ (TURN)    │  │
      │  ├───────────┤  │
      │  │ Nginx     │  │
      │  │ (TLS)     │  │
      │  └───────────┘  │
      └────────────────┘
```
