# Protocolo E2EE — Halo

## 1. Resumen

Halo implementa un modelo híbrido:
- **Conversaciones 1:1**: Signal Protocol (X3DH + Double Ratchet)
- **Grupos**: Sender Keys (cada miembro tiene una clave de envío para el grupo)
- **Seguridad adicional**: Safety Numbers para verificación, contadores de secuencia contra replay

## 2. Conversaciones 1:1 — Signal Protocol

### 2.1 X3DH (Intercambio inicial)

```
Alice                              Bob (ya registrado)
  │                                    │
  │  GET /users/bob/pre-keys           │
  │───────────────────────────────────>│
  │                                    │
  │  {                                 │
  │    identity_key: Ed25519Pub,       │
  │    signed_pre_key: X25519Pub,      │
  │    signature: Ed25519Sig,          │
  │    one_time_pre_keys: [X25519Pub]  │
  │  }                                 │
  │<───────────────────────────────────│
  │                                    │
  │  Alice genera:                      │
  │  - ephemeral_key: X25519KeyPair     │
  │  - shared_secret = SK(              │
  │      identity_priv || ephemeral,    │
  │      bob_identity_pub ||            │
  │      bob_signed_pre_key_pub,        │
  │      ephemeral || bob_signed_pre,   │
  │      ephemeral || bob_one_time      │
  │    )                                │
  │                                    │
  │  Alice envía mensaje inicial:       │
  │  { identity_key, ephemeral_key,     │
  │    used_one_time_pre_key_index,     │
  │    ciphertext(shared_secret, msg) } │
  │───────────────────────────────────>│
  │                                    │
  │  Bob deriva shared_secret y         │
  │  descifra el mensaje inicial        │
```

### 2.2 Double Ratchet

Después de X3DH, ambos lados derivan las primeras chain keys. Cada mensaje:

1. Emisor genera nuevo ratchet key pair efímero
2. Deriva nueva chain key y clave de cifrado (HKDF)
3. Cifra con XChaCha20-Poly1305
4. Envía: `{ ratchet_public_key, sequence_number, iv, salt, ciphertext, signature }`

Esto garantiza:
- **Forward secrecy**: comprometer una clave de ratchet no descifra mensajes anteriores
- **Post-compromise security**: un mensaje honesto después de un compromiso recupera la seguridad

## 3. Grupos — Sender Keys

Para grupos con 3+ miembros, usar X3DH+Double Ratchet para cada par es O(n²). Se usa **Sender Keys**:

### 3.1 Setup inicial

Cuando Alice crea un grupo con Bob y Charlie:

```
Alice:                                    │
  1. Genera SenderKey (X25519)            │
  2. Para cada miembro:                   │
     a. Inicia session X3DH+Double Ratchet│
     b. Envía SenderKey cifrada           │
        con la session 1:1                │
Alice ────[SenderKey cifrada]────> Bob    │
Alice ────[SenderKey cifrada]────> Charlie│
                                          │
Luego Alice envía al grupo:               │
  { sender_key_ephemeral,                 │
    sequence_number_in_group,             │
    ciphertext }                          │
  Todos descifran con la SenderKey        │
```

### 3.2 Envío de mensajes

1. Alice tiene una **Sender Chain** (una chain key + contador) para el grupo
2. Por cada mensaje: deriva clave de cifrado del ratchet, incrementa contador
3. Firma el mensaje con su identity key Ed25519 (autenticación)
4. Bob y Charlie descifran con la Sender Key compartida

### 3.3 Seguridad en Sender Keys

| Propiedad | Cómo se logra |
|-----------|---------------|
| **Autenticación** | El mensaje incluye firma Ed25519 del emisor |
| **Forward secrecy** | Ratchet sobre la sender chain |
| **Post-compromise** | Rotación periódica de Sender Key (futuro) |
| **Deniability** | El receptor puede forjar el mensaje porque conoce la Sender Key (no puede probar quién lo envió ante un tercero) |

### 3.4 Nuevos miembros

Cuando se agrega un nuevo miembro al grupo:
1. Un miembro existente (con privilegios) inicia session 1:1 con el nuevo
2. Le envía la Sender Key actual cifrada
3. El nuevo miembro puede descifrar mensajes futuros (no los pasados)

### 3.5 Expulsión de miembros y rotación de Sender Key

Si un miembro es removido:
1. Un admin genera **nueva Sender Key**
2. La distribuye cifrada a cada miembro restante (vía sessions 1:1)
3. El miembro expulsado no puede descifrar mensajes futuros

## 4. Safety Numbers (Verificación de claves)

### 4.1 Propósito

Prevenir ataques MITM donde el servidor (o un atacante) presente claves públicas falsas. Los Safety Numbers permiten que los usuarios verifiquen fuera de banda que están hablando con la persona correcta.

### 4.2 Cálculo

```
safety_number = SHA-512(
  "Halo Safety Number v1" ||
  alice_identity_key_public ||
  bob_identity_key_public
)
```

