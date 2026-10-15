//! Linha XYZ com projeção isométrica leve. X/Z preservam os valores de origem.
use crate::motor::{core::{Entrada,ErroMotor},primitivas::{Area,linha,ponto}};
use feather_draw::{Cena,Ponto};
pub fn desenhar(c:&mut Cena,d:&Entrada,a:Area)->Result<(),ErroMotor>{
    let xs=super::dispersao::xs(d)?;
    let zs=d.z_values.as_ref().ok_or(ErroMotor::DadosInsuficientes)?;
    let xmin=xs.iter().copied().fold(f64::INFINITY,f64::min);
    let xmax=xs.iter().copied().fold(f64::NEG_INFINITY,f64::max);
    let zmin=zs.iter().copied().fold(f64::INFINITY,f64::min);
    let zmax=zs.iter().copied().fold(f64::NEG_INFINITY,f64::max);
    let ymin=d.series.iter().flat_map(|s|&s.valores).copied().fold(f64::INFINITY,f64::min);
    let ymax=d.series.iter().flat_map(|s|&s.valores).copied().fold(f64::NEG_INFINITY,f64::max);
    let norm=|v:f64,lo:f64,hi:f64|((v-lo)/(hi-lo).max(1.)).clamp(0.,1.) as f32;
    for s in &d.series {
        let pontos:Vec<Ponto>=s.valores.iter().enumerate().map(|(i,y)|{
            let xx=norm(xs[i],xmin,xmax);
            let yy=norm(*y,ymin,ymax);
            let zz=norm(zs[i],zmin,zmax);
            Ponto::novo(a.x+a.w*(0.15+0.65*xx+0.20*zz),
                a.y+a.h*(0.8-0.64*yy-0.23*zz))
        }).collect();
        linha(c,pontos.clone(),s.cor,2.);
        for p in pontos{ponto(c,p.x,p.y,2.4,s.cor);}
    }
    Ok(())
}
