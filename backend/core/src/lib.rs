//! # mam-core
//! Núcleo do MAM: orquestração, ciclo de vida da aplicação,
//! configuração portátil e inicialização de processos auxiliares.
//!
//! Este crate é a raiz do workspace modular. Os domínios são crates
//! separados que dependem de `mam-core` quando necessário.

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
