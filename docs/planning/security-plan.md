# Plan de Seguridad — Halo

## 1. Threat Model

### 1.1 Activos a proteger

| Activo | Descripción | Impacto si comprometido |
|--------|-------------|------------------------|
| Claves privadas (identity, ratchet, signed pre-key) | Claves criptográficas del usuario almacenadas en keychain del SO | Suplantación, descifrado de mensajes pasados y futuros |
| Mensajes (historial) | Contenido de conversaciones | Pérdida de privacidad total |
| Credenciales de acceso | Password + tokens JWT | Acceso no autorizado a la cuenta |
| Sesiones activas | Tokens de sesión WebSocket + P2P | Suplantación en tiempo real |
| Metadata de comunicaciones | Con quién, cuándo, frecuencia | Exposición de patrones de comunicación |
| Lista de contactos | Relaciones entre usuarios | Exposición de red social |
| Server secret key | Clave de firma JWT | Emisión de tokens arbitrarios |
| Database | Registros de usuarios, sesiones, grupos | Exposición masiva de datos |

### 1.2 Perfiles de atacante

| Atacante | Capacidades | Objetivo |
|----------|-------------|----------|
| **Externo (internet)** | Escucha pasiva de tráfico, escaneo de puertos, ataques DoS | Interceptar tráfico, identificar servicios |
| **MITM en red local** | ARP spoofing, DNS spoofing, interceptación de tráfico no TLS | Descifrar tráfico, inyectar mensajes |
| **Atacante web (OWASP Top 10)** | Inyección SQL, XSS, CSRF, path traversal | Acceso a BD, robo de datos |
| **Insider malicioso (admin server)** | Acceso a servidor, logs, BD, variables de entorno | Leer metadata, modificar usuarios, emitir tokens |
| **Insider curioso (admin server)** | Acceso de solo lectura a logs y BD | Leer metadata |
| **Atacante con acceso físico** | Acceso a disco, RAM, keychain del SO | Extraer claves, historial |
| **Estado/Fuerza de seguridad** | Capacidad de interceptar tráfico, exigir backdoors | Vigilancia masiva |
| **Proveedor de cloud / hosting** | Acceso a infraestructura subyacente | Análisis de tráfico, metadata |
| **Atacante de replay** | Intercepta y reenvía mensajes cifrados | Engañar al receptor con mensajes duplicados |
| **Atacante MITM con server hostil** | Servidor presenta claves públicas falsas para suplantar contactos | Interceptar claves, descifrar si el usuario no verifica |

### 1.3 Supuestos de seguridad

- El servidor central NO es trusted para el contenido de los mensajes (E2EE)
- El servidor central ES trusted para: autenticación, señalización, presencia, metadata de grupos
- Las claves privadas nunca salen del dispositivo del usuario
- El cliente Tauri es de código abierto y verificable (reproducible)
- TLS protege el tráfico entre cliente y servidor
- DTLS/SRTP protege el tráfico P2P (WebRTC)
- El keychain del SO protege claves en reposo

### 1.4 Riesgos aceptados

| Riesgo | Mitigación parcial |
|--------|-------------------|
| Metadata expuesta al servidor central | El servidor necesita metadata para funcionar (enrutamiento, presencia) |
| Análisis de tráfico (quién habla con quién) | Ofuscación de timing con tráfico dummy (futuro) |
| Servidor comprometido puede modificar código | Builds reproducibles + firmado de binarios (futuro) |
| Pérdida de claves = pérdida de historial | El usuario puede exportar/backup su keychain manualmente |

---

## 2. Gestión de Claves Criptográficas

### 2.1 Tipos de claves

| Clave | Algoritmo | Generación | Almacenamiento | Propósito |
|-------|-----------|------------|----------------|-----------|
| Identity Key Pair | Ed25519 | Al registrarse | Keychain SO + servidor (solo pública) | Identidad del usuario, firma de pre-keys |
| Signed Pre-Key Pair | X25519 | Al registrarse, rotación semanal | Keychain SO + servidor (pública firmada) | Intercambio inicial X3DH |
| One-Time Pre-Keys | X25519 | Lote de 100 al registrarse | Keychain SO + servidor (públicas, se agotan) | Intercambio inicial X3DH (PFS) |
| Ratchet Key Pair | X25519 | Por cada mensaje (Double Ratchet) | Keychain SO (volátil) | Derivation de claves de cifrado |
| Chain Keys | AES-256 | Por cada mensaje (Double Ratchet) | Keychain SO (volátil) | Cifrado de mensajes |

