# Manga Archive Manager (MAM)

Aplicação desktop multiplataforma (Windows e Linux, **x86-64**) para gerenciamento, arquivamento, organização, leitura, transferência e processamento inteligente de mangás e quadrinhos digitais — projetada para ser **100% portátil em SSD externo**, sem depender de servidores ou bancos remotos.

---

## Stack Tecnológica

- **Desktop**: Tauri 2
- **Frontend**: React + TypeScript (`backend/frontend/`)
- **Backend**: Rust workspace modular (`backend/`)
- **Banco de Dados**: SQLite local (`System/database.db` com caminhos relativos)
- **AI Engine**: Python 3.11+ (`ai-engine/` via JSON-RPC 2.0 sobre stdio)
- **Inferência Local**: ONNX Runtime (prioritário) / PyTorch (complementar), CPU obrigatório, GPU opcional

---

## Como Usar (Passo a Passo)

### 1. Executando via Código-Fonte (Desenvolvimento)

#### Pré-requisitos
- **Rust** (`rustup` / `cargo`) — no Windows com toolchain GNU + MSYS2 GCC (`C:\msys64\mingw64\bin` no `PATH`) ou MSVC + Windows SDK
- **Node.js** + `npm`
- **Python 3.11+** (com `pillow` e `numpy`; opcionalmente `onnxruntime` para inferência `.onnx`)

#### Passo a passo
```powershell
# 1. Clonar o repositório
git clone https://github.com/GabbCamilo/mam.git
cd mam

# 2. Instalar dependências e compilar o frontend
cd backend/frontend
npm install
npm run build

# 3. Iniciar a aplicação desktop Tauri
cd ..
# No Windows (caso utilize MSYS2 mingw64):
$env:PATH = "C:\msys64\mingw64\bin;$env:PATH"
cargo run -p app
```

---

### 2. Usando em Modo Portátil (SSD Externo / Pendrive)

#### Gerando o pacote portátil (Windows x86-64)
```powershell
cd mam
# No Windows (caso utilize MSYS2 mingw64):
$env:PATH = "C:\msys64\mingw64\bin;$env:PATH"
.\packaging\windows\build-portable-zip.ps1
```
O pacote será gerado em `dist\MAM-windows-x86_64.zip` contendo `app.exe`, `ai-engine/` e a pasta `models/` vazia.
*(Opcional: passe `-PythonRuntimeDir "C:\caminho\python-embed"` para embutir um runtime Python portátil dentro do `.zip`.)*

Para Linux x86-64 (AppImage e TAR.GZ), consulte [packaging/README.md](packaging/README.md) e execute `./packaging/linux/build-portable-linux.sh`.

#### Executando a partir de um SSD Externo
1. Extraia `MAM-windows-x86_64.zip` no seu SSD externo.
2. O programa resolve a raiz da biblioteca na seguinte ordem:
   1. Variável de ambiente **`MAM_LIBRARY_DIR`** (se definida);
   2. Pasta **`MAM_LIBRARY/`** ou **`System/database.db`** ao lado do executável `app.exe`;
   3. Diretório de dados local do sistema (`app_data_dir`) como fallback.
3. Exemplo apontando para uma biblioteca em qualquer unidade:
   ```powershell
   $env:MAM_LIBRARY_DIR = "E:\MinhaBibliotecaManga"
   .\app.exe
   ```
4. **Transferindo apenas sua biblioteca (sem o programa)**: todos os caminhos gravados no SQLite são **relativos à raiz**. Basta copiar a pasta da biblioteca (`System/database.db` + arquivos `.cbz`) para outro SSD ou computador e apontar `MAM_LIBRARY_DIR` para ela.

---

### 3. Guia das Funcionalidades na Interface

1. **Aba `Biblioteca`**:
   - Cadastre obras informando título e caminho relativo (ex.: `Manga/Minha Obra`).
   - Clique em **Detectar Capítulos CBZ** para varrer arquivos `.cbz` existentes na raiz da biblioteca.
   - Selecione uma obra para ver volumes e capítulos avulsos e clique para abrir no **Leitor CBZ**.
2. **Leitor CBZ (`CbzReader`)**:
   - Navegue com os botões ou setas do teclado (`←` / `→`) entre páginas e capítulos consecutivos.
   - Alterne entre os modos **Horizontal**, **Vertical** e **Página Dupla**, ajuste **Zoom** e **Ajuste à Tela**.
   - Salve **Favoritos**, **Marcadores** e retome automaticamente da última página lida.
   - Use **Processar Página (IA)**, **Sobreposição: ON/OFF** e **Camada IA / Revisão** para executar o AI Engine local e revisar traduções por região (`nao_revisado` → `revisado`) sem alterar o `.cbz` original.
3. **Aba `Fontes & Importação`**:
   - **Busca integrada**: pesquise obras por título no **MangaDex (API Pública)**, marque capítulos e clique em **Arquivar Selecionados**.
   - **Importar via Extrator Modular**: informe título da obra, nº do capítulo e um link (`mangadex.org/chapter/<id>`, `mangafire.to/...`) ou HTML / lista ordenada de URLs de imagens (uma por linha). O MAM baixa em sequência, gera o `.cbz` atomicamente com SHA-256 e disponibiliza na **Biblioteca**.
4. **Aba `Transferência & Sync`**:
   - Informe origem e destino para **Comparar Bibliotecas**, **Sincronizar (Sem Sobrescrita)** ou fazer **Cópia Integral (SHA-256)** — arquivos da origem nunca são excluídos automaticamente.
5. **Aba `AI Engine`**:
   - Verifique a saúde do processo auxiliar Python (`ping` JSON-RPC 2.0 stdio) e inspecione modelos `.onnx`/`.pt` instalados localmente em um diretório (sem downloads automáticos).

---

## Testes Automatizados

```powershell
# Backend Rust (workspace completo)
cd backend
cargo test --workspace

# AI Engine Python
cd ai-engine
python -m unittest discover -s tests -v

# Frontend
cd backend/frontend
npm run build
```

---

## Metodologia e Documentação

O projeto segue **OpenSpec (Spec-Driven Development)** com schema `spec-driven` em `openspec/`. Consulte `docs/` para visão do produto, arquitetura, roadmap e glossário.

## Licença

Privado — uso interno.