Se muestran como dos grupos de 30 dígitos base-10 (formateados para comparación visual).

El orden de las claves es el mismo para ambos participantes (lexicográfico), de modo que ambos ven el mismo número.

### 4.3 Flujo de verificación

```
Alice                                            Bob
  │                                               │
  │  1. Alice ve Safety Number en pantalla        │
  │     basado en su clave + clave de Bob         │
  │                                               │
  │  2. Comparan fuera de banda:                  │
  │     - En persona: "leé los primeros 6 dígitos"│
  │     - QR: Alice escanea QR de Bob             │
  │     - Audio: frase codificada                 │
  │                                               │
  │  3. Alice marca a Bob como "verificado"       │
  │     (opcional: futuras alerts si cambia)      │
```

### 4.4 Cambio de claves

Si la identity key de un usuario cambia (nuevo dispositivo):
- Los Safety Numbers cambian
- Los contactos con los que tenía sesión previa reciben una notificación: **"La identity key de Bob ha cambiado. ¿Confirmás que es Bob?"**
- La vieja session 1:1 se marca para downgrade si no se verifica la nueva

### 4.5 Seguridad

- **No hacer**: aceptar cambios de key silenciosamente (vulnerable a MITM persistente)
- **Sí hacer**: alertar al usuario y requerir confirmación
- **Opción**: registrar la primera key vista como "trust on first use" (TOFU), con alertas si cambia

## 5. Replay Protection

### 5.1 Contadores de secuencia

Cada mensaje incluye un `sender_sequence_number: u64` que es un contador **monotónico estrictamente creciente** por remitente.

```
┌──────────────────────────────────┐
│ Message                          │
│──────────────────────────────────│
│ id: MessageId                    │
│ conversation_id: ConversationId  │
│ sender_id: UserId                │
│ sender_sequence_number: u64      │ ← NUEVO
│ ciphertext: Ciphertext           │
│ iv: Iv, salt: Salt               │
│ signature: Signature             │
│ status: MessageStatus            │
│ timestamp: Timestamp             │
│ reply_to: Option<MessageId>      │
└──────────────────────────────────┘
```

### 5.2 Validación

El receptor mantiene un mapa: `(conversation_id, sender_id) -> last_sequence_number`.

Al recibir un mensaje:
1. Extraer `sender_sequence_number` del mensaje (después de descifrar)
2. Obtener `last_seq` del mapa local
3. Si `sender_sequence_number <= last_seq` → **RECHAZAR** (posible replay)
4. Si `sender_sequence_number > last_seq` → **ACEPTAR**, actualizar `last_seq`

### 5.3 Persistencia

- El `last_sequence_number` se almacena localmente en el cliente (keychain o BD cifrada)
- Al restablecer una sesión 1:1 (nuevo dispositivo), el contador se resetea (es aceptable porque X3DH produce nuevo shared secret)
- En grupos: cada miembro mantiene el contador por cada otro miembro del grupo

### 5.4 Manejo de gaps

Si el receptor recibe `seq=5` y nunca vio `seq=4`:
- Opción A: aceptar (puede perderse por ordering de red)
- Opción B: solicitar reenvío (más seguro pero más complejo)
- **Halo**: Opción A con límite máximo de gap (ej: tolerar gap hasta 100 mensajes)

## 6. Consideraciones adicionales

### 6.1 Firmas en todos los mensajes

Cada mensaje (1:1 y grupo) va firmado con Ed25519 del remitente:
- 1:1: firma sobre `ciphertext || iv || salt || sender_sequence_number`
- Grupo: firma sobre `ciphertext || iv || salt || sender_sequence_number || group_id`

Esto evita que un atacante (o el servidor) modifique mensajes en tránsito.

### 6.2 Almacenamiento de claves en el cliente

| Clave | Store | Backup |
|-------|-------|--------|
| Identity Key (privada) | Keychain SO | No |
| Sender Keys (grupos) | Keychain SO cifrado | No |
| Session state (ratchet) | Keychain SO cifrado | Exportable por el usuario |
| Contadores de secuencia | Keychain SO / BD cifrada | Incluido en export |

### 6.3 Rotación de claves

| Evento | Acción |
|--------|--------|
| Nuevo dispositivo | Nueva Identity Key. Notificar a contactos. |
| Miembro removido de grupo | Rotar Sender Key del grupo |
| 30 días sin rotación de ratchet | Forzar rotación (seguridad) |
| Safety number no coincide | Alertar y requerir verificación manual |

## 7. Referencias

- [Signal Protocol](https://signal.org/docs/)
- [X3DH](https://signal.org/docs/specifications/x3dh/)
- [Double Ratchet](https://signal.org/docs/specifications/doubleratchet/)
- [Sender Keys (Matrix)](https://spec.matrix.org/v1.13/client-server-api/#end-to-end-encryption)
- [ADR 009: E2EE](../adr/009-e2ee.md)
