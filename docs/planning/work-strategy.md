# Estrategia de Trabajo

## Orden de desarrollo

El orden establecido NO debe modificarse sin justificación documentada.

```
 1. Documentación       ← Estamos aquí
 2. Infraestructura     ← Configuración del proyecto Rust, BD, CI
 3. Dominio             ← Entidades, VO, reglas de negocio
 4. Puertos             ← Traits/interfaces
 5. Casos de uso        ← Lógica de aplicación
 6. Adaptadores DB      ← Repositorios PostgreSQL
 7. API REST            ← Handlers HTTP
 8. WebSocket + Señalización
 9. Cliente Tauri       ← Frontend nativo
10. E2EE                ← Cifrado extremo a extremo
11. Testing             ← Completar cobertura
12. CI/CD               ← Despliegue automatizado
```

## Justificación del orden

| Posición | Componente | ¿Por qué aquí? |
|----------|-----------|----------------|
| 1 | Documentación | Sin diseño claro, el código es caótico |
| 2 | Infraestructura | Sin proyecto funcionando no se puede desarrollar |
| 3 | Dominio | Es el núcleo, todo lo demás depende de él |
| 4 | Puertos | Define contratos antes de implementar |
| 5 | Casos de uso | Orquesta el dominio usando puertos |
| 6 | Adaptadores DB | Primer adaptador concreto |
| 7 | API REST | Expone casos de uso al mundo exterior |
| 8 | WS + Señalización | Comunicación en tiempo real |
| 9 | Cliente Tauri | Requiere servidor funcionando |
| 10 | E2EE | Requiere cliente funcional para probar |
| 11 | Testing | Sistema completo para testear |
| 12 | CI/CD | Código listo para automatizar |

## Principios de trabajo

### 1. Una cosa a la vez
Cada tarea se completa antes de pasar a la siguiente. No se salta una tarea incompleta.

### 2. Tests primero (cuando aplique)
Los casos de uso y el dominio se testean antes o durante su implementación. Los adaptadores pueden testearse después.

### 3. Commits atómicos
Cada commit representa un cambio completo y funcional. No commits "work in progress" en ramas principales.

### 4. Documentación sincronizada
Si el código cambia la arquitectura, la documentación se actualiza inmediatamente.

### 5. Código > Comentarios
El código debe ser autoexplicativo. Los comentarios se reservan para "por qué" no para "qué".

## Ciclo de desarrollo por tarea

1. **Revisar** la documentación relevante
2. **Diseñar** la solución (si aplica, actualizar docs)
3. **Implementar** siguiendo las convenciones
4. **Testear** (unitario + integración si aplica)
5. **Compilar** (cargo build --no warnings)
6. **Commit** (conventional commit)
7. **Mover** la tarea a completada
