# icons

Tauri는 번들 빌드 시 아이콘 파일을 요구한다. 개발 (`tauri dev`) 단계에서는 필요 없으나,
릴리스 빌드 (`tauri build`) 전에는 다음 파일을 채워야 한다.

- `32x32.png`
- `128x128.png`
- `128x128@2x.png`
- `icon.icns` (macOS)
- `icon.ico` (Windows)

빠르게 만들고 싶으면 `cargo tauri icon path/to/source.png` (1024x1024 권장)로 자동 생성한다.
