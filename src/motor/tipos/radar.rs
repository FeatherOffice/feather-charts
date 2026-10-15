//! Radar polar: eixo radial comum para todas as séries.
use crate::motor::{core::Entrada,primitivas::{Area,linha,poligono,contraste}};
use feather_draw::{Cena,Ponto,Cor};
pub fn desenhar(c:&mut Cena,d:&Entrada,a:Area){
    let cx=a.x+a.w/2.;let cy=a.y+a.h/2.;let r=a.w.min(a.h)*0.43;
    let max=d.series.iter().flat_map(|s|&s.valores).copied().fold(0_f64,|p,v|p.max(v.abs())).max(1.);
    let n=d.categorias.len();
    for i in 0..n {
        let t=(i as f32*std::f32::consts::TAU/n as f32)-std::f32::consts::FRAC_PI_2;
        linha(c,vec![Ponto::novo(cx,cy),Ponto::novo(cx+r*t.cos(),cy+r*t.sin())],
            Cor::rgb(180,188,198),0.7);
    }
    for s in &d.series {
        let points:Vec<_>=s.valores.iter().enumerate().map(|(i,v)|{
            let t=(i as f32*std::f32::consts::TAU/n as f32)-std::f32::consts::FRAC_PI_2;
            let rr=r*(*v/max).clamp(-1.,1.) as f32;
            Ponto::novo(cx+rr*t.cos(),cy+rr*t.sin())
        }).collect();
        poligono(c,points.clone(),contraste(s.cor,72));
        if let Some(first)=points.first().copied(){
            let mut border=points; border.push(first); linha(c,border,s.cor,2.);
        }
    }
}
