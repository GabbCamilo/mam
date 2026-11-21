//! Cache L1 LRU em memória para páginas e metadados frequentemente acessados.

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

/// Cache LRU em memória com capacidade máxima de entradas e limite opcional em bytes.
pub struct CacheL1Lru<K, V> {
    capacidade: usize,
    limite_bytes: Option<usize>,
    bytes_atuais: usize,
    ordem: VecDeque<K>,
    itens: HashMap<K, (V, usize)>,
}

impl<K, V> CacheL1Lru<K, V>
where
    K: Eq + Hash + Clone,
{
    pub fn nova(capacidade: usize) -> Self {
        Self {
            capacidade: capacidade.max(1),
            limite_bytes: None,
            bytes_atuais: 0,
            ordem: VecDeque::new(),
            itens: HashMap::new(),
        }
    }

    /// Cria um cache LRU limitado por quantidade de entradas e teto de memória em bytes.
    pub fn nova_com_limite_bytes(capacidade: usize, limite_bytes: usize) -> Self {
        Self {
            capacidade: capacidade.max(1),
            limite_bytes: Some(limite_bytes.max(1)),
            bytes_atuais: 0,
            ordem: VecDeque::new(),
            itens: HashMap::new(),
        }
    }

    /// Obtém uma referência ao valor e promove a chave para mais recente.
    pub fn obter(&mut self, chave: &K) -> Option<&V> {
        if self.itens.contains_key(chave) {
            self.promover(chave);
            self.itens.get(chave).map(|(v, _)| v)
        } else {
            None
        }
    }

    /// Insere ou atualiza um item sem peso explícito (peso 0).
    pub fn inserir(&mut self, chave: K, valor: V) {
        self.inserir_com_peso(chave, valor, 0);
    }

    /// Insere ou atualiza um item informando seu tamanho aproximado em bytes.
    pub fn inserir_com_peso(&mut self, chave: K, valor: V, peso_bytes: usize) {
        if let Some((_, peso_antigo)) = self.itens.remove(&chave) {
            self.bytes_atuais = self.bytes_atuais.saturating_sub(peso_antigo);
            self.ordem.retain(|k| k != &chave);
        }

        while self.itens.len() >= self.capacidade
            || self
                .limite_bytes
                .is_some_and(|max| self.bytes_atuais.saturating_add(peso_bytes) > max && !self.itens.is_empty())
        {
            if let Some(mais_antigo) = self.ordem.pop_front() {
                if let Some((_, peso_removido)) = self.itens.remove(&mais_antigo) {
                    self.bytes_atuais = self.bytes_atuais.saturating_sub(peso_removido);
                }
            } else {
                break;
            }
        }

        self.ordem.push_back(chave.clone());
        self.itens.insert(chave, (valor, peso_bytes));
        self.bytes_atuais = self.bytes_atuais.saturating_add(peso_bytes);
    }

    pub fn bytes_em_uso(&self) -> usize {
        self.bytes_atuais
    }

    pub fn len(&self) -> usize {
        self.itens.len()
    }

    pub fn is_empty(&self) -> bool {
        self.itens.is_empty()
    }

    fn promover(&mut self, chave: &K) {
        if let Some(pos) = self.ordem.iter().position(|k| k == chave) {
            if let Some(k) = self.ordem.remove(pos) {
                self.ordem.push_back(k);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_l1_lru_descarta_menos_recente() {
        let mut cache = CacheL1Lru::nova(2);
        cache.inserir("p1", vec![1u8]);
        cache.inserir("p2", vec![2u8]);
        // Promove p1
        assert_eq!(cache.obter(&"p1"), Some(&vec![1u8]));
        // Inserir p3 deve descartar p2
        cache.inserir("p3", vec![3u8]);
        assert_eq!(cache.len(), 2);
        assert!(cache.obter(&"p2").is_none());
        assert!(cache.obter(&"p1").is_some());
        assert!(cache.obter(&"p3").is_some());
    }

    #[test]
    fn cache_l1_lru_respeita_limite_bytes() {
        let mut cache = CacheL1Lru::nova_com_limite_bytes(10, 100);
        cache.inserir_com_peso("a", 1, 60);
        cache.inserir_com_peso("b", 2, 60);
        // "a" deve ter sido descartado para caber "b" dentro dos 100 bytes
        assert!(cache.obter(&"a").is_none());
        assert_eq!(cache.obter(&"b"), Some(&2));
        assert_eq!(cache.bytes_em_uso(), 60);
    }
}
