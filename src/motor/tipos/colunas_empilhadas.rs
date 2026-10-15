//! Tipo colunas_empilhadas — variante independente com primitiva compartilhada.
use crate::motor::{core::Entrada, primitivas::Area};
use feather_draw::Cena;
use super::colunas;
pub fn desenhar(c:&mut Cena,d:&Entrada,a:Area){colunas::desenhar_modo(c,d,a,colunas::Modo::Empilhado);}
