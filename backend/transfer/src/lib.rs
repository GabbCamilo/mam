//! # mam-transfer
//! Cópia integral, sincronização e transferência seletiva de bibliotecas.
//! Garante validação SHA-256 pós-cópia e nunca exclui arquivos da origem.

pub mod container;
pub mod copia;
pub mod sincronizacao;

use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub use container::RelatorioContainer;
pub use sincronizacao::DiferencaBibliotecas;

/// Erros do módulo de transferência e sincronização.
#[derive(Debug, Error)]
pub enum TransferError {
    #[error("Erro de E/S: {0}")]
    Io(#[from] std::io::Error),

    #[error("Erro de armazenamento: {0}")]
    Storage(String),

    #[error("Diretório de origem não encontrado: {0}")]
    OrigemNaoEncontrada(String),

    #[error("Caminho relativo inválido: {0}")]
    CaminhoInvalido(String),

    #[error("Falha na verificação de integridade SHA-256 em '{arquivo}': esperado {esperado}, obtido {obtido}")]
    IntegridadeFalhou {
        arquivo: String,
        esperado: String,
        obtido: String,
    },
}

/// Relatório estruturado de uma operação de transferência ou sincronização.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatorioTransferencia {
    pub operacao_id: String,
    pub arquivos_copiados: Vec<String>,
    pub arquivos_ignorados: Vec<String>,
    pub bytes_transferidos: u64,
}

impl RelatorioTransferencia {
    pub fn novo() -> Self {
        Self {
            operacao_id: Uuid::new_v4().to_string(),
            arquivos_copiados: Vec::new(),
            arquivos_ignorados: Vec::new(),
            bytes_transferidos: 0,
        }
    }
}

impl Default for RelatorioTransferencia {
    fn default() -> Self {
        Self::novo()
    }
}

/// Gerenciador de transferência e sincronização de bibliotecas MAM.
pub struct TransferManager;

impl TransferManager {
    pub fn nova() -> Self {
        Self
    }

    /// Copia integralmente uma biblioteca para um diretório de destino.
    pub async fn copia_integral(
        &self,
        origem: &Path,
        destino: &Path,
    ) -> Result<RelatorioTransferencia, TransferError> {
        copia::copia_integral(origem, destino)
    }

    /// Transfere seletivamente obras, volumes ou capítulos por caminho relativo.
    pub async fn transferencia_seletiva(
        &self,
        origem: &Path,
        destino: &Path,
        caminhos_relativos: &[String],
    ) -> Result<RelatorioTransferencia, TransferError> {
        copia::transferencia_seletiva(origem, destino, caminhos_relativos)
    }

