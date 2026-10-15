//! Primitivas neutras ao backend; nunca recalculam agregações.
use feather_draw::{Cena, Cor, Estilo, Figura, Ponto, Retangulo};

/// Área de desenho usada pelos renderizadores de tipos de gráfico.
/// É parte da API de `motor::tipos` e precisa ser tão visível quanto
/// as funções `desenhar` que a recebem. Dados e escala continuam na engine.
#[derive(Clone, Copy)]
pub struct Area { pub x: f32, pub y: f32, pub w: f32, pub h: f32 }
impl Area {
    pub fn nova(largura: f32, altura: f32) -> Self {
        Self { x: largura * 0.12, y: altura * 0.08,
            w: largura * 0.82, h: altura * 0.79 }
    }
    pub fn y(self, v: f64, min: f64, max: f64) -> f32 {
        let d = if max > min { max - min } else { 1. };
        self.y + (self.h as f64 * (1. - (v - min) / d).clamp(0., 1.)) as f32
    }
    pub fn x(self, v: f64, min: f64, max: f64) -> f32 {
        let d = if max > min { max - min } else { 1. };
        self.x + (self.w as f64 * ((v - min) / d).clamp(0., 1.)) as f32
    }
}
pub fn rect(c: &mut Cena, x: f32, y: f32, w: f32, h: f32, cor: Cor) {
    if w <= 0. || h <= 0. { return; }
    c.adicionar(Figura::Retangulo {
        area: Retangulo::novo(x, y, w, h),
        estilo: Estilo { preenchimento: Some(cor), contorno: None, espessura: 0. },
    });
}
pub fn linha(c: &mut Cena, pontos: Vec<Ponto>, cor: Cor, espessura: f32) {
    if pontos.len() < 2 { return; }
    c.adicionar(Figura::Polilinha {
        pontos, fechada: false,
        estilo: Estilo { preenchimento: None, contorno: Some(cor), espessura },
    });
}
pub fn poligono(c: &mut Cena, pontos: Vec<Ponto>, cor: Cor) {
    if pontos.len() < 3 { return; }
    c.adicionar(Figura::Polilinha {
        pontos, fechada: true,
        estilo: Estilo { preenchimento: Some(cor), contorno: None, espessura: 0. },
    });
}
pub fn ponto(c: &mut Cena, x: f32, y: f32, r: f32, cor: Cor) {
    let passos = 16;
    let pts = (0..passos).map(|i| {
        let t = i as f32 * std::f32::consts::TAU / passos as f32;
        Ponto::novo(x + r * t.cos(), y + r * t.sin())
    }).collect();
    poligono(c, pts, cor);
}
pub fn limites(dados: &super::core::Entrada) -> (f64, f64) {
    let mut lo = 0.0_f64;
    let mut hi = 0.0_f64;
    for v in dados.series.iter().flat_map(|s| &s.valores) {
        lo = lo.min(*v);
        hi = hi.max(*v);
    }
    if lo == hi { (lo - 1., hi + 1.) } else { (lo, hi) }
}
pub fn x_categoria(area: Area, indice: usize, total: usize) -> f32 {
    area.x + area.w * (indice as f32 + 0.5) / total.max(1) as f32
}
pub fn contraste(c: Cor, alpha: u8) -> Cor { Cor::rgba(c.r, c.g, c.b, alpha) }
