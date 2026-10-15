//! Waterfall: cada barra é o incremento ao saldo anterior.
use crate::motor::{core::Entrada,primitivas::{Area,rect}};
use feather_draw::{Cena,Cor};
pub fn desenhar(c:&mut Cena,d:&Entrada,a:Area){
    let s=&d.series[0];
    let mut saldo=0_f64;let mut min=0_f64;let mut max=0_f64;
    for v in &s.valores {saldo+=*v;min=min.min(saldo);max=max.max(saldo);}
    if min==max {max+=1.;}
    saldo=0.;
    let banda=a.w/d.categorias.len() as f32;
    for (i,v) in s.valores.iter().enumerate(){
        let novo=saldo+v;
        let y0=a.y(saldo,min,max);let y1=a.y(novo,min,max);
        rect(c,a.x+i as f32*banda+banda*0.15,y0.min(y1),
             banda*0.7,(y0-y1).abs(),if *v>=0. {Cor::rgb(34,160,100)}else{Cor::rgb(220,70,72)});
        saldo=novo;
    }
}
