# Empacotamento Linux (x86-64) — Manga Archive Manager

## Formatos
- **AppImage** (prioritário): `MAM-linux-x86_64.AppImage`
- **TAR.GZ** (alternativo): `MAM-linux-x86_64.tar.gz`

## Dependências de Sistema (Build / Execução Tauri 2 no Linux)
- `libwebkit2gtk-4.1-0` (ou pacote equivalente da distribuição)
- `libgtk-3-0`, `libayatana-appindicator3-1`, `librsvg2-2`
- `appimagetool` (opcional para gerar o `.AppImage` final a partir do `AppDir`)

## Permissões
- Conceder permissão de execução antes de rodar o AppImage ou o script:
  ```bash
  chmod +x packaging/linux/build-portable-linux.sh
  chmod +x MAM-linux-x86_64.AppImage
  ```
