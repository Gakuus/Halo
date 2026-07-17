# Glosario

| Término | Definición |
|---------|------------|
| **Access Token** | JWT de corta duración (15 min) que autentica al usuario ante el servidor |
| **ADR** | Architecture Decision Record. Documento que registra una decisión arquitectónica y su contexto |
| **Axum** | Framework web en Rust usado para el servidor HTTP/WS |
| **Chain Key** | Clave simétrica derivada del ratchet que cifra cada mensaje |
| **Conversación** | Canal de chat 1:1 entre dos usuarios |
| **CSPRNG** | Cryptographically Secure Pseudo-Random Number Generator |
| **Cursor** | Identificador del último elemento en una lista paginada (paginación cursor-based) |
| **Double Ratchet** | Protocolo criptográfico que deriva nuevas claves por cada mensaje, garantizando forward secrecy |
| **DTLS** | Datagram TLS. Capa de cifrado para WebRTC |
| **E2EE** | End-to-End Encryption. Cifrado que solo el emisor y receptor pueden descifrar |
| **Ed25519** | Algoritmo de firma digital usado para identity keys |
| **Forward Secrecy** | Propiedad: si una clave a largo plazo se compromete, los mensajes anteriores no se pueden descifrar |
| **Grupo** | Canal de chat multi-usuario (hasta 50 miembros) |
| **HS256** | HMAC-SHA256. Algoritmo de firma simétrica para JWT |
| **Identity Key** | Par de claves Ed25519 que identifica al usuario. No rota |
| **JWT** | JSON Web Token. Formato de token de autenticación |
| **Keychain** | Almacén seguro del SO (macOS Keychain, Linux Secret Service, Windows DPAPI) |
| **Nest** | Método de Axum para anidar routers bajo un prefijo de ruta |
| **One-Time Pre-Key** | Clave X25519 de un solo uso para el intercambio inicial X3DH |
| **PFS** | Perfect Forward Secrecy. Ver Forward Secrecy |
| **PGP** | Pretty Good Privacy. Estándar de cifrado asimétrico |
| **Pre-Key** | Clave pública publicada en el servidor para permitir el intercambio inicial |
| **Refresh Token** | JWT de larga duración (7 días) usado para renovar el access token |
| **SAST** | Static Application Security Testing. Análisis estático de seguridad |
| **SDP** | Session Description Protocol. Formato de negociación de WebRTC |
| **Señalización** | Intercambio inicial de metadata (SDP, ICE candidates) para establecer conexión P2P |
| **Session** | Sesión de usuario en el servidor. Asociada a un JWT y una conexión WS |
| **Signed Pre-Key** | Pre-key firmada con la identity key para evitar suplantación |
| **SRTP** | Secure Real-time Transport Protocol. Cifrado de audio/video en WebRTC |
| **STUN** | Session Traversal Utilities for NAT. Servidor que ayuda a descubrir la IP pública |
| **Tauri** | Framework para apps desktop nativas con frontend web |
| **Threat Model** | Análisis sistemático de amenazas, activos y atacantes |
| **TURN** | Traversal Using Relays around NAT. Servidor que relayea tráfico cuando P2P no es posible |
| **VO** | Value Object. Objeto inmutable del dominio sin identidad propia |
| **WebRTC** | Protocolo para comunicación P2P en tiempo real (audio, video, datos) |
| **WS** | WebSocket. Protocolo de comunicación bidireccional sobre TCP |
| **X25519** | Curva elíptica Diffie-Hellman usada para intercambio de claves |
| **X3DH** | Extended Triple Diffie-Hellman. Protocolo de intercambio de claves inicial para E2EE |
| **XChaCha20-Poly1305** | Cifrado autenticado (AEAD) usado para cifrar mensajes |
