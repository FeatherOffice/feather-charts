//! Tipo barras_empilhadas — variante independente com primitiva compartilhada.
use crate::motor::{core::Entrada, primitivas::Area};
use feather_draw::Cena;
use super::barras;
pub fn desenhar(c:&mut Cena,d:&Entrada,a:Area){barras::desenhar_modo(c,d,a,barras::Modo::Empilhado);}
