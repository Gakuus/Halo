# Template de Notificación de Incidente de Seguridad

## Uso

Este template se usa para notificar a usuarios y/o autoridades en caso de una brecha de seguridad. Completar los campos entre `{{ }}`.

---

## Asunto: [CRITICAL/HIGH] Notificación de incidente de seguridad — Halo

### Versión para usuarios

---

**Fecha:** {{ fecha }}

Estimado usuario de Halo,

Hemos detectado un incidente de seguridad que {{ puede haber / ha }} comprometido tus datos.

### ¿Qué pasó?

{{ descripción clara y honesta del incidente, sin jerga técnica }}

### ¿Qué datos están involucrados?

- {{ tipo de datos expuestos: ej. "tu dirección de email, username y hash de contraseña" }}
- {{ aclarar qué NO fue expuesto: ej. "tus mensajes no fueron comprometidos porque están cifrados de extremo a extremo" }}
- {{ si no hay certeza, decirlo }}

### ¿Qué hicimos?

- {{ acción inmediata tomada: ej. "rotamos la clave de firma JWT y bloqueamos el acceso externo a la base de datos" }}
- {{ parche aplicado }}

### ¿Qué necesitas hacer?

- [ ] Cambiar tu contraseña en [enlace]
- [ ] Verificar actividad reciente en tu cuenta
- [ ] {{ si aplica: reinstalar la app para obtener nueva identity key }}

### ¿Necesitas ayuda?

Si tenés preguntas o notaste actividad sospechosa, contactanos en {{ email/soporte }}.

---

### Versión para autoridades (GDPR)

---

**Notificación de violación de datos personales**

**Fecha del incidente:** {{ fecha }}
**Fecha de esta notificación:** {{ fecha }}

**Responsable del tratamiento:**
- Nombre: {{ nombre }}
- Email: {{ email }}
- Teléfono: {{ teléfono }}

**Naturaleza de la violación:**
- {{ descripción técnica }}
- Categorías de datos afectados: {{ categorías }}
- Número aproximado de usuarios afectados: {{ número }}
- Volumen de registros afectados: {{ volumen }}

**Consecuencias probables:**
- {{ riesgos para los derechos y libertades de los usuarios }}

**Medidas tomadas o propuestas:**
- {{ medidas correctivas aplicadas }}
- {{ medidas para mitigar daños potenciales }}

**Datos del DPO (si aplica):**
- Nombre: {{ nombre }}
- Email: {{ email }}

---

## Registro interno

- **Ticket de incidente:** {{ link }}
- **Severidad:** {{ CRITICAL / HIGH }}
- **Causa raíz:** {{ descripción }}
- **Lecciones aprendidas:** {{ link a post-mortem }}
- **Notificado a usuarios:** {{ fecha y medio }}
- **Notificado a autoridades:** {{ fecha y medio }}
