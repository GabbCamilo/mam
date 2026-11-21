# Empacotamento Linux (x86-64) — Manga Archive Manager

## Formatos
- **AppImage** (prioritário): `MAM-linux-x86_64.AppImage` (usa `MAM_LIBRARY` ao lado do `.AppImage` se a pasta existir, ou `~/.local/share/` por padrão)
- **TAR.GZ completo**: `MAM-linux-x86_64.tar.gz`
- **TAR.GZ atualização segura**: `MAM-linux-x86_64-update.tar.gz` (contém apenas `app`, `ai-engine/` e `connectors/` — **sem `MAM_LIBRARY`**, seguro para extrair sobre instalações existentes)

## Dependências de Sistema (Build / Execução Tauri 2 no Linux)
- `libwebkit2gtk-4.1-0` (ou pacote equivalente da distribuição)
- `libgtk-3-0`, `libayatana-appindicator3-1`, `librsvg2-2`
- `python3` (para recursos do `ai-engine` / Leitor Web CDP)
- `google-chrome` ou `chromium` (opcional, utilizado pelo Leitor Web Inteligente CDP para sites SPA)
- `appimagetool` (opcional para gerar o `.AppImage` final a partir do `AppDir`)

## Permissões
- Conceder permissão de execução antes de rodar o AppImage ou o script:
  ```bash
  chmod +x packaging/linux/build-portable-linux.sh
  chmod +x MAM-linux-x86_64.AppImage
  ```
