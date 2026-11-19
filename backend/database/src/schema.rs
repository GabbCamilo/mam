//! Definições SQL do schema do banco de dados MAM.
//! Todas as tabelas armazenam caminhos relativos à raiz da biblioteca.

pub const SCHEMA_SQL: &str = r#"
-- Controle de versão do schema (migrações)
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    aplicado_em TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Obras da biblioteca
CREATE TABLE IF NOT EXISTS obras (
    id TEXT PRIMARY KEY,
    titulo TEXT NOT NULL,
    caminho_relativo TEXT NOT NULL,
    origem TEXT,
    data_arquivamento TEXT NOT NULL DEFAULT (datetime('now')),
    estado_integridade TEXT NOT NULL DEFAULT 'desconhecido',
    metadados_complementares TEXT, -- JSON
    criado_em TEXT NOT NULL DEFAULT (datetime('now')),
    atualizado_em TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Volumes de uma obra
CREATE TABLE IF NOT EXISTS volumes (
    id TEXT PRIMARY KEY,
    obra_id TEXT NOT NULL,
    numero INTEGER NOT NULL,
    caminho_relativo TEXT NOT NULL,
    titulo TEXT,
    FOREIGN KEY (obra_id) REFERENCES obras(id) ON DELETE CASCADE
);

-- Capítulos de um volume
CREATE TABLE IF NOT EXISTS capitulos (
    id TEXT PRIMARY KEY,
    volume_id TEXT,
    obra_id TEXT NOT NULL,
    numero INTEGER NOT NULL,
    titulo_original TEXT,
    caminho_relativo TEXT NOT NULL,
    quantidade_paginas INTEGER NOT NULL DEFAULT 0,
    data_arquivamento TEXT NOT NULL DEFAULT (datetime('now')),
    estado_integridade TEXT NOT NULL DEFAULT 'desconhecido',
    FOREIGN KEY (obra_id) REFERENCES obras(id) ON DELETE CASCADE,
    FOREIGN KEY (volume_id) REFERENCES volumes(id) ON DELETE SET NULL
);

-- Metadados detalhados do capítulo (páginas, origem, etc.)
CREATE TABLE IF NOT EXISTS metadados_capitulo (
    capitulo_id TEXT PRIMARY KEY,
    paginas_json TEXT NOT NULL DEFAULT '[]', -- JSON array de páginas
    origem_conteudo TEXT,
    data_arquivamento TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (capitulo_id) REFERENCES capitulos(id) ON DELETE CASCADE
);

-- Configurações da biblioteca (chave-valor)
CREATE TABLE IF NOT EXISTS configuracao_biblioteca (
    chave TEXT PRIMARY KEY,
    valor TEXT NOT NULL,
    atualizado_em TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Histórico de operações (arquivamento, transferência, sincronização)
CREATE TABLE IF NOT EXISTS historico_operacoes (
    id TEXT PRIMARY KEY,
    tipo TEXT NOT NULL,
    alvo_caminho TEXT,
    detalhes_json TEXT, -- JSON
    usuario TEXT,
    criado_em TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Estado de integridade de arquivos (hash SHA-256)
CREATE TABLE IF NOT EXISTS integridade_arquivo (
    caminho_relativo TEXT PRIMARY KEY,
    hash_sha256 TEXT NOT NULL,
    estado TEXT NOT NULL DEFAULT 'desconhecido',
    verificado_em TEXT,
    FOREIGN KEY (caminho_relativo) REFERENCES capitulos(caminho_relativo)
);

-- Referências a resultados de IA (OCR, balões, tradução)
CREATE TABLE IF NOT EXISTS resultados_ia (
    id TEXT PRIMARY KEY,
    capitulo_id TEXT NOT NULL,
    pagina_numero INTEGER NOT NULL,
    modelo_id TEXT NOT NULL,
    versao_modelo TEXT NOT NULL,
    tipo_processamento TEXT NOT NULL, -- ocr, bubbles, translation
    resultado_json TEXT NOT NULL, -- JSON com coordenadas, texto, idioma, estado revisao
    idioma_origem TEXT,
    idioma_destino TEXT,
    estado_revisao TEXT NOT NULL DEFAULT 'nao_revisado',
    criado_em TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (capitulo_id) REFERENCES capitulos(id) ON DELETE CASCADE
);

-- Índices para consultas frequentes
CREATE INDEX IF NOT EXISTS idx_capitulos_obra ON capitulos(obra_id);
CREATE INDEX IF NOT EXISTS idx_capitulos_volume ON capitulos(volume_id);
CREATE INDEX IF NOT EXISTS idx_volumes_obra ON volumes(obra_id);
CREATE INDEX IF NOT EXISTS idx_resultados_ia_capitulo ON resultados_ia(capitulo_id);
CREATE INDEX IF NOT EXISTS idx_historico_tipo ON historico_operacoes(tipo);
"#;