    /// Compara duas bibliotecas (ou um container `.zip` com uma biblioteca) identificando ausentes, diferentes e idênticos.
    pub async fn comparar_bibliotecas(
        &self,
        origem: &Path,
        destino: &Path,
    ) -> Result<DiferencaBibliotecas, TransferError> {
        let orig = origem.to_path_buf();
        let dest = destino.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let eh_zip = orig
                .is_file()
                && orig
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e.eq_ignore_ascii_case("zip"));
            if eh_zip {
                container::comparar_container_com_biblioteca(&orig, &dest)
            } else {
                sincronizacao::comparar_bibliotecas(&orig, &dest)
            }
        })
        .await
        .map_err(|e| TransferError::Storage(format!("Falha em tarefa de comparação: {}", e)))?
    }

    /// Sincroniza duas bibliotecas (ou de um container `.zip` para biblioteca) com controle explícito de sobrescrita.
    pub async fn sincronizar_bibliotecas(
        &self,
        origem: &Path,
        destino: &Path,
        permitir_sobrescrita: bool,
    ) -> Result<RelatorioTransferencia, TransferError> {
        let orig = origem.to_path_buf();
        let dest = destino.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let eh_zip = orig
                .is_file()
                && orig
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e.eq_ignore_ascii_case("zip"));
            if eh_zip {
                container::sincronizar_container_para_biblioteca(&orig, &dest, permitir_sobrescrita)
            } else {
                sincronizacao::sincronizar_bibliotecas(&orig, &dest, permitir_sobrescrita)
            }
        })
        .await
        .map_err(|e| TransferError::Storage(format!("Falha em tarefa de sincronização: {}", e)))?
    }

    /// Comprime uma biblioteca completa em um container `.zip` nomeável para distribuição.
    pub async fn empacotar_container(
        &self,
        raiz_biblioteca: &Path,
        pasta_destino: &Path,
        nome_container: &str,
    ) -> Result<RelatorioContainer, TransferError> {
        let raiz = raiz_biblioteca.to_path_buf();
        let dest = pasta_destino.to_path_buf();
        let nome = nome_container.to_string();
        tokio::task::spawn_blocking(move || {
            container::empacotar_biblioteca_container(&raiz, &dest, &nome)
        })
        .await
        .map_err(|e| TransferError::Storage(format!("Falha em tarefa de empacotamento: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[tokio::test]
    async fn copia_integral_preserva_origem_e_valida_sha256() {
        let base = env::temp_dir().join(format!("mam_test_transfer_{}", Uuid::new_v4()));
        let origem = base.join("origem");
        let destino = base.join("destino");
        fs::create_dir_all(origem.join("Manga/Obra A")).unwrap();
        fs::write(origem.join("Manga/Obra A/Cap 01.cbz"), b"conteudo-cbz-1").unwrap();
        fs::write(origem.join("Manga/Obra A/Cap 02.cbz"), b"conteudo-cbz-2").unwrap();

        let tm = TransferManager::nova();
        let rel = tm.copia_integral(&origem, &destino).await.unwrap();

        assert_eq!(rel.arquivos_copiados.len(), 2);
        assert!(destino.join("Manga/Obra A/Cap 01.cbz").exists());
        assert!(origem.join("Manga/Obra A/Cap 01.cbz").exists());
        assert!(origem.join("Manga/Obra A/Cap 02.cbz").exists());

        let _ = fs::remove_dir_all(&base);
    }

    #[tokio::test]
    async fn transferencia_seletiva_e_validacao_caminho() {
        let base = env::temp_dir().join(format!("mam_test_seletiva_{}", Uuid::new_v4()));
        let origem = base.join("origem");
        let destino = base.join("destino");
        fs::create_dir_all(origem.join("Manga/Obra A")).unwrap();
        fs::create_dir_all(origem.join("Manga/Obra B")).unwrap();
        fs::write(origem.join("Manga/Obra A/Cap 01.cbz"), b"a1").unwrap();
        fs::write(origem.join("Manga/Obra B/Cap 01.cbz"), b"b1").unwrap();

        let tm = TransferManager::nova();
        assert!(
            tm.transferencia_seletiva(&origem, &destino, &["../fora".to_string()])
                .await
                .is_err()
        );

        let rel = tm
            .transferencia_seletiva(&origem, &destino, &["Manga/Obra A".to_string()])
            .await
            .unwrap();
        assert_eq!(rel.arquivos_copiados, vec!["Manga/Obra A/Cap 01.cbz"]);
        assert!(!destino.join("Manga/Obra B/Cap 01.cbz").exists());

        let _ = fs::remove_dir_all(&base);
    }

    #[tokio::test]
    async fn comparacao_e_sincronizacao_sem_sobrescrita_inesperada() {
        let base = env::temp_dir().join(format!("mam_test_sync_{}", Uuid::new_v4()));
        let origem = base.join("origem");
        let destino = base.join("destino");
        fs::create_dir_all(origem.join("Manga")).unwrap();
        fs::create_dir_all(destino.join("Manga")).unwrap();

        fs::write(origem.join("Manga/novo.cbz"), b"novo").unwrap();
        fs::write(origem.join("Manga/igual.cbz"), b"mesmo").unwrap();
        fs::write(destino.join("Manga/igual.cbz"), b"mesmo").unwrap();
        fs::write(origem.join("Manga/conflito.cbz"), b"versao-origem").unwrap();
        fs::write(destino.join("Manga/conflito.cbz"), b"versao-destino").unwrap();

        let tm = TransferManager::nova();
        let diff = tm.comparar_bibliotecas(&origem, &destino).await.unwrap();
        assert_eq!(diff.ausentes_no_destino, vec!["Manga/novo.cbz"]);
        assert_eq!(diff.diferentes, vec!["Manga/conflito.cbz"]);
        assert_eq!(diff.identicos, vec!["Manga/igual.cbz"]);

        let rel = tm
            .sincronizar_bibliotecas(&origem, &destino, false)
            .await
            .unwrap();
        assert_eq!(rel.arquivos_copiados, vec!["Manga/novo.cbz"]);
        assert_eq!(rel.arquivos_ignorados, vec!["Manga/conflito.cbz"]);
        assert_eq!(
            fs::read(destino.join("Manga/conflito.cbz")).unwrap(),
            b"versao-destino"
        );

        let _ = fs::remove_dir_all(&base);
    }

    #[tokio::test]
    async fn container_empacota_compara_e_sincroniza() {
        let base = env::temp_dir().join(format!("mam_test_container_{}", Uuid::new_v4()));
        let origem = base.join("bib_origem");
        let exports = base.join("exports");
        let destino = base.join("bib_destino");
        fs::create_dir_all(origem.join("Manga/Obra X")).unwrap();
        fs::create_dir_all(origem.join("System")).unwrap();
        fs::create_dir_all(destino.join("Manga/Obra X")).unwrap();

        fs::write(origem.join("Manga/Obra X/Cap_01.cbz"), b"cbz-1").unwrap();
        fs::write(origem.join("Manga/Obra X/Cap_02.cbz"), b"cbz-2").unwrap();
        fs::write(origem.join("System/database.db"), b"sqlite-db").unwrap();
        fs::write(destino.join("Manga/Obra X/Cap_01.cbz"), b"cbz-1").unwrap();

        let tm = TransferManager::nova();
        let rel_c = tm
            .empacotar_container(&origem, &exports, "ColecaoTeste")
            .await
            .unwrap();
        assert_eq!(rel_c.quantidade_arquivos, 3);
        let zip_path = std::path::PathBuf::from(&rel_c.caminho_arquivo);
        assert!(zip_path.exists());

        let diff = tm.comparar_bibliotecas(&zip_path, &destino).await.unwrap();
        assert!(diff
            .ausentes_no_destino
            .contains(&"Manga/Obra X/Cap_02.cbz".to_string()));
        assert!(diff
            .identicos
            .contains(&"Manga/Obra X/Cap_01.cbz".to_string()));

        let rel_s = tm
            .sincronizar_bibliotecas(&zip_path, &destino, false)
            .await
            .unwrap();
        assert!(rel_s
            .arquivos_copiados
            .contains(&"Manga/Obra X/Cap_02.cbz".to_string()));
        assert_eq!(
            fs::read(destino.join("Manga/Obra X/Cap_02.cbz")).unwrap(),
            b"cbz-2"
        );

        let _ = fs::remove_dir_all(&base);
    }
}
