//! Colunas agrupadas/empilhadas; valores brutos imutáveis.
use crate::motor::{core::Entrada, primitivas::{Area, rect}};
use feather_draw::Cena;
#[derive(Clone, Copy)]
pub enum Modo { Agrupado, Empilhado, Percentual }
pub fn desenhar(cena: &mut Cena, d: &Entrada, area: Area) {
    desenhar_modo(cena, d, area, Modo::Agrupado)
}
pub fn desenhar_modo(cena: &mut Cena, d: &Entrada, area: Area, modo: Modo) {
    let n = d.categorias.len();
    let grupo = area.w / n as f32;
    let faixa = (grupo * 0.72).min(96.);
    let (mut baixo, mut alto) = (0.0_f64, 0.0_f64);
    for i in 0..n {
        let mut pos = 0_f64;
        let mut neg = 0_f64;
        for s in &d.series {
            let v = s.valores[i];
            if v >= 0. { pos += v; } else { neg += v; }
            if matches!(modo, Modo::Agrupado) {
                alto = alto.max(v);
                baixo = baixo.min(v);
            }
        }
        if matches!(modo, Modo::Empilhado) {
            alto = alto.max(pos);
            baixo = baixo.min(neg);
        }
    }
    if matches!(modo, Modo::Percentual) {
        alto = 1.;
        baixo = if d.series.iter().any(|s| s.valores.iter().any(|v| *v < 0.)) { -1. } else { 0. };
    }
    if baixo == alto { alto += 1.; }
    for i in 0..n {
        let soma_positiva = d.series.iter().map(|s| s.valores[i].max(0.)).sum::<f64>();
        let soma_negativa = d.series.iter().map(|s| (-s.valores[i]).max(0.)).sum::<f64>();
        let (mut pos, mut neg) = (0_f64, 0_f64);
        for (j, s) in d.series.iter().enumerate() {
            let mut valor = s.valores[i];
            if matches!(modo, Modo::Percentual) {
                let base = if valor >= 0. { soma_positiva } else { soma_negativa };
                valor = if base > 0. { valor / base } else { 0. };
            }
            let (inicio, fim) = if matches!(modo, Modo::Agrupado) {
                (0., valor)
            } else if valor >= 0. {
                let x = pos; pos += valor; (x, pos)
            } else {
                let x = neg; neg += valor; (x, neg)
            };
            let y0 = area.y(inicio, baixo, alto);
            let y1 = area.y(fim, baixo, alto);
            let largura = if matches!(modo, Modo::Agrupado) {
                faixa / d.series.len() as f32
            } else { faixa };
            let x = area.x + grupo * i as f32 + (grupo - faixa) * 0.5
                + if matches!(modo, Modo::Agrupado) { largura * j as f32 } else { 0. };
            rect(cena, x, y0.min(y1), largura * 0.93, (y0 - y1).abs(), s.cor);
        }
    }
}
