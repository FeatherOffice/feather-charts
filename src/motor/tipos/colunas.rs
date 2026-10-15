//! Colunas agrupadas/empilhadas; valores brutos imutáveis.
use crate::estilos::{brilho_barra, figura_barra, raios_cantos, PapelBarra};
use crate::motor::{core::Entrada, primitivas::Area};
use crate::AparenciaGrafico;
use feather_draw::{Cena, Estilo, Retangulo};
#[derive(Clone, Copy)]
pub enum Modo { Agrupado, Empilhado, Percentual }
pub fn desenhar(cena: &mut Cena, d: &Entrada, area: Area) {
    desenhar_modo(cena, d, area, Modo::Agrupado)
}
pub fn desenhar_modo(cena: &mut Cena, d: &Entrada, area: Area, modo: Modo) {
    desenhar_modo_ap(cena, d, area, modo, &AparenciaGrafico::default())
}
/// Papel de cada série na pilha da categoria `i`, por sinal: (primeira, última)
/// com valor não nulo. A última é a ponta; a primeira, a base.
fn extremos_pilha(d: &Entrada, i: usize, positivo: bool) -> Option<(usize, usize)> {
    let mut it = d.series.iter().enumerate().filter(|(_, s)| {
        let v = s.valores[i];
        if positivo { v > 0. } else { v < 0. }
    }).map(|(j, _)| j);
    let primeiro = it.next()?;
    Some((primeiro, it.next_back().unwrap_or(primeiro)))
}
/// Igual a `desenhar_modo`, com cantos arredondados e brilho da aparência.
/// Em empilhadas só o segmento da ponta (e, em `TopoEBase`, o da base) arredonda.
pub fn desenhar_modo_ap(
    cena: &mut Cena, d: &Entrada, area: Area, modo: Modo, ap: &AparenciaGrafico,
) {
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
            let (w, h) = (largura * 0.93, (y0 - y1).abs());
            if w <= 0. || h <= 0. { continue; }
            let retangulo = Retangulo::novo(x, y0.min(y1), w, h);
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
                .map_or([0.; 4], |ar| raios_cantos(retangulo, true, positivo, papel, &ar));
            if let Some(neon) = &ap.neon {
                brilho_barra(cena, retangulo, raios, ap.espessura_contorno.max(1.), cor, neon);
            }
            cena.adicionar(figura_barra(retangulo, raios, Estilo {
                preenchimento: Some(cor), contorno: ap.contorno(), espessura: ap.espessura_contorno,
            }));
        }
    }
}
