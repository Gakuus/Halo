# Guía de Desarrollo Local

## Requisitos

| Herramienta | Versión | Propósito |
|-------------|---------|-----------|
| Rust | 1.85+ | Compilador |
| Docker + Compose | Última | PostgreSQL |
| `cargo-make` | Última | Task runner |
| `cargo-audit` | Última | Escaneo de CVEs |
| `cargo-deny` | Última | Licencias |

```bash
rustup update stable
cargo install cargo-make cargo-audit cargo-deny
```

## Setup inicial

```bash
# 1. Clonar el repo
git clone git@github.com:Gakuus/Halo.git
cd Halo

# 2. Copiar configuración local
cp backend/.env.example backend/.env
# Editar .env si es necesario (los defaults funcionan para dev)

# 3. Levantar PostgreSQL
docker compose up -d

# 4. Correr migraciones
cd backend
cargo sqlx migrate run

# 5. Verificar que compila
cargo check --lib
```

## Comandos útiles

```bash
# Compilar todo
cargo build

# Tests unitarios (sin BD)
cargo test --lib

# Tests de integración (requiere BD corriendo)
cargo test

# Lint
cargo clippy -- -D warnings

# Formateo
cargo fmt --check   # verificar
cargo fmt --all     # aplicar

# Escaneo de seguridad
cargo audit

# Licencias
cargo deny check

# Limpiar build cache
cargo clean
```

## Variables de entorno

| Variable | Default | Descripción |
|----------|---------|-------------|
| `DATABASE_URL` | `postgres://halo:halo_dev@localhost:5432/halo` | Conexión a PostgreSQL |
| `JWT_SECRET` | `dev-secret-do-not-use-in-production` | Clave de firma JWT |
| `JWT_EXPIRATION_SECONDS` | `900` | Expiración access token (15 min) |
| `JWT_REFRESH_EXPIRATION_SECONDS` | `604800` | Expiración refresh token (7 días) |
| `HOST` | `0.0.0.0` | Host del servidor |
| `PORT` | `8080` | Puerto del servidor |
| `RUST_LOG` | `info` | Nivel de logging |
| `CORS_ORIGINS` | `http://localhost:5173,tauri://localhost` | Orígenes CORS |

## Docker Compose

```yaml
services:
  postgres:
    image: postgres:17-alpine
    environment:
      POSTGRES_USER: halo
      POSTGRES_PASSWORD: halo_dev
      POSTGRES_DB: halo
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data
```

```bash
# Iniciar
docker compose up -d

# Detener (sin borrar datos)
docker compose stop

# Detener y borrar datos
docker compose down -v

# Ver logs
docker compose logs -f
```

## Estructura del proyecto

```
backend/
  src/
    domain/         → Entidades, VO, errores
    application/    → Casos de uso
    ports/          → Traits (repositorios, servicios)
    adapters/
      api/          → Handlers HTTP, middleware, WebSocket
      db/           → Implementaciones Postgres
    config.rs       → Config desde env
    lib.rs          → Punto de entrada de librería
    main.rs         → Punto de entrada binario
  migrations/       → SQL migraciones (sqlx)
  tests/            → Tests de integración
```

## Workflow de ramas

```
main ← develop ← feature/*
```

Ver `docs/guides/git-flow.md` para detalles.

## Troubleshooting

### `cargo sqlx migrate run` falla
- Postgres está corriendo? `docker compose ps`
- Puerto correcto? `DATABASE_URL` en `.env`

### Tests de integración fallan
- `database URL` apunta a la BD de dev?
- Migraciones corriendo? `cargo sqlx migrate run`

### Clippy errors después de merge
- `cargo clippy --fix` corrige automáticamente
- Si persiste, puede ser un conflicto de merge

### Build lento
- `cargo build` en debug es más rápido
- Usar `sccache` para cachear compilaciones: `cargo install sccache && export RUSTC_WRAPPER=sccache`