### 2.2 Ciclo de vida

```
Generación → Almacenamiento seguro → Uso → Rotación → Destrucción
```

#### 2.2.1 Generación
- Todas las claves se generan en el **dispositivo del cliente** usando `ring` o `sodiumoxide`
- Usar `OsRng` (CSPRNG del sistema operativo)
- La semilla se deriva del keychain del SO (Apple Keychain, Linux Secret Service, Windows DPAPI)

#### 2.2.2 Almacenamiento
- **Claves privadas**: Keychain del SO. Nunca se envían al servidor ni se guardan en disco sin cifrar.
- **Claves públicas**: Se envían al servidor en el registro y se almacenan en PostgreSQL.
- **Pre-keys de un solo uso**: Se almacenan cifradas en disco local (con clave derivada del keychain). El servidor mantiene un contador de cuántas quedan.

#### 2.2.3 Rotación

| Clave | Rotación | Trigger |
|-------|----------|---------|
| Identity Key | Nunca (salvo dispositivo perdido) | Voluntaria |
| Signed Pre-Key | Semanal | Temporizador |
| One-Time Pre-Key | Reponer cuando < 10 disponibles | Automático al conectarse |
| Ratchet Key | Cada mensaje | Automático |
| Chain Key | Cada mensaje | Automático |

#### 2.2.4 Destrucción segura
- Al eliminar cuenta: borrar todas las claves del servidor
- El cliente debe sobrescribir la memoria con `zeroize` después de usar cada clave
- Al hacer logout: destruir ratchet keys y chain keys (mantener identity + pre-keys)

### 2.3 Safety Numbers (verificación de claves)

Para prevenir MITM con server hostil, cada par de usuarios tiene un **safety number** derivado de ambas identity keys:

```
safety_number = SHA-512("Halo Safety Number v1" || alice_pub || bob_pub)
```

- Se muestra como 60 dígitos en pantalla
- Los usuarios comparan fuera de banda (en persona, QR, audio)
- Si cambia la identity key de un contacto, se alerta al usuario
- Opcional: marcar contacto como "verificado" después de confirmación

### 2.4 Recuperación ante pérdida de dispositivo
- **No hay recuperación de claves privadas** (el servidor no las tiene)
- El usuario debe generar nueva identity key → los contactos reciben notificación de "nuevo dispositivo"
- Los mensajes anteriores no se pueden descifrar (forward secrecy)
- Opción futura: backup cifrado de claves con recovery code (como Signal)

---

## 3. Seguridad en el Desarrollo (SDLC)

### 3.1 Prácticas obligatorias

| Práctica | Herramienta | Frecuencia |
|----------|-------------|------------|
| Análisis estático (SAST) | `cargo clippy -- -D warnings` | Cada commit |
| Escaneo de dependencias | `cargo audit`, Dependabot | Cada push + semanal |
| Formateo | `cargo fmt --check` | Cada commit |
| Tests unitarios | `cargo test --lib` | Cada push |
| Tests de integración | `cargo test` con BD real | Cada PR a develop |
| Code review | GitHub PR con al menos 1 revisor | Cada PR |
| Análisis de seguridad manual | Revisión de código criptográfico | Cada PR que toque crypto |

### 3.2 Dependencias

- `cargo deny` para verificar: licencias permitidas, sin `bitcoin`/`ssh` de fuentes no oficiales
- Política de actualización: CVEs críticos en < 48hs, altos en < 1 semana
- Lista blanca de crates permitidas para crypto: solo `ring`, `x25519-dalek`, `ed25519-dalek`, `chacha20poly1305`
- Prohibido: crates de crypto no auditadas, implementaciones propias de algoritmos estándar

### 3.3 Fuzzing
- `cargo fuzz` para parsers de mensajes y señalización
- Objetivos prioritarios: deserialización de mensajes WS, parsing de SDP, parsing de JWT

---

## 4. Account Lockout

### 4.1 Configuración

| Parámetro | Valor | Descripción |
|-----------|-------|-------------|
| `MAX_FAILED_ATTEMPTS` | 20 | Intentos fallidos antes de bloquear |
| `LOCKOUT_DURATION` | 24 horas | Tiempo que dura el bloqueo |

### 4.2 Comportamiento

