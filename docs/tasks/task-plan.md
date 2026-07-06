# Plan de Tareas Detallado

## Fase 1: Documentación (Completada)

- [x] README.md
- [x] docs/architecture/hexagonal-architecture.md
- [x] docs/architecture/p2p-mesh-architecture.md
- [x] docs/domain/domain-model.md
- [x] docs/domain/use-cases.md
- [x] docs/adr/ (11 ADRs)
- [x] docs/api/rest-api.md
- [x] docs/api/signaling-protocol.md
- [x] docs/guides/conventions.md
- [x] docs/guides/git-flow.md
- [x] docs/guides/testing-strategy.md
- [x] docs/guides/observability.md
- [x] docs/guides/security.md
- [x] docs/diagrams/architecture-diagrams.md
- [x] docs/planning/roadmap.md
- [x] docs/planning/work-strategy.md
- [ ] docs/meetings/ (para futuras reuniones)
- [x] docs/tasks/task-plan.md

---

## Fase 2: Infraestructura Base (Backend)

### T2.1: Inicializar proyecto Cargo
- [ ] `cargo init backend --name halo-server`
- [ ] Configurar Cargo.toml con dependencias iniciales
- [ ] Configurar workspace (si aplica)

### T2.2: Estructura hexagonal
- [ ] Crear módulos: `domain`, `application`, `ports`, `adapters`
- [ ] Configurar `mod.rs` para cada módulo
- [ ] Configurar `lib.rs`

### T2.3: Configuración
- [ ] Crear `config.rs` para configuración
- [ ] Soporte para variables de entorno
- [ ] Archivo `.env.example`

### T2.4: Observabilidad base
- [ ] Configurar tracing subscriber
- [ ] Logging en desarrollo (formato humano)
- [ ] Logging en producción (formato JSON)

### T2.5: Base de datos
- [ ] Crear `docker-compose.yml` (PostgreSQL)
- [ ] Configurar sqlx pool
- [ ] Crear migración inicial: `users` table
- [ ] Crear migración: `sessions` table
- [ ] Crear migración: `groups` table
- [ ] Crear migración: `group_members` table
- [ ] Crear migración: `conversations` table

### T2.6: CI básico
- [ ] GitHub Actions: build
- [ ] GitHub Actions: clippy
- [ ] GitHub Actions: test

### T2.7: Health check
- [ ] Endpoint GET /health
- [ ] Verificar conexión BD

---

## Fase 3: Dominio

### T3.1: Errores de dominio
- [ ] `domain/error.rs` con todos los errores

### T3.2: Value Objects
- [ ] `UserId`, `Username`, `Email`, `PasswordHash`
- [ ] `SessionId`, `JwtId`, `JwtToken`
- [ ] `MessageId`, `ConversationId`, `GroupId`
- [ ] `Ciphertext`, `Iv`, `Salt`, `Signature`
- [ ] `Timestamp`, `IpAddress`, `UserAgent`
- [ ] Tests unitarios para cada VO

### T3.3: Entidades
- [ ] `User`: constructor, getters, validación
- [ ] `Session`: constructor, `is_expired()`, `revoke()`
- [ ] `Message`: constructor, validación de campos
- [ ] `Conversation`: constructor, validación de participantes
- [ ] `Group`: constructor, gestión de miembros
- [ ] `GroupMember`: constructor
- [ ] Tests unitarios para cada entidad

### T3.4: Enums
- [ ] `SessionStatus`, `MessageStatus`, `ConversationType`, `GroupRole`

---

## Fase 4: Puertos (Interfaces)

### T4.1: Repository traits
- [ ] `UserRepository`: `find_by_id`, `find_by_username`, `find_by_email`, `save`, `search`
- [ ] `SessionRepository`: `find_by_id`, `find_by_user_id`, `save`, `update_status`, `find_active_by_user`
- [ ] `ConversationRepository`: `find_by_id`, `find_by_participants`, `save`, `find_by_user`
- [ ] `GroupRepository`: `find_by_id`, `find_by_member`, `save`, `add_member`, `remove_member`

