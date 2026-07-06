# ADR-007: Tauri para el cliente de escritorio

**Estado:** Aceptado

**Contexto:** Necesitamos un cliente nativo que soporte WebRTC mesh, almacenamiento local del historial, keychain para claves E2EE y operación en segundo plano.

**Decisión:** Tauri (solo Linux).

**Opciones consideradas:**
- **Electron:** Pesado (Chromium + Node), mayor consumo de memoria
- **Web (PWA):** Sin acceso a keychain, sin operación background robusta, límite de conexiones WebRTC
- **Tauri:** Nativo (Rust), liviano, acceso completo al SO, mismo lenguaje que el backend

**Consecuencias:**
- Binarios pequeños (~5MB vs ~150MB de Electron)
- Acceso a keychain del SO para claves E2EE
- Almacenamiento en disco real para historial
- Operación en system tray / background
- WebRTC nativo vía webrtc-rs
- Se puede compartir lógica Rust con el backend (modelos de dominio, E2EE)
