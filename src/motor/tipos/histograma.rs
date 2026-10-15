//! Histograma: contagens de frequência; binning é parte semântica deste tipo.
use crate::motor::{core::Entrada,primitivas::{Area,rect}};
use feather_draw::Cena;
pub fn desenhar(c:&mut Cena,d:&Entrada,a:Area){
    let s=&d.series[0];
    let min=s.valores.iter().copied().fold(f64::INFINITY,f64::min);
    let max=s.valores.iter().copied().fold(f64::NEG_INFINITY,f64::max);
    let bins=((s.valores.len() as f64).sqrt() as usize).clamp(1,32);
    let mut frequencia=vec![0_usize;bins];
    for v in &s.valores {
        let pos=if max>min {((*v-min)/(max-min)*bins as f64) as usize}else{0};
        frequencia[pos.min(bins-1)]+=1;
    }
    let max_count=*frequencia.iter().max().unwrap_or(&1) as f32;
    let banda=a.w/bins as f32;
    for (i,f) in frequencia.iter().enumerate() {
        let h=a.h*(*f as f32/max_count.max(1.));
        rect(c,a.x+i as f32*banda,a.y+a.h-h,banda*0.98,h,s.cor);
    }
}
