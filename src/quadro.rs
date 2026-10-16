//! Personalização do quadro: título e legenda, pizza/rosca, fundos e cartão.
//! Opcional e serializável; o padrão reproduz o desenho anterior.
use crate::personalizacao::FonteTexto;
use crate::ErroGrafico;
use feather_draw::{Cor, Ponto, Preenchimento, Parada, Retangulo};
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------ título/legenda

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AlinhamentoTitulo {
    Esquerda,
    #[default]
    Centro,
    Direita,
}

/// Estilo do título ou do subtítulo. `tamanho` `None` usa o padrão (1,4 vez
/// a fonte para o título, 1 vez para o subtítulo).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EstiloTexto {
    pub fonte: FonteTexto,
    pub alinhamento: AlinhamentoTitulo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PosicaoLegenda {
    Cima,
    #[default]
    Baixo,
    Esquerda,
    Direita,
    Nenhuma,
}

/// Ativar este bloco liga o layout novo: a área de plotagem se ajusta ao
/// espaço de legenda, títulos e títulos de eixo (sem a sobreposição da
/// legenda com os rótulos do eixo X que o layout antigo tinha).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ConfigLegenda {
    pub posicao: PosicaoLegenda,
    pub fonte: FonteTexto,
}

// ------------------------------------------------------------ pizza/rosca

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Explosao {
    /// Índice da fatia (categoria).
    pub indice: usize,
    /// Deslocamento para fora, em fração do raio (0 a 0,5).
    pub deslocamento: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Borda {
    #[serde(with = "crate::serde_cor::cor")]
    pub cor: Cor,
    pub espessura: f32,
}
impl Default for Borda {
    fn default() -> Self {
        Self { cor: Cor::rgb(255, 255, 255), espessura: 1.5 }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EstiloPizza {
    /// Espessura do anel, em fração do raio (0 a 1). `None` = pizza cheia;
    /// `Some(0.4)` = rosca com furo de 60% do raio.
    pub espessura_anel: Option<f32>,
    /// Ângulo da primeira fatia em graus, sentido horário a partir do topo.
    pub angulo_inicial: f32,
    pub explodir: Vec<Explosao>,
    /// Explosão aplicada a todas as fatias sem entrada em `explodir`.
    pub explodir_todas: f32,
    /// Borda entre as fatias; sobrepõe o contorno global.
    pub borda: Option<Borda>,
}
impl EstiloPizza {
    pub fn validar(&self) -> Result<(), ErroGrafico> {
        let anel = self.espessura_anel.map_or(true, |e| e.is_finite() && e > 0.0 && e <= 1.0);
        let expl = |d: f32| d.is_finite() && (0.0..=0.5).contains(&d);
        let ok = anel
            && self.angulo_inicial.is_finite()
            && expl(self.explodir_todas)
            && self.explodir.iter().all(|e| expl(e.deslocamento))
            && self.borda.map_or(true, |b| b.espessura.is_finite() && b.espessura >= 0.0);
        if ok { Ok(()) } else { Err(ErroGrafico::ValorInvalido) }
    }
    pub fn deslocamento(&self, fatia: usize) -> f32 {
        self.explodir
            .iter()
            .rev()
            .find(|e| e.indice == fatia)
            .map_or(self.explodir_todas, |e| e.deslocamento)
    }
    pub fn maior_deslocamento(&self, fatias: usize) -> f32 {
        (0..fatias).map(|i| self.deslocamento(i)).fold(0.0, f32::max)
    }
    /// Fração do raio ocupada pelo furo (0 = pizza cheia).
    pub fn furo(&self) -> f32 {
        self.espessura_anel.map_or(0.0, |e| (1.0 - e).clamp(0.0, 0.99))
    }
}

// ------------------------------------------------------- fundo e cartão

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Fundo {
    Solido(#[serde(with = "crate::serde_cor::cor")] Cor),
    /// `angulo` em graus: 90 vai de cima para baixo, 0 da esquerda para a direita.
    Gradiente {
        #[serde(with = "crate::serde_cor::cor")]
        de: Cor,
        #[serde(with = "crate::serde_cor::cor")]
        ate: Cor,
        angulo: f32,
    },
}
impl Fundo {
    pub fn validar(&self) -> Result<(), ErroGrafico> {
        match self {
            Self::Gradiente { angulo, .. } if !angulo.is_finite() => Err(ErroGrafico::ValorInvalido),
            _ => Ok(()),
        }
    }
    pub fn preenchimento(&self, area: Retangulo) -> Preenchimento {
        match *self {
            Self::Solido(c) => Preenchimento::Solido(c),
            Self::Gradiente { de, ate, angulo } => {
                let (cx, cy) = (area.x + area.largura / 2.0, area.y + area.altura / 2.0);
                let a = angulo.to_radians();
                let (dx, dy) = (a.cos(), a.sin());
                // Meio comprimento da projeção do retângulo sobre a direção.
                let m = (area.largura * dx.abs() + area.altura * dy.abs()) / 2.0;
                Preenchimento::Linear {
                    de: Ponto::novo(cx - dx * m, cy - dy * m),
                    ate: Ponto::novo(cx + dx * m, cy + dy * m),
                    paradas: vec![Parada { posicao: 0.0, cor: de }, Parada { posicao: 1.0, cor: ate }],
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Sombra {
    #[serde(with = "crate::serde_cor::cor")]
    pub cor: Cor,
    pub dx: f32,
    pub dy: f32,
    /// Largura do esmaecimento, em unidades da cena (0 = sombra dura).
    pub desfoque: f32,
}
impl Default for Sombra {
    fn default() -> Self {
        Self { cor: Cor::rgba(0, 0, 0, 70), dx: 0.0, dy: 3.0, desfoque: 8.0 }
    }
}

/// O gráfico como um cartão: cantos arredondados, borda, sombra e respiros.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Cartao {
    pub raio: f32,
    pub borda: Option<Borda>,
    pub sombra: Option<Sombra>,
    /// Afasta o cartão das bordas da cena (espaço para a sombra aparecer).
    pub margem_externa: f32,
    /// Respiro entre a borda do cartão e o conteúdo (títulos, legenda, plotagem).
    pub margem_interna: f32,
}
impl Default for Cartao {
    fn default() -> Self {
        Self { raio: 12.0, borda: None, sombra: None, margem_externa: 0.0, margem_interna: 0.0 }
    }
}
impl Cartao {
    pub fn validar(&self) -> Result<(), ErroGrafico> {
        let ok = [self.raio, self.margem_externa, self.margem_interna]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.0)
            && self.borda.map_or(true, |b| b.espessura.is_finite() && b.espessura >= 0.0)
            && self.sombra.map_or(true, |s| {
                s.dx.is_finite() && s.dy.is_finite() && s.desfoque.is_finite() && s.desfoque >= 0.0
            });
        if ok { Ok(()) } else { Err(ErroGrafico::ValorInvalido) }
    }
    /// Retângulo do cartão dentro da cena.
    pub fn area(&self, largura: f32, altura: f32) -> Retangulo {
        let m = self.margem_externa.min(largura.min(altura) / 4.0);
        Retangulo::novo(m, m, largura - 2.0 * m, altura - 2.0 * m)
    }
    /// Afastamento total do conteúdo em relação às bordas da cena.
    pub fn recuo(&self) -> f32 {
        self.margem_externa + self.margem_interna
    }
}
