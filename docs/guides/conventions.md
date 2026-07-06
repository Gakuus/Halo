# Convenciones

## Naming

### Rust

| Elemento | Convención | Ejemplo |
|----------|-----------|---------|
| Módulos | `snake_case` | `user_repository` |
| Archivos | `snake_case.rs` | `postgres_user_repo.rs` |
| Structs | `PascalCase` | `UserRepository` |
| Traits | `PascalCase` | `UserRepository` |
| Enum + variants | `PascalCase` | `MessageStatus::Delivered` |
| Funciones | `snake_case` | `fn validate_jwt()` |
| Variables | `snake_case` | `let user_count` |
| Constantes | `SCREAMING_SNAKE_CASE` | `MAX_HEARTBEAT_INTERVAL` |
| Type aliases | `PascalCase` | `type Result<T> = ...` |
| Lifetime params | `'a`, `'b` | `&'a str` |
| Genéricos | `T`, `E` o descriptivo | `T: Repository` |
| Módulo privado | `_` prefijo | `_private_helper` |
| Error types | `PascalCase + Error` | `DomainError` |

### API

| Elemento | Convención | Ejemplo |
|----------|-----------|---------|
| Endpoints | `snake_case` | `/auth/login` |
| JSON fields | `snake_case` | `user_id`, `access_token` |
| Query params | `snake_case` | `?search_query` |
| Headers | `Pascal-Case` | `Authorization` |

### Base de datos

| Elemento | Convención | Ejemplo |
|----------|-----------|---------|
| Tablas | `snake_case` plural | `users`, `sessions` |
| Columnas | `snake_case` | `identity_public_key` |
| PK | `id` | `id UUID PRIMARY KEY` |
| FK | `{tabla}_id` | `user_id` |
| Timestamps | `created_at`, `updated_at` | |
| Índices | `idx_{tabla}_{columna}` | `idx_users_username` |

---

## Commits

### Formato (Conventional Commits)

```
<type>(<scope>): <description>

[optional body]
[optional footer]
```

### Types

| Type | Uso |
|------|-----|
| `feat` | Nueva funcionalidad |
| `fix` | Corrección de bug |
| `docs` | Cambios en documentación |
| `style` | Formato, lint, whitespace |
| `refactor` | Refactorización sin cambio funcional |
| `test` | Añadir o modificar tests |
| `chore` | Tareas de mantenimiento (CI, build, deps) |
| `perf` | Mejora de rendimiento |
| `sec` | Parche de seguridad |

### Scope (opcional)

- `core`, `domain`, `api`, `ws`, `db`, `auth`, `e2ee`, `frontend`, `docs`

### Ejemplos

```
feat(api): add user registration endpoint

Implement POST /api/v1/auth/register with validation
and JWT token generation.

Closes #12
```

```
fix(ws): handle websocket reconnection with expired token
```

```
docs(adr): add ADR-010 for p2p mesh topology
```

```
refactor(domain): simplify Session validation logic
```

---

## Branches

### Naming

| Rama | Formato | Origen |
|------|---------|--------|
| `main` | `main` | Protegida |
| `develop` | `develop` | `main` |
| `feature/*` | `feature/<nombre>` | `develop` |
| `fix/*` | `fix/<nombre>` | `develop` |
| `hotfix/*` | `hotfix/<nombre>` | `main` |
| `release/*` | `release/<version>` | `develop` |

### Reglas

- `main` y `develop` están protegidas (requieren PR + review)
- Las branches feature/fix se borran tras merge
- Un commit por feature (squash al mergear)
- Prefix numérico opcional: `feature/12-add-login`

---

## Documentación

- Toda la documentación en `docs/` en español (público objetivo hispanohablante)
- Código documentado en inglés (nombres, comentarios)
- README.md bilingüe (español para explicación, inglés técnico para código)
- ADRs en español
- Diagramas en formato compatible con Mermaid (dentro de .md)
- Cualquier decisión arquitectónica importante debe tener ADR

---

## Versionado

- SemVer estricto: `MAJOR.MINOR.PATCH`
- `MAJOR`: cambios incompatibles en API
- `MINOR`: nuevas funcionalidades compatibles
- `PATCH`: bug fixes compatibles
- Releases en GitHub con changelog
- Tags: `v1.0.0`, `v1.1.0`

---

## Logs

| Nivel | Uso |
|-------|-----|
| `error` | Errores que requieren atención humana |
| `warn` | Situaciones inesperadas pero recuperables |
| `info` | Eventos importantes (login, registro, conexión) |
| `debug` | Información detallada para desarrollo |
| `trace` | Datos muy detallados (solo debugging extremo) |

### Formato

```
2026-07-06T10:30:00Z INFO [user_service] User logged in: user_id=abc-123
2026-07-06T10:30:01Z ERROR [ws_handler] Connection failed: user_id=abc-123 error="session expired"
```

---

## Errores

- Todos los errores siguen una jerarquía: `DomainError` (negocio) / `InfrastructureError` (técnico)
- Los errores de dominio se definen en `domain/error.rs`
- Los errores de infraestructura se definen en cada adaptador
- Los errores se mapean a respuestas HTTP en los adaptadores API
- Usar `thiserror` crate para definir errores
- Los errores deben implementar `Display` y tener mensajes descriptivos

---

## Manejo de Result

```rust
// Tipo Result global del proyecto
type Result<T> = std::result::Result<T, DomainError>;

// En infraestructura, usar errores concretos
type RepositoryResult<T> = std::result::Result<T, RepositoryError>;
```

- Usar `anyhow` para errores no recuperables en binarios
- Usar `thiserror` para errores de dominio y de librería
- No usar `unwrap()` o `expect()` en producción (excepto tests)
- Preferir `map_err()` para convertir errores entre capas

---

## Formato de archivos

- UTF-8 sin BOM
- LF (Unix) como fin de línea
- indentación con 4 espacios (Rust estándar)
- `rustfmt` para formateo automático
- `clippy` sin warnings permitidos
- Máximo 100 caracteres por línea (rustfmt config)
- Una línea en blanco al final del archivo
