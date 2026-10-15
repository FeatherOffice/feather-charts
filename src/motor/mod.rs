//! Motor compartilhado de gráficos: sem Iced, DOM ou regras de planilha.
//! Cada tipo tem seu módulo em tipos/ e compartilha o contrato em core.rs.
pub mod core;
mod primitivas;
pub mod tipos;

pub use core::{renderizar, renderizar_com, renderizar_svg, Entrada, ErroMotor, TipoGrafico};
/// Área da Cena compartilhada pelos 25 renderizadores públicos de `tipos`.
/// Reexportar evita uma assinatura `pub` com tipo inacessível aos consumidores.
pub use primitivas::Area;