1. Por cada login fallido, se incrementa `failed_login_attempts` en la BD (persiste entre reinicios)
2. Al alcanzar `MAX_FAILED_ATTEMPTS`, se establece `locked_until = now + LOCKOUT_DURATION`
3. En cada intento de login, se verifica `is_locked()` antes de checkear la contraseña
4. Si la cuenta está bloqueada, se devuelve error `AccountLocked` (HTTP 423)
5. Si el login es exitoso, se resetea `failed_login_attempts = 0` y `locked_until = NULL`
6. Si `locked_until` ya expiró, `is_locked()` devuelve `false` y se permite el login
7. Cuentas inexistentes: mismo tiempo de respuesta que cuentas existentes (no revelar existencia)

### 4.3 Seguridad

- El mensaje de error es genérico: no revelar si la cuenta está bloqueada o la contraseña es incorrecta
- El contador persiste en BD (no se pierde al reiniciar el servidor)
- No hay mecanismo de desbloqueo manual (solo esperar o reiniciar mediante DB admin)

---

## 5. Audit Logging

### 5.1 Replay protection

Cada mensaje incluye `sender_sequence_number: u64`, un contador **monotónico estrictamente creciente** por remitente.

- El receptor mantiene `(conversation_id, sender_id) -> last_seq`
- Si `seq ≤ last_seq` → rechazar (posible replay)
- Si `seq > last_seq` → aceptar y actualizar
- Gaps tolerados hasta 100 mensajes (por reordenamiento de red)

### 5.2 Eventos a registrar

| Evento | Nivel | Detalle |
|--------|-------|---------|
| Registro de usuario | INFO | user_id, username, IP (sin password ni key privada) |
| Login exitoso | INFO | user_id, IP, user_agent, session_id |
| Login fallido | WARN | IP, username intentado (sin password) |
| Logout | INFO | user_id, session_id |
| Creación de conversación | INFO | conversation_id, participant_a, participant_b |
| Creación de grupo | INFO | group_id, owner_id, member_count |
| Cambio de rol en grupo | INFO | group_id, target_user_id, new_role, actor_id |
| Eliminación de cuenta | INFO | user_id |
| Conexión WebSocket | INFO | user_id, session_id, IP |
| Desconexión WebSocket | INFO | user_id, session_id, reason |
| Error de autenticación | WARN | IP, reason |
| Rate limit excedido | WARN | IP, user_id, endpoint |
| Error interno | ERROR | error message, request_id |
| Intento de acceso a recurso ajeno | WARN | user_id, target_id, resource_type |
| Rotación de server secret | CRITICAL | admin_id (si aplica) |
| CVE detectado en dependencia | CRITICAL | crate, versión, CVE |

### 5.3 Prohibido registrar

- Contenido de mensajes (cifrados o no)
- Claves privadas o semillas
- Passwords en claro
- Tokens JWT completos (solo el jti/hash)
- Pre-keys de un solo uso

### 5.4 Protección de logs
- Logs rotados diariamente, retención 90 días
- Logs almacenados fuera del servidor de aplicación (centralizado)
- Acceso a logs solo para admins con 2FA
- Logs no contienen PII más allá de user_id
- En desarrollo: logs más detallados, pero sin secrets

---

## 6. Incident Response Plan

### 6.1 Clasificación de incidentes

| Severidad | Definición | Tiempo de respuesta |
|-----------|------------|---------------------|
| **CRITICAL** | Fuga de claves privadas de usuarios, breach de BD, ejecución remota de código | < 1 hora |
| **HIGH** | Fuga de metadata masiva, DoS sostenido, vulnerabilidad en autenticación | < 4 horas |
| **MEDIUM** | Fuga de logs, vulnerabilidad en endpoints no críticos, CVE en dependencia | < 24 horas |
| **LOW** | Spam, rate limiting insuficiente, información de versión expuesta | < 1 semana |

### 6.2 Playbook

#### CRITICAL: Breach de base de datos

1. **Detectar**: Alerta de monitoring, reporte externo, anomalía en logs
2. **Contener**: 
   - Rotar inmediatamente `JWT_SECRET` → invalida todos los tokens
   - Bloquear acceso externo a la BD (firewall)
   - Poner servidor en modo mantenimiento
3. **Investigar**:
   - Extraer logs de acceso a BD (quién, cuándo, qué tablas)
   - Determinar vector de ataque (SQLi? credenciales filtradas? 0-day?)
   - Estimar alcance (qué datos fueron accedidos)
