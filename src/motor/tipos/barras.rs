//! Barras horizontais, agrupadas e empilhadas.
use crate::estilos::{brilho_barra, figura_barra, raios_cantos, PapelBarra};
use crate::motor::{core::Entrada, primitivas::Area};
use crate::AparenciaGrafico;
use feather_draw::{Cena, Estilo, Retangulo};
#[derive(Clone, Copy)]
pub enum Modo { Agrupado, Empilhado, Percentual }
pub fn desenhar(c: &mut Cena, d: &Entrada, a: Area) { desenhar_modo(c,d,a,Modo::Agrupado) }
pub fn desenhar_modo(c: &mut Cena, d: &Entrada, a: Area, modo: Modo) {
    desenhar_modo_ap(c, d, a, modo, &AparenciaGrafico::default())
}
/// Primeira e última série com valor não nulo do sinal na categoria `i`.
fn extremos_pilha(d: &Entrada, i: usize, positivo: bool) -> Option<(usize, usize)> {
    let mut it = d.series.iter().enumerate().filter(|(_, s)| {
        let v = s.valores[i];
        if positivo { v > 0. } else { v < 0. }
    }).map(|(j, _)| j);
    let primeiro = it.next()?;
    Some((primeiro, it.next_back().unwrap_or(primeiro)))
}
/// Igual a `desenhar_modo`, com cantos arredondados e brilho da aparência.
pub fn desenhar_modo_ap(c: &mut Cena, d: &Entrada, a: Area, modo: Modo, ap: &AparenciaGrafico) {
    let n = d.categorias.len();
    let faixa = a.h / n as f32;
    let (mut min, mut max) = (0_f64, 0_f64);
    for i in 0..n {
        let (mut pos, mut neg) = (0_f64, 0_f64);
        for s in &d.series {
            let v = s.valores[i];
            pos += v.max(0.);
            neg += v.min(0.);
            if matches!(modo, Modo::Agrupado) { min = min.min(v); max = max.max(v); }
        }
        if matches!(modo, Modo::Empilhado) { min = min.min(neg); max = max.max(pos); }
    }
    if matches!(modo, Modo::Percentual) {
        min = if d.series.iter().any(|s| s.valores.iter().any(|v| *v < 0.)) { -1. } else { 0. };
        max = 1.;
    }
    if min == max { max += 1.; }
    for i in 0..n {
        let total_pos: f64 = d.series.iter().map(|s| s.valores[i].max(0.)).sum();
        let total_neg: f64 = d.series.iter().map(|s| (-s.valores[i]).max(0.)).sum();
        let (mut pos, mut neg) = (0., 0.);
        for (j, s) in d.series.iter().enumerate() {
            let mut valor = s.valores[i];
            if matches!(modo, Modo::Percentual) {
                let total = if valor >= 0. {total_pos} else {total_neg};
                valor = if total > 0. {valor / total} else {0.};
            }
            let (inicio, fim) = if matches!(modo, Modo::Agrupado) {(0.,valor)}
                else if valor >= 0. {let p=pos;pos+=valor;(p,pos)}
                else {let p=neg;neg+=valor;(p,neg)};
            let x0=a.x(inicio,min,max);let x1=a.x(fim,min,max);
            let altura=if matches!(modo, Modo::Agrupado) {
                faixa * 0.72 / d.series.len() as f32
            } else {faixa*0.72};
            let y=a.y+faixa*i as f32+faixa*0.14
                +if matches!(modo,Modo::Agrupado){altura*j as f32}else{0.};
            let (w, h) = ((x1 - x0).abs(), altura * 0.93);
            if w <= 0. || h <= 0. { continue; }
            let retangulo = Retangulo::novo(x0.min(x1), y, w, h);
            let cor = ap.cor_serie(j, s.cor);
            let positivo = s.valores[i] >= 0.;
            let papel = if matches!(modo, Modo::Agrupado) {
                PapelBarra::ISOLADA
            } else {
                match extremos_pilha(d, i, positivo) {
                    Some((base, ponta)) => PapelBarra { base: j == base, ponta: j == ponta },
                    None => PapelBarra::ISOLADA,
                }
            };
            let raios = ap.arredondamento
                .map_or([0.; 4], |ar| raios_cantos(retangulo, false, positivo, papel, &ar));
            if let Some(neon) = &ap.neon {
                brilho_barra(c, retangulo, raios, ap.espessura_contorno.max(1.), cor, neon);
            }
            c.adicionar(figura_barra(retangulo, raios, Estilo {
                preenchimento: Some(cor), contorno: ap.contorno(), espessura: ap.espessura_contorno,
            }));
        }
    }
}
