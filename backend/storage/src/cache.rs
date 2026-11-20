//! Cache L1 LRU em memória para páginas e metadados frequentemente acessados.

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

/// Cache LRU em memória com capacidade máxima fixa de entradas.
pub struct CacheL1Lru<K, V> {
    capacidade: usize,
    ordem: VecDeque<K>,
    itens: HashMap<K, V>,
}

impl<K, V> CacheL1Lru<K, V>
where
    K: Eq + Hash + Clone,
{
    pub fn nova(capacidade: usize) -> Self {
        Self {
            capacidade: capacidade.max(1),
            ordem: VecDeque::new(),
            itens: HashMap::new(),
        }
    }

    /// Obtém uma referência ao valor e promove a chave para mais recente.
    pub fn obter(&mut self, chave: &K) -> Option<&V> {
        if self.itens.contains_key(chave) {
            self.promover(chave);
            self.itens.get(chave)
        } else {
            None
        }
    }

    /// Insere ou atualiza um item, descartando o menos recentemente usado se exceder a capacidade.
    pub fn inserir(&mut self, chave: K, valor: V) {
        if self.itens.contains_key(&chave) {
            self.itens.insert(chave.clone(), valor);
            self.promover(&chave);
            return;
        }

        if self.itens.len() >= self.capacidade {
            if let Some(mais_antigo) = self.ordem.pop_front() {
                self.itens.remove(&mais_antigo);
            }
        }

        self.ordem.push_back(chave.clone());
        self.itens.insert(chave, valor);
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
}