4. **Notificar**: 
   - A todos los usuarios: pedir cambio de password
   - A autoridades regulatorias si aplica (GDPR: < 72 horas)
5. **Recuperar**:
   - Restaurar BD desde backup limpio
   - Aplicar parche de seguridad
   - No reusar el mismo `JWT_SECRET`
6. **Post-mortem**: Análisis de causa raíz, mejoras de seguridad

#### HIGH: Fuga de server secret key

1. Rotar `JWT_SECRET` inmediatamente (todos los tokens invalidados)
2. Forzar relogin de todos los usuarios
3. Revisar logs de emisión de tokens en busca de actividad anómala
4. Si se emitieron tokens maliciosos, revocar las sesiones asociadas

#### MEDIUM: CVE en dependencia crítica

1. Evaluar impacto en Halo (la crate vulnerable se usa? cómo?)
2. Si es explotable: actualizar y desplegar parche en < 24hs
3. Si no es explotable en nuestro contexto: documentar y monitorear

### 6.3 Comunicación
- Canal interno: Discord/Signal privado del equipo
- Canal externo: GitHub Security Advisories + comunicado oficial
- Plantilla de notificación a usuarios lista en `docs/incidents/notification-template.md`

---

## 7. Secrets Management

### 7.1 Qué es un secret

| Secret | Dónde se almacena | Acceso |
|--------|-------------------|--------|
| `JWT_SECRET` | GitHub Secrets / Vault / env | Solo CI + servidor producción |
| `DATABASE_URL` | GitHub Secrets / env | Solo CI + servidor producción |
| `TURN_SHARED_SECRET` | GitHub Secrets / env | Solo servidor producción |
| Password de base de datos | GitHub Secrets / env | Solo CI + servidor producción |
| Claves privadas de TLS | Cert manager / env | Solo servidor producción |

### 7.2 Reglas

- **Nunca** commitear secrets al repositorio
- Usar `.env.example` con placeholders (sin valores reales)
- En desarrollo: usar secrets locales vía `.env` (gitignorado)
- En CI: usar GitHub Secrets
- En producción: usar variables de entorno + Vault (futuro)
- Rotación periódica de todos los secrets (cada 6 meses como mínimo)
- Audit trail de quién accedió a cada secret (GitHub audit log)

### 7.3 Database credentials
- Usuario de BD con solo los permisos necesarios (no `superuser`)
- Password generado con `openssl rand -base64 32`
- Conexiones a BD solo desde el servidor de aplicación (firewall)
- En desarrollo: password único por desarrollador

---

## 8. Seguridad en Red

### 8.1 TLS

| Entorno | Versión TLS | Certificados |
|---------|-------------|--------------|
| Producción | TLS 1.3 mínimo | Let's Encrypt (autorenovación) |
| Staging | TLS 1.3 mínimo | Let's Encrypt o self-signed |
| Desarrollo | Sin TLS (localhost) | N/A |

### 8.2 Firewall (producción)

| Puerto | Servicio | Origen |
|--------|----------|--------|
| 443 | HTTPS/WSS (Axum) | 0.0.0.0/0 |
| 3478 | TURN (STUN) | 0.0.0.0/0 |
| 5349 | TURN (TLS) | 0.0.0.0/0 |
| 49152-65535 | TURN (relay) | 0.0.0.0/0 |
| 5432 | PostgreSQL | Solo servidor aplicación |
| 22 | SSH | Solo IPs del equipo |

### 8.3 Headers de seguridad HTTP

```
Strict-Transport-Security: max-age=63072000; includeSubDomains; preload
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
Content-Security-Policy: default-src 'self'; connect-src 'self' wss://*.halo.app
Referrer-Policy: no-referrer
Permissions-Policy: microphone=(), camera=()
```

---

## 9. Data at Rest

### 9.1 Base de datos

- Cifrado en reposo a nivel de disco (proveedor cloud: EBS encryption, etc.)
- Columnas con PII (email) candidatas a cifrado con `pgcrypto` (futuro)
- Backups cifrados con GPG
- Retención de backups: daily 7 días, weekly 4 semanas, monthly 12 meses
- Prueba de restore: mensual

### 9.2 Clientes (Tauri)

- Keychain del SO para claves criptográficas y tokens
- Caché de mensajes cifrados en SQLite local (cifrado con clave derivada del keychain)
- Opción de auto-lock de la app al bloquear pantalla
- Borrado de datos locales al hacer logout

