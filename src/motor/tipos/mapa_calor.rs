//! Mapa de calor: intensidade normalizada global; mantém cada valor original.
use crate::motor::{core::Entrada,primitivas::{Area,rect}};
use feather_draw::{Cena,Cor};
pub fn desenhar(c:&mut Cena,d:&Entrada,a:Area){
    let vals=d.series.iter().flat_map(|s|&s.valores);
    let min=vals.clone().copied().fold(f64::INFINITY,f64::min);
    let max=vals.copied().fold(f64::NEG_INFINITY,f64::max);
    let den=(max-min).max(f64::EPSILON);
    let w=a.w/d.categorias.len() as f32;
    let h=a.h/d.series.len() as f32;
    for (j,s) in d.series.iter().enumerate(){
        for (i,v) in s.valores.iter().enumerate(){
            let t=((*v-min)/den).clamp(0.,1.);
            let r=(40.+215.*t) as u8;
            let b=(230.-200.*t) as u8;
            rect(c,a.x+i as f32*w,a.y+j as f32*h,w.max(1.),h.max(1.),Cor::rgb(r,80,b));
        }
    }
}
