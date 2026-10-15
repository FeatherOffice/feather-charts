//! Bolhas: X real, Y na primeira série, tamanho opcional na segunda.
use crate::motor::{core::{Entrada,ErroMotor},primitivas::{Area,ponto,limites}};
use super::dispersao;
use feather_draw::Cena;
pub fn desenhar(c:&mut Cena,d:&Entrada,a:Area)->Result<(),ErroMotor>{
    let xs=dispersao::xs(d)?;
    let xmin=xs.iter().copied().fold(f64::INFINITY,f64::min);
    let xmax=xs.iter().copied().fold(f64::NEG_INFINITY,f64::max);
    let (min,max)=limites(d);
    let s=&d.series[0];
    let tamanho=d.z_values.as_ref().or_else(||d.series.get(1).map(|x|&x.valores));
    let max_t=tamanho.map(|v|v.iter().copied().map(f64::abs).fold(0_f64,f64::max)).unwrap_or(1.).max(1.);
    for (i,v) in s.valores.iter().enumerate(){
        let tam=tamanho.map(|ts|ts[i].abs()).unwrap_or(1.);
        let r=3. + 13.*(tam/max_t).sqrt() as f32;
        ponto(c,a.x(xs[i],xmin,xmax),a.y(*v,min,max),r,s.cor);
    }
    Ok(())
}