---

## 10. GDPR / Compliance

### 10.1 Datos personales recolectados

| Dato | Propósito | Base legal | Retención |
|------|-----------|------------|-----------|
| Email | Identificación, recuperación de cuenta | Consentimiento | Hasta eliminación de cuenta |
| Username | Identificación pública | Necesidad contractual | Hasta eliminación de cuenta |
| Password hash | Autenticación | Necesidad contractual | Hasta eliminación de cuenta |
| IP Address | Seguridad, rate limiting | Interés legítimo | 90 días (logs) |
| User Agent | Diagnóstico | Interés legítimo | 90 días (logs) |
| Clave pública identity | Cifrado E2EE | Necesidad contractual | Hasta eliminación de cuenta |
| Metadata de conversaciones | Funcionamiento del servicio | Necesidad contractual | Hasta eliminación de cuenta |

### 10.2 Derechos del usuario

- **Acceso**: Endpoint `GET /account/data` exporta todos los datos del usuario en JSON
- **Rectificación**: Endpoint `PATCH /account/email` para cambiar email
- **Eliminación**: Endpoint `DELETE /account` elimina cuenta y todos los datos asociados
- **Portabilidad**: Exportar datos en formato JSON descargable
- **Oposición**: El usuario puede eliminar su cuenta en cualquier momento

### 10.3 Data Breach Notification
- Notificar a autoridad de protección de datos en < 72 horas (cuando aplique)
- Notificar a usuarios afectados en < 24 horas
- Mantener registro de todas las brechas (incluso las no notificables)

---

## 11. Disclosure Policy

### 11.1 Reporte de vulnerabilidades

```yaml
Email: security@halo.app (futuro)
Clave PGP: [fingerprint]
Tiempo esperado de respuesta: < 48 horas
Política: No acciones legales contra investigadores de buena fe
```

### 11.2 Proceso

1. Investigador reporta vulnerabilidad por email cifrado con PGP
2. Equipo de seguridad acusa recibo en < 24 horas
3. Evaluación de severidad en < 48 horas
4. Parche desarrollado en timeline acordado con el investigador
5. Vulnerabilidad parcheada y divulgada públicamente
6. Reconocimiento al investigador (Hall of Fame)

### 11.3 Fuera de alcance

- DoS/DDoS (reportar igual, pero baja prioridad)
- Phishing/social engineering contra usuarios
- Vulnerabilidades en dependencias de terceros ya reportadas
- Faltas a la guía de estilo / best practices sin impacto de seguridad

---

## 12. Cryptographic Agility

### 12.1 Protocol negotiation

Todos los protocolos criptográficos incluyen un campo `version` para permitir upgrades:

```
message_version: 1,
kem: "X25519",
aead: "XChaCha20-Poly1305",
hash: "SHA-512",
signature: "Ed25519"
```

### 12.2 Upgrade path

| Versión | Cambio |
|---------|--------|
| 1 (actual) | X25519 + XChaCha20-Poly1305 + Ed25519 |
| 2 (futuro) | Migrar a X25519MLKEM768 (PQ-safe) + AES-256-GCM |
| 3 (futuro) | Si NIST publica estándar post-cuántico, migrar a ese |

### 12.3 Compatibilidad hacia atrás

- Nuevos clientes pueden hablar con viejos usando la versión más baja común
- El servidor anuncia las versiones soportadas al conectar
- Deprecación de versión: anunciar con 6 meses de antelación

---

## 13. Checklists

### 13.1 Pre-deploy

- [ ] `cargo audit` sin vulnerabilidades conocidas
- [ ] `cargo clippy -- -D warnings` sin errores
- [ ] `cargo test --lib` 100% verde
- [ ] `cargo deny` sin problemas de licencias
- [ ] Secrets rotados respecto al deploy anterior?
- [ ] TLS configurado y verificado (SSL Labs test)
- [ ] Headers de seguridad HTTP configurados
- [ ] Rate limiting activado
- [ ] Logs sin PII
- [ ] Firewall rules aplicadas
- [ ] Backups configurados y probados

### 13.2 Post-incident

- [ ] Post-mortem documentado
- [ ] Parche aplicado y desplegado
- [ ] Usuarios notificados (si aplica)
- [ ] Autoridades notificadas (si aplica, GDPR)
- [ ] Medidas correctivas implementadas
- [ ] Lecciones aprendidas compartidas con el equipo