### T4.2: Service traits
- [ ] `AuthPort`: `generate_token`, `validate_token`, `refresh_token`, `revoke_token`
- [ ] `PresencePort`: `user_online`, `user_offline`, `is_online`, `get_online_users`
- [ ] `SignalingPort`: `send_to_user`, `broadcast_to_contacts`, `send_to_group_members`

---

## Fase 5: Casos de Uso

### T5.1: Autenticación
- [ ] `RegisterUseCase`: validar + crear usuario + generar JWT
- [ ] `LoginUseCase`: verificar credenciales + JWT + sesión
- [ ] `ValidateJwtUseCase`: verificar JWT + sesión activa
- [ ] `RefreshTokenUseCase`: rotar tokens
- [ ] `LogoutUseCase`: revocar sesión

### T5.2: Usuarios
- [ ] `SearchUsersUseCase`
- [ ] `ListOnlineUsersUseCase`
- [ ] `GetUserProfileUseCase`
- [ ] `GetUserPublicKeyUseCase`

### T5.3: WebSocket / Señalización
- [ ] `ConnectWsUseCase`
- [ ] `DisconnectWsUseCase`
- [ ] `ReconnectWsUseCase`
- [ ] `HandleSignalOfferUseCase`
- [ ] `HandleSignalAnswerUseCase`
- [ ] `HandleSignalIceCandidateUseCase`
- [ ] `HandleSignalAcceptUseCase`
- [ ] `HandleSignalRejectUseCase`

### T5.4: Grupos
- [ ] `CreateGroupUseCase`
- [ ] `AddGroupMemberUseCase`
- [ ] `RemoveGroupMemberUseCase`
- [ ] `LeaveGroupUseCase`
- [ ] `DeleteGroupUseCase`

### T5.5: Tests unitarios
- [ ] Tests para cada caso de uso con mocks

---

## Fase 6: Adaptadores de Persistencia

### T6.1: User repository
- [ ] `PostgresUserRepository`: implementar todos los métodos
- [ ] Tests de integración

### T6.2: Session repository
- [ ] `PostgresSessionRepository`
- [ ] Tests de integración

### T6.3: Conversation repository
- [ ] `PostgresConversationRepository`
- [ ] Tests de integración

### T6.4: Group repository
- [ ] `PostgresGroupRepository`
- [ ] Tests de integración

---

## Fase 7: API REST

### T7.1: Middleware
- [ ] Auth middleware (JWT validation)
- [ ] Error handling middleware
- [ ] CORS middleware
- [ ] Rate limiting middleware
- [ ] Request ID middleware

### T7.2: Auth routes
- [ ] POST /api/v1/auth/register
- [ ] POST /api/v1/auth/login
- [ ] POST /api/v1/auth/refresh
- [ ] POST /api/v1/auth/logout

### T7.3: User routes
- [ ] GET /api/v1/users/search
- [ ] GET /api/v1/users/online
- [ ] GET /api/v1/users/:id
- [ ] GET /api/v1/users/:id/public-key

### T7.4: Group routes
- [ ] POST /api/v1/groups
- [ ] GET /api/v1/groups/:id
- [ ] POST /api/v1/groups/:id/members
- [ ] DELETE /api/v1/groups/:id/members/:user_id
- [ ] DELETE /api/v1/groups/:id

### T7.5: TURN routes
- [ ] GET /api/v1/turn/credentials

### T7.6: Tests de integración
- [ ] Tests para cada endpoint

---

## Fase 8: WebSocket + Señalización

### T8.1: WebSocket handler
- [ ] WS /api/v1/ws
- [ ] Autenticación con token JWT
- [ ] Manejo de conexión/desconexión

### T8.2: Heartbeat
- [ ] Ping/Pong handler
- [ ] Timeout detection
- [ ] Sesión reconnecting

### T8.3: Presencia
- [ ] Broadcast de user_online/user_offline
- [ ] Lista de usuarios online

### T8.4: Señalización WebRTC
- [ ] Reenvío de offers
- [ ] Reenvío de answers
- [ ] Reenvío de ICE candidates
- [ ] Manejo de accept/reject
- [ ] Manejo de end/disconnect

