//! Scatter: X numérico real, Y na série; sem interpolação do eixo categórico.
use crate::motor::{core::{Entrada,ErroMotor},primitivas::{Area,ponto,limites}};
use feather_draw::Cena;
pub fn xs(d:&Entrada)->Result<Vec<f64>,ErroMotor>{
    if let Some(xs)=&d.x_values {return Ok(xs.clone());}
    d.categorias.iter().map(|c|
        c.parse::<f64>().ok().filter(|v|v.is_finite()).ok_or(ErroMotor::EixoXInvalido)
    ).collect()
}
pub fn desenhar(c:&mut Cena,d:&Entrada,a:Area)->Result<(),ErroMotor>{
    let x=xs(d)?;
    let xmin=x.iter().copied().fold(f64::INFINITY,f64::min);
    let xmax=x.iter().copied().fold(f64::NEG_INFINITY,f64::max);
    let (min,max)=limites(d);
    for s in &d.series {
        for (i,v) in s.valores.iter().enumerate(){
            ponto(c,a.x(x[i],xmin,xmax),a.y(*v,min,max),3.5,s.cor);
        }
    }
    Ok(())
}
