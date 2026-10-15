//! Gráfico combinado: colunas da primeira série e linhas das demais.
use crate::motor::{core::Entrada,primitivas::Area};
use super::{colunas,linhas};
use feather_draw::Cena;
pub fn desenhar(c:&mut Cena,d:&Entrada,a:Area){
    let mut col=d.clone();
    col.series.truncate(1);
    colunas::desenhar(c,&col,a);
    if d.series.len()>1 {
        let mut curva=d.clone();
        curva.series.remove(0);
        linhas::desenhar(c,&curva,a);
    }
}