### T8.5: Reconexión
- [ ] Manejar reconexión con misma sesión
- [ ] Timeout de reconexión

---

## Fase 9: Cliente Tauri

### T9.1: Inicialización
- [ ] Crear proyecto Tauri
- [ ] Configurar Cargo.toml con dependencias
- [ ] Configurar frontend base (HTML/CSS/JS framework)

### T9.2: Autenticación UI
- [ ] Pantalla de login
- [ ] Pantalla de registro
- [ ] Manejo de tokens (keychain)
- [ ] Auto-refresh de token

### T9.3: WebSocket cliente
- [ ] Conexión WS con JWT
- [ ] Reconexión automática
- [ ] Heartbeat

### T9.4: Contactos
- [ ] Lista de contactos
- [ ] Indicador online/offline
- [ ] Búsqueda de usuarios

### T9.5: Señalización WebRTC (Rust)
- [ ] Crear oferta SDP
- [ ] Manejar respuesta SDP
- [ ] Intercambio ICE candidates
- [ ] Establecer Data Channel

### T9.6: UI de chat 1:1
- [ ] Ventana de conversación
- [ ] Lista de mensajes
- [ ] Input de mensaje
- [ ] Indicador de escritura

### T9.7: Grupos
- [ ] Vista de grupos
- [ ] Crear grupo
- [ ] Chat de grupo (mesh)

### T9.8: Historial local
- [ ] Almacenamiento en disco
- [ ] Carga de historial
- [ ] Búsqueda en historial

### T9.9: Indicadores
- [ ] Delivery receipt indicator
- [ ] Read receipt indicator
- [ ] Typing indicator

---

## Fase 10: E2EE

### T10.1: Criptografía base
- [ ] Implementar X25519 ECDH
- [ ] Implementar XChaCha20-Poly1305
- [ ] Implementar Ed25519 firmas

### T10.2: Intercambio de claves (X3DH)
- [ ] Generación de claves identity
- [ ] Registro de clave pública
- [ ] Obtención de clave pública de otro usuario
- [ ] Derivación de clave compartida

### T10.3: Doble Rachet
- [ ] Implementación del protocolo
- [ ] Ratchet de cadena
- [ ] Ratchet Diffie-Hellman

### T10.4: Cifrado de mensajes
- [ ] Cifrar mensaje antes de enviar
- [ ] Descifrar mensaje al recibir
- [ ] Verificar firma

### T10.5: Clave de grupo
- [ ] Generar clave de grupo
- [ ] Distribuir clave cifrada a miembros
- [ ] Rotación de clave al añadir/remover miembros

### T10.6: Keychain integration
- [ ] Guardar claves en keychain del SO
- [ ] Cargar claves desde keychain

### T10.7: Tests
- [ ] Vectores de prueba conocidos
- [ ] Tests de integración E2EE

---

## Fase 11: Testing Completo

### T11.1: Tests unitarios (completar cobertura)
- [ ] Dominio: 95%+
- [ ] Casos de uso: 90%+
- [ ] Adaptadores: 80%+

### T11.2: Tests de integración
- [ ] API REST completa
- [ ] WebSocket + señalización
- [ ] Flujo de autenticación completo

### T11.3: Tests E2E
- [ ] Flujo completo: register → login → WS → WebRTC → message → delivery
- [ ] Grupo mesh con 3+ clientes
- [ ] Reconexión y recuperación

### T11.4: Benchmarks
- [ ] Conexiones concurrentes
- [ ] Throughput de señalización

---

## Fase 12: CI/CD y Despliegue

### T12.1: Docker
- [ ] Dockerfile para backend (multi-stage build)
- [ ] docker-compose.yml completo (backend + DB + Coturn)
- [ ] .dockerignore

### T12.2: CI/CD GitHub Actions
- [ ] Build + Test + Lint
- [ ] Build Tauri (Linux - AppImage/.deb)
- [ ] Deploy a servidor (SSH/Docker)

### T12.3: TURN Server
- [ ] Configuración de Coturn
- [ ] Integración con autenticación del backend

### T12.4: Documentación de despliegue
- [ ] Guía de instalación
- [ ] Variables de entorno
- [ ] Troubleshooting
