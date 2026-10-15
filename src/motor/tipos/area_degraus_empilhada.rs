//! Tipo area_degraus_empilhada — variante independente com primitiva compartilhada.
use crate::motor::{core::Entrada, primitivas::Area};
use feather_draw::Cena;
use super::linhas;
pub fn desenhar(c:&mut Cena,d:&Entrada,a:Area){linhas::desenhar_modo(c,d,a,linhas::Modo::Empilhado,true,true);}
