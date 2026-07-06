# Estrategia de Testing

## Principios

1. El dominio y los casos de uso se testean sin infraestructura
2. Tests rápidos (unitarios) vs tests lentos (integración)
3. Cobertura: priorizar lógica de negocio sobre boilerplate
4. No testear getters/setters triviales
5. Testear casos borde y errores

---

## Pirámide de testing

```
      ╱╲
     ╱ E2E ╲
    ╱────────╲
   ╱ Integración ╲
  ╱────────────────╲
 ╱  Unit tests (dominio) ╲
╱──────────────────────────╲
```

---

## 1. Tests Unitarios

### Dominio (capa domain)

**¿Qué testear?**
- Entidades: constructores válidos e inválidos
- Value Objects: validación de formatos
- Reglas de negocio: invariantes
- Domain Errors: conversiones, mensajes

**¿Cómo?**
- Sin dependencias externas
- Con `#[cfg(test)] mod tests` en el mismo archivo
- Usar `assert_eq!`, `assert_matches!`, `assert!`

**Ejemplos de tests:**
- `Username::new("ab")` → error (muy corto)
- `Username::new("valid_user_123")` → ok
- `Email::new("invalido")` → error
- `User::new()` con todos los campos válidos → ok
- `Session::is_expired()` con heartbeat reciente → false
- `Session::is_expired()` sin heartbeat → true

### Casos de uso (capa application)

**¿Qué testear?**
- Flujo feliz de cada caso de uso
- Casos de error (credenciales inválidas, usuario no encontrado, etc.)
- Invariantes que el caso de uso debe mantener

**¿Cómo?**
- Mockear puertos con traits
- Inyectar implementaciones mock (pueden ser manuales o con `mockall`)
- Verificar interacciones con los mocks

```rust
// Pseudoejemplo
struct MockUserRepository {
    users: Vec<User>,
}

impl UserRepository for MockUserRepository { ... }

#[tokio::test]
async fn test_login_success() {
    let repo = MockUserRepository::with_user("test", "hash...");
    let auth = MockAuthPort::new();
    let use_case = LoginUseCase::new(repo, auth);
    let result = use_case.execute("test", "password").await;
    assert!(result.is_ok());
}
```

---

## 2. Tests de Integración

### Repositorios (adaptadores DB)

**¿Qué testear?**
- CRUD básico de cada repositorio
- Queries específicas (búsqueda, filtros)
- Migraciones (que se ejecutan correctamente)
- Transacciones y rollbacks

**¿Cómo?**
- Base de datos PostgreSQL real en Docker
- `testcontainers` crate para levantar BD en tests
- O usar BD separada por test con `sqlx::test`
- Limpiar datos entre tests

```rust
// Pseudoejemplo
#[sqlx::test]
async fn test_create_user(pool: PgPool) {
    let repo = PostgresUserRepository::new(pool);
    let user = User::new(...);
    let result = repo.save(user).await;
    assert!(result.is_ok());
}
```

### API REST

**¿Qué testear?**
- Endpoints responden con códigos correctos
- Validación de entrada (400)
- Autenticación (401 sin token)
- Permisos (403)
- Flujos completos (register → login → access)

**¿Cómo?**
- Levantar servidor Axum en tests
- Cliente HTTP real (reqwest)
- BD de test

### WebSocket + Señalización

**¿Qué testear?**
- Conexión con token válido → éxito
- Conexión con token inválido → rechazo
- Heartbeat funciona
- Señalización: offer → answer → ICE candidates
- Presencia: online/offline correcto
- Reconexión con nueva sesión

**¿Cómo?**
- Cliente WebSocket de prueba (tokio_tungstenite)
- Múltiples clientes simultáneos
- Timeouts para evitar tests colgados

---

## 3. Tests E2E

**¿Qué testear?**
- Flujo completo: register → login → WS → WebRTC → message → delivery
- Grupo mesh con 3+ clientes
- Reconexión y recuperación
- E2EE: cifrar → enviar → recibir → descifrar

**¿Cómo?**
- Servidor real en localhost
- Múltiples instancias del cliente Tauri headless (o simulación)
- Script de test que coordina los clientes

---

## 4. Mocking

### mockall

Usar `mockall` crate para generar mocks de traits:

```rust
#[mockall::automock]
trait UserRepository {
    async fn find_by_username(&self, username: &str) -> Result<User, DomainError>;
    async fn save(&self, user: User) -> Result<(), DomainError>;
}
```

### Alternativa: mocks manuales

Para traits simples, implementar mocks manuales:

```rust
struct MockUserRepository {
    should_fail: bool,
    users: HashMap<String, User>,
}
```

---

## Configuración del proyecto

```
backend/
├── tests/
│   ├── common/
│   │   └── mod.rs          # Setup compartido (BD test, clientes)
│   ├── api/
│   │   ├── auth_test.rs
│   │   └── users_test.rs
│   ├── ws/
│   │   └── signaling_test.rs
│   └── e2e/
│       └── chat_flow_test.rs
└── src/
    ├── domain/
    │   ├── user.rs          # #[cfg(test)] mod tests
    │   ├── message.rs       # #[cfg(test)] mod tests
    │   └── ...
    └── application/
        └── login_test.rs    # Tests unitarios simulacro
```

## Comandos

```bash
# Tests unitarios (rápidos, sin BD)
cargo test --lib

# Tests de integración (requieren BD)
cargo test --test '*' -- --test-threads=1

# Todos los tests
cargo test

# Con cobertura
cargo tarpaulin --ignore-tests
```

## Objetivo de cobertura

| Capa | Cobertura mínima |
|------|-----------------|
| Dominio | 95%+ |
| Casos de uso | 90%+ |
| Adaptadores | 80%+ |
| API endpoints | 90%+ |
| **Global** | **85%+** |
