//! Mercado: open, high, low, close nas quatro séries, sem reordenar dados.
use crate::motor::{core::{Entrada,ErroMotor},primitivas::{Area,rect,linha}};
use feather_draw::{Cena,Ponto,Cor};
pub fn desenhar(c:&mut Cena,d:&Entrada,a:Area)->Result<(),ErroMotor>{
    if d.series.len()<4 {return Err(ErroMotor::DadosInsuficientes);}
    let (ab,hi,lo,fe)=(&d.series[0],&d.series[1],&d.series[2],&d.series[3]);
    let min=lo.valores.iter().copied().fold(f64::INFINITY,f64::min);
    let max=hi.valores.iter().copied().fold(f64::NEG_INFINITY,f64::max);
    let (min,max)=if min==max {(min-1.,max+1.)}else{(min,max)};
    let bw=a.w/d.categorias.len() as f32;
    for i in 0..d.categorias.len(){
        if lo.valores[i]>hi.valores[i] || ab.valores[i]<lo.valores[i]
        || fe.valores[i]>hi.valores[i] || fe.valores[i]<lo.valores[i]
        || ab.valores[i]>hi.valores[i] {return Err(ErroMotor::ValorInvalido);}
        let x=a.x+bw*(i as f32+0.5);
        let cor=if fe.valores[i]>=ab.valores[i] {Cor::rgb(32,160,106)}
            else {Cor::rgb(215,65,70)};
        linha(c,vec![Ponto::novo(x,a.y(lo.valores[i],min,max)),
            Ponto::novo(x,a.y(hi.valores[i],min,max))],cor,1.5);
        let y0=a.y(ab.valores[i],min,max);
        let y1=a.y(fe.valores[i],min,max);
        rect(c,x-bw*0.25,y0.min(y1),bw*0.5,(y0-y1).abs().max(1.),cor);
    }
    Ok(())
}
