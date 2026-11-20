#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BACKEND_DIR="${REPO_ROOT}/backend"
FRONTEND_DIR="${BACKEND_DIR}/frontend"
AI_ENGINE_DIR="${REPO_ROOT}/ai-engine"
OUTPUT_DIR="${1:-${REPO_ROOT}/dist}"

STAGE_DIR="${OUTPUT_DIR}/MAM-linux-x86_64"
APPDIR="${OUTPUT_DIR}/MAM.AppDir"
TAR_PATH="${OUTPUT_DIR}/MAM-linux-x86_64.tar.gz"
APPIMAGE_PATH="${OUTPUT_DIR}/MAM-linux-x86_64.AppImage"

echo "==> Compilando frontend React + TypeScript..."
(cd "${FRONTEND_DIR}" && npm run build)

echo "==> Compilando backend Rust (release)..."
(cd "${BACKEND_DIR}" && cargo build --release -p app)

BIN_PATH="${BACKEND_DIR}/target/release/app"
if [[ ! -f "${BIN_PATH}" ]]; then
  echo "Erro: binário release não encontrado em ${BIN_PATH}" >&2
  exit 1
fi

echo "==> Montando pacote TAR.GZ portátil..."
rm -rf "${STAGE_DIR}"
mkdir -p "${STAGE_DIR}/ai-engine" "${STAGE_DIR}/models"
cp "${BIN_PATH}" "${STAGE_DIR}/app"
chmod +x "${STAGE_DIR}/app"
cp -r "${AI_ENGINE_DIR}/mam_ai_engine" "${STAGE_DIR}/ai-engine/"
cp "${AI_ENGINE_DIR}/requirements.txt" "${STAGE_DIR}/ai-engine/"

tar -C "${OUTPUT_DIR}" -czf "${TAR_PATH}" "MAM-linux-x86_64"
echo "==> TAR.GZ gerado: ${TAR_PATH}"

echo "==> Montando AppDir para AppImage..."
rm -rf "${APPDIR}"
mkdir -p "${APPDIR}/usr/bin" "${APPDIR}/usr/share/mam/ai-engine" "${APPDIR}/usr/share/mam/models"
cp "${BIN_PATH}" "${APPDIR}/usr/bin/mam"
chmod +x "${APPDIR}/usr/bin/mam"
cp -r "${AI_ENGINE_DIR}/mam_ai_engine" "${APPDIR}/usr/share/mam/ai-engine/"
cp "${AI_ENGINE_DIR}/requirements.txt" "${APPDIR}/usr/share/mam/ai-engine/"

cat > "${APPDIR}/AppRun" <<'EOF'
#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "${0}")")"
export MAM_AI_ENGINE_DIR="${HERE}/usr/share/mam/ai-engine"
exec "${HERE}/usr/bin/mam" "$@"
EOF
chmod +x "${APPDIR}/AppRun"

cat > "${APPDIR}/mam.desktop" <<'EOF'
[Desktop Entry]
Name=Manga Archive Manager
Exec=mam
Type=Application
Categories=Utility;
EOF

if command -v appimagetool >/dev/null 2>&1; then
  ARCH=x86_64 appimagetool "${APPDIR}" "${APPIMAGE_PATH}"
  echo "==> AppImage gerado: ${APPIMAGE_PATH}"
else
  echo "Aviso: appimagetool não encontrado no PATH; AppDir pronto em ${APPDIR}."
fi
