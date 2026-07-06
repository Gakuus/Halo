# ADR-008: WebRTC Data Channels para comunicación P2P

**Estado:** Aceptado

**Contexto:** Necesitamos un transporte P2P que permita comunicación directa entre clientes, con cifrado nativo y soporte para NAT traversal.

**Decisión:** WebRTC Data Channels (SCTP sobre ICE).

**Opciones consideradas:**
- **WebSocket + servidor relay:** El servidor ve todo el tráfico (aunque cifrado E2EE), punto central de fallo
- **WebRTC Data Channels:** P2P real, cifrado nativo DTLS, NAT traversal vía ICE/STUN/TURN
- **libp2p:** Más complejo, menos maduro en Rust

**Consecuencias:**
- Comunicación directa entre peers (sin servidor intermediario)
- Cifrado DTLS nativo en el canal (capa adicional a E2EE)
- NAT traversal automático (ICE + STUN + TURN)
- Soporte para Data Channels confiables (SCTP)
- Grupo mesh requiere (N-1) conexiones por cliente
- webrtc-rs crate para implementación en Rust
