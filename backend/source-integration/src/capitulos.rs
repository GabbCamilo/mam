//! Listagem de capítulos de uma obra em uma fonte.

use super::{SourceError, SourceIntegrationManager, CapituloBuscaResultado};

impl SourceIntegrationManager {
    /// Lista capítulos de uma obra em uma fonte específica,
    /// filtrando aqueles já arquivados (detecção de duplicata).
    pub async fn listar_capitulos_com_duplicatas(
        &self,
        fonte_id: &str,
        obra_id_fonte: &str,
    ) -> Result<Vec<CapituloInfoComStatus>, SourceError> {
        let capitulos = self.listar_capitulos(fonte_id, obra_id_fonte).await?;

        let mut resultado = Vec::new();
        for cap in capitulos {
            let ja_arquivado = if let Some(url) = &cap.url {
                self.capitulo_ja_arquivado(url).await.unwrap_or(false)
            } else {
                false
            };

            resultado.push(CapituloInfoComStatus {
                fonte_id: cap.fonte_id,
                obra_id_fonte: cap.obra_id_fonte,
                capitulo_id_fonte: cap.capitulo_id_fonte,
                numero: cap.numero,
                titulo: cap.titulo,
                url: cap.url,
                ja_arquivado,
            });
        }

        Ok(resultado)
    }

    /// Lista capítulos já arquivados para uma obra no banco local.
    pub async fn capitulos_arquivados_obra(
        &self,
        obra_titulo: &str,
    ) -> Result<Vec<String>, SourceError> {
        // Busca a obra no banco para obter seu caminho relativo
        match self.db.buscar_obra_por_titulo(obra_titulo).await? {
            Some(_obra) => {
                // Retorna capítulos da obra via listar_capitulos_obra do banco
                let caps = self.db.listar_capitulos_obra(
                    _obra.id,
                ).await?;
                Ok(caps.into_iter().map(|c| c.caminho_relativo.as_str().to_string()).collect())
            }
            None => Ok(vec![]),
        }
    }
}

/// Informação de capítulo com status de duplicata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CapituloInfoComStatus {
    pub fonte_id: String,
    pub obra_id_fonte: String,
    pub capitulo_id_fonte: String,
    pub numero: Option<i32>,
    pub titulo: Option<String>,
    pub url: Option<String>,
    pub ja_arquivado: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capitulo_info_com_status_cria() {
        let info = CapituloInfoComStatus {
            fonte_id: "fonte-1".to_string(),
            obra_id_fonte: "obra-1".to_string(),
            capitulo_id_fonte: "cap-1".to_string(),
            numero: Some(1),
            titulo: Some("Capítulo 1".to_string()),
            url: Some("https://exemplo.com/cap1".to_string()),
            ja_arquivado: false,
        };
        assert!(!info.ja_arquivado);
        assert_eq!(info.numero, Some(1));
    }
}
