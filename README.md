# Halo

**Chat privado en tiempo real con cifrado extremo a extremo, red P2P mesh y autenticación centralizada.**

> **Plataforma:** Solo Linux. Tanto el servidor como el cliente Tauri están diseñados exclusivamente para Linux.

Halo es un sistema de mensajería que combina un servidor central ligero (autenticación, presencia, señalización WebRTC) con una red P2P mesh para el intercambio de mensajes. Todo el tráfico de mensajes viaja cifrado extremo a extremo a través de WebRTC Data Channels, sin que el servidor pueda acceder al contenido.

---

## Objetivo

Proveer un sistema de chat seguro, privado y descentralizado donde:

- Los mensajes viajan directamente entre pares (P2P) con cifrado E2EE
- El servidor solo gestiona autenticación, presencia y señalización
- El historial es local y nunca se almacena en el servidor
- Los grupos utilizan topología mesh (cada cliente conecta directamente con todos los miembros)
- No hay punto central de vigilancia ni almacenamiento de conversaciones

---

## Tecnologías

### Backend (Servidor Central)

| Tecnología | Propósito |
|-----------|-----------|
| Rust | Lenguaje principal |
| Axum | Framework HTTP + WebSocket |
| Tokio | Runtime asíncrono |
| jsonwebtoken | JWT (autenticación) |
| sqlx | Conexión a PostgreSQL |
| PostgreSQL | Base de datos |
| Serde | Serialización |
| Tracing | Observabilidad |
| UUID | Identificadores |
| bcrypt/argon2 | Hash de contraseñas |

### Frontend (Cliente Escritorio)

| Tecnología | Propósito |
|-----------|-----------|
| Tauri | Framework de aplicación nativa (Linux) |
| Rust | Lógica nativa (E2EE, WebRTC, almacenamiento) |
| WebRTC (webrtc-rs) | Data Channels P2P |
| Web frontend (HTML/CSS/JS) | Interfaz de usuario |
| keyring | Almacén seguro de claves |

---

## Arquitectura

### Hexagonal (Ports & Adapters)

El backend sigue una Arquitectura Hexagonal con tres capas:

```
┌─────────────────────────────┐
│       Adaptadores (API)      │  ← Axum handlers, WebSocket
├─────────────────────────────┤
│        Puertos (Ports)       │  ← Traits: AuthPort, MessagePort, etc.
├─────────────────────────────┤
│      Aplicación / Casos      │  ← Use cases
├─────────────────────────────┤
│         Dominio              │  ← Entidades puras, sin dependencias
└─────────────────────────────┘
```

**Regla fundamental:** El dominio nunca depende de infraestructura.

### Red P2P Mesh

```
Cliente A ──WebRTC── Cliente B
    │                      │
    ├──WebRTC── Cliente C ──┤
    │                      │
    └──WebRTC── Cliente D ──┘
           (Grupo Mesh)

          ▲              ▲
          │ Signaling    │ TURN relay
          ▼              ▼
       Servidor Central
     (Auth + Presencia)
```

---

## Instalación (futura)

```bash
# Backend
cd backend
cp .env.example .env
docker compose up -d db
cargo run

# Frontend
cd frontend
npm install
cargo tauri dev  # Solo Linux
```

---

## Roadmap

| Fase | Descripción |
|------|-------------|
| 1 | Planificación y documentación |
| 2 | Arquitectura y dominio |
| 3 | Infraestructura base (PostgreSQL, Axum) |
| 4 | Autenticación (registro, login, JWT) |
| 5 | Señalización WebRTC |
| 6 | Cliente Tauri base |
| 7 | WebRTC Data Channels (P2P 1:1) |
| 8 | Grupos mesh |
| 9 | E2EE |
| 10 | Delivery/Read/Typing |
| 11 | Testing |
| 12 | CI/CD y despliegue |

---

## Estructura del repositorio

```
docs/              Documentación completa del proyecto
├── architecture/  Documentos de arquitectura
├── adr/           Architecture Decision Records
├── api/           Especificación de API y protocolos
├── diagrams/      Diagramas de arquitectura
├── domain/        Modelo de dominio y casos de uso
├── meetings/      Notas de reuniones
├── planning/      Roadmap y estrategia
├── tasks/         Desglose de tareas
└── guides/        Convenciones, testing, seguridad

backend/           Servidor central (Rust/Axum)
├── src/
│   ├── domain/       Entidades y reglas de negocio
│   ├── application/  Casos de uso
│   ├── ports/        Interfaces (traits)
│   └── adapters/     Implementaciones concretas
└── tests/

frontend/          Cliente Tauri (Rust/Web)
├── src-tauri/     Lógica nativa Rust
├── src/           Frontend web
└── public/

scripts/           Scripts de desarrollo y despliegue
tools/             Herramientas auxiliares
.github/           CI/CD y plantillas
```

---

## Licencia

Por definir.
