-- Migração 002: Adiciona coluna idioma à tabela capitulos
ALTER TABLE capitulos ADD COLUMN idioma TEXT;
CREATE INDEX IF NOT EXISTS idx_capitulos_idioma ON capitulos(obra_id, idioma);
