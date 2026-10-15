//! Pizza: frações não negativas, com triangulação setorial vetorial.
use crate::motor::{core::{Entrada, ErroMotor}, primitivas::{Area, poligono}};
use feather_draw::{Cena, Cor, Ponto};
const CORES:[Cor;8]=[
    Cor::rgb(66,133,244),Cor::rgb(234,67,53),Cor::rgb(251,188,4),
    Cor::rgb(52,168,83),Cor::rgb(142,36,170),Cor::rgb(0,172,193),
    Cor::rgb(255,112,67),Cor::rgb(124,179,66),
];
pub fn desenhar(c:&mut Cena,d:&Entrada,a:Area)->Result<(),ErroMotor>{
    let s=&d.series[0];
    if s.valores.iter().any(|v|*v<0.) {return Err(ErroMotor::ValorInvalido);}
    let total=s.valores.iter().sum::<f64>();
    if total<=0. {return Err(ErroMotor::SemDados);}
    let cx=a.x+a.w*0.5;
    let cy=a.y+a.h*0.5;
    let r=a.w.min(a.h)*0.46;
    let mut ang=-std::f64::consts::FRAC_PI_2;
    for (i,v) in s.valores.iter().enumerate(){
        if *v<=0. {continue;}
        let fim=ang+*v/total*std::f64::consts::TAU;
        let steps=((fim-ang)*32.).ceil().max(2.) as usize;
        let mut pts=Vec::with_capacity(steps+2);
        pts.push(Ponto::novo(cx,cy));
        for j in 0..=steps {
            let t=(ang+(fim-ang)*j as f64/steps as f64) as f32;
            pts.push(Ponto::novo(cx+r*t.cos(),cy+r*t.sin()));
        }
        poligono(c,pts,CORES[i%CORES.len()]);
        ang=fim;
    }
    Ok(())
}
