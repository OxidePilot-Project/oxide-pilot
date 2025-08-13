Coloca aquí los iconos generados por `create_icon.py` o por tu pipeline de diseño.

Flujo recomendado:
- Provee `src-tauri/icon.png` (1024x1024, fondo transparente si es posible).
- Ejecuta `python src-tauri/create_icon.py` o usa `scripts/build-windows.ps1` que lo hará automáticamente si detecta `icon.png`.
- Tauri tomará los iconos de `src-tauri/icons/` al empaquetar.
