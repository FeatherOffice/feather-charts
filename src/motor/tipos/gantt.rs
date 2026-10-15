//! Cronograma: início e término numéricos nas duas primeiras séries.
use crate::motor::{core::{Entrada,ErroMotor},primitivas::{Area,rect}};
use feather_draw::Cena;
pub fn desenhar(c:&mut Cena,d:&Entrada,a:Area)->Result<(),ErroMotor>{
    if d.series.len()<2 {return Err(ErroMotor::DadosInsuficientes);}
    let inicio=&d.series[0];let fim=&d.series[1];
    if inicio.valores.iter().zip(&fim.valores).any(|(s,e)|e<s) {
        return Err(ErroMotor::ValorInvalido);
    }
    let min=inicio.valores.iter().copied().fold(f64::INFINITY,f64::min);
    let max=fim.valores.iter().copied().fold(f64::NEG_INFINITY,f64::max);
    let intervalo=(max-min).max(1.);
    let banda=a.h/d.categorias.len() as f32;
    for i in 0..d.categorias.len(){
        let x=a.x+a.w*((inicio.valores[i]-min)/intervalo) as f32;
        let w=a.w*((fim.valores[i]-inicio.valores[i])/intervalo) as f32;
        rect(c,x,a.y+i as f32*banda+banda*0.2,w.max(1.),banda*0.6,inicio.cor);
    }
    Ok(())
}
