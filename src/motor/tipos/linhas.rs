//! Linhas/áreas cartesianas. Compartilha geometria com variantes de degraus.
use crate::motor::{core::Entrada, primitivas::{Area, limites, linha, poligono, contraste, ponto, x_categoria}};
use crate::estilos::{desenhar_serie_linha, SerieLinha};
use crate::AparenciaGrafico;
use feather_draw::{Cena, Ponto};
#[derive(Clone, Copy)]
pub enum Modo { Simples, Empilhado, Percentual }
pub fn desenhar(c: &mut Cena, d: &Entrada, a: Area) {
    desenhar_modo(c,d,a,Modo::Simples,false,false);
}
pub fn desenhar_modo(c: &mut Cena, d: &Entrada, a: Area, modo: Modo, degraus: bool, preencher: bool) {
    desenhar_modo_ap(c, d, a, modo, degraus, preencher, &AparenciaGrafico::default())
}
/// Igual a `desenhar_modo`, com suavização, gradiente de área e brilho da
/// aparência. Degraus nunca são suavizados.
pub fn desenhar_modo_ap(
    c: &mut Cena, d: &Entrada, a: Area, modo: Modo, degraus: bool, preencher: bool,
    ap: &AparenciaGrafico,
) {
    let n = d.categorias.len();
    let totais: Vec<f64> = (0..n).map(|i| d.series.iter()
        .map(|s| s.valores[i].abs()).sum::<f64>()).collect();
    let mut acum = vec![0_f64;n];
    let mut series = Vec::with_capacity(d.series.len());
    for s in &d.series {
        let mut valores = Vec::with_capacity(n);
        for (i, v) in s.valores.iter().enumerate() {
            let v = if matches!(modo,Modo::Percentual) {
                if totais[i]>0. {*v/totais[i]} else {0.}
            } else {*v};
            if matches!(modo,Modo::Simples) { valores.push(v); }
            else { acum[i] += v; valores.push(acum[i]); }
        }
        series.push(valores);
    }
    let (mut min,mut max)= if matches!(modo,Modo::Simples) {limites(d)} else {
        let mut lo=0_f64;let mut hi=0_f64;
        for v in series.iter().flat_map(|s|s.iter()) {lo=lo.min(*v);hi=hi.max(*v);}
        (lo,hi)
    };
    if min==max {min-=1.;max+=1.;}
    let baseline=a.y(0.,min,max);
    for (indice, (s, valores)) in d.series.iter().zip(series.iter()).enumerate() {
        let cor = ap.cor_serie(indice, s.cor);
        let original: Vec<Ponto> = valores.iter().enumerate().map(|(i,v)| {
            Ponto::novo(x_categoria(a,i,n),a.y(*v,min,max))
        }).collect();
        let mut path=Vec::with_capacity(original.len()*2);
        if let Some(p)=original.first(){path.push(*p);}
        for par in original.windows(2){
            if degraus { path.push(Ponto::novo(par[1].x,par[0].y)); }
            path.push(par[1]);
        }
        if ap.estiliza_linha(indice, preencher, !degraus) {
            desenhar_serie_linha(c, &path, &SerieLinha {
                cor, indice_serie: indice, baseline, area: preencher,
                permite_suavizar: !degraus, aparencia: ap,
            });
            if !degraus { for p in original {ponto(c,p.x,p.y,2.3,cor);} }
            continue;
        }
        if preencher && !path.is_empty() {
            let mut pol=Vec::with_capacity(path.len()+2);
            pol.push(Ponto::novo(path[0].x,baseline));
            pol.extend(path.iter().copied());
            pol.push(Ponto::novo(path.last().unwrap().x,baseline));
            poligono(c,pol,contraste(cor,88));
        }
        linha(c,path,cor,2.);
        if !degraus { for p in original {ponto(c,p.x,p.y,2.3,cor);} }
    }
}
