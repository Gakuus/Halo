# Git Flow

## Estructura de ramas

```
main
  │
  ├── develop
  │     │
  │     ├── feature/login
  │     ├── feature/websocket
  │     │
  │     └── release/v1.0.0
  │           │
  │           └── main (merge)
  │
  └── hotfix/critical-bug
        │
        └── main (merge directo)
```

---

## Descripción de ramas

### `main`
- Código en producción
- Siempre estable
- Solo se mergea desde `release/*` o `hotfix/*`
- Cada merge = nuevo release (tag)

### `develop`
- Rama de integración
- Contiene la última funcionalidad completa
- Las features se mergean aquí
- Debe pasar tests antes de mergear

### `feature/*`
- Nueva funcionalidad
- Se crea desde `develop`
- Se mergea a `develop` cuando está completa
- Naming: `feature/<id>-<descripcion>` ej: `feature/12-add-login`

### `fix/*`
- Corrección de bug en desarrollo
- Se crea desde `develop`
- Se mergea a `develop`
- Naming: `fix/<id>-<descripcion>` ej: `fix/15-fix-websocket-reconnect`

### `hotfix/*`
- Corrección urgente en producción
- Se crea desde `main`
- Se mergea a `main` Y a `develop`
- Naming: `hotfix/<id>-<descripcion>` ej: `hotfix/18-security-vuln`

### `release/*`
- Preparación de release
- Se crea desde `develop`
- Solo se permiten bug fixes, no features nuevas
- Al finalizar, se mergea a `main` y a `develop`
- Naming: `release/v1.0.0`

---

## Flujo de trabajo

### 1. Iniciar nueva feature

```bash
git checkout develop
git pull origin develop
git checkout -b feature/12-add-login
```

### 2. Trabajar en la feature

```bash
# Commits frecuentes
git add .
git commit -m "feat(auth): add login endpoint"
git push origin feature/12-add-login
```

### 3. Completar la feature

```bash
# Asegurar que develop está actualizado
git checkout develop
git pull origin develop
git merge feature/12-add-login --squash
git commit -m "feat(auth): add login endpoint (#12)"
git push origin develop

# Borrar rama
git branch -d feature/12-add-login
```

### 4. Release

```bash
git checkout develop
git pull origin develop
git checkout -b release/v1.0.0

# Bug fixes menores aquí
git commit -m "fix: minor release fixes"

# Merge a main
git checkout main
git merge release/v1.0.0
git tag v1.0.0
git push origin main --tags

# Merge de vuelta a develop
git checkout develop
git merge release/v1.0.0
git push origin develop

git branch -d release/v1.0.0
```

### 5. Hotfix

```bash
git checkout main
git checkout -b hotfix/18-security-vuln

git commit -m "fix(sec): patch security vulnerability (#18)"

git checkout main
git merge hotfix/18-security-vuln
git tag v1.0.1
git push origin main --tags

git checkout develop
git merge hotfix/18-security-vuln
git push origin develop

git branch -d hotfix/18-security-vuln
```

---

## Política de commits

- Commits atómicos: un cambio lógico por commit
- Mensajes en inglés (para estándar internacional)
- Usar Conventional Commits (ver convenciones)
- Un feature completo = un commit (squash al mergear)

## Política de PRs

- Toda merge a `main` o `develop` requiere PR
- El PR debe pasar CI (build + test + lint)
- Al menos 1 approval (o auto-approval en proyecto unipersonal)
- El PR debe incluir tests si aplica
- El título del PR debe ser descriptivo
- Squash al mergear (commits atómicos en develop)

## Tags

- Tags semánticos: `v1.0.0`, `v1.0.1`, `v1.1.0`
- Tags firmados con GPG (si configurado)
- Changelog en GitHub Releases
