//! Aparência declarativa dos gráficos: motor puro, sem Iced.
use crate::{ErroGrafico, Tipo};
use feather_draw::{Cor, Retangulo};

/// Margens relativas ao tamanho da cena, para manter o layout proporcional.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutGrafico {
    pub esquerda: f32,
    pub direita: f32,
    pub superior: f32,
    pub inferior: f32,
}
impl Default for LayoutGrafico {
    fn default() -> Self {
        Self {
            esquerda: 0.12,
            direita: 0.08,
            superior: 0.10,
            inferior: 0.12,
        }
    }
}
impl LayoutGrafico {
    pub fn area(&self, largura: f32, altura: f32) -> Result<Retangulo, ErroGrafico> {
        let fracoes = [self.esquerda, self.direita, self.superior, self.inferior];
        if !largura.is_finite()
            || !altura.is_finite()
            || largura <= 0.0
            || altura <= 0.0
            || fracoes.iter().any(|x| !x.is_finite() || *x < 0.0 || *x >= 1.0)
            || self.esquerda + self.direita >= 1.0
            || self.superior + self.inferior >= 1.0
        {
            return Err(ErroGrafico::EixoInvalido);
        }
        let area = Retangulo::novo(
            largura * self.esquerda,
            altura * self.superior,
            largura * (1.0 - (self.esquerda + self.direita)),
            altura * (1.0 - (self.superior + self.inferior)),
        );
        if !area.largura.is_finite()
            || !area.altura.is_finite()
            || area.largura <= 0.0
            || area.altura <= 0.0
        {
            return Err(ErroGrafico::EixoInvalido);
        }
        Ok(area)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marcador {
    /// Nenhum nas linhas e quadrado na dispersão, como no renderer original.
    Automatico,
    Nenhum,
    Quadrado,
    Circulo,
}

/// Opções de desenho que não alteram os valores das séries nem o modelo `Grafico`.
#[derive(Debug, Clone, PartialEq)]
pub struct AparenciaGrafico {
    /// Paleta cíclica; vazia preserva `Serie::cor`.
    pub paleta: Vec<Cor>,
    pub opacidade: f32,
    pub espessura_linha: f32,
    pub cor_contorno: Option<Cor>,
    pub espessura_contorno: f32,
    pub marcador: Marcador,
    pub tamanho_marcador: f32,
    pub layout: LayoutGrafico,
}
impl Default for AparenciaGrafico {
    fn default() -> Self {
        Self {
            paleta: Vec::new(),
            opacidade: 1.0,
            espessura_linha: 2.0,
            cor_contorno: None,
            espessura_contorno: 0.0,
            marcador: Marcador::Automatico,
            tamanho_marcador: 5.0,
            layout: LayoutGrafico::default(),
        }
    }
}
impl AparenciaGrafico {
    pub fn validar(&self) -> Result<(), ErroGrafico> {
        if !self.opacidade.is_finite()
            || !(0.0..=1.0).contains(&self.opacidade)
            || !self.espessura_linha.is_finite()
            || self.espessura_linha < 0.0
            || !self.espessura_contorno.is_finite()
            || self.espessura_contorno < 0.0
            || !self.tamanho_marcador.is_finite()
            || self.tamanho_marcador <= 0.0
        {
            return Err(ErroGrafico::ValorInvalido);
        }
        Ok(())
    }

    pub fn cor_serie(&self, indice: usize, original: Cor) -> Cor {
        let mut cor = if self.paleta.is_empty() {
            original
        } else {
            self.paleta[indice % self.paleta.len()]
        };
        cor.a = (f32::from(cor.a) * self.opacidade).round() as u8;
        cor
    }

    pub fn marcador_para(&self, tipo: Tipo) -> Marcador {
        match self.marcador {
            Marcador::Automatico if tipo == Tipo::Dispersao => Marcador::Quadrado,
            Marcador::Automatico => Marcador::Nenhum,
            outro => outro,
        }
    }

    pub fn contorno(&self) -> Option<Cor> {
        self.cor_contorno.map(|mut cor| {
            cor.a = (f32::from(cor.a) * self.opacidade).round() as u8;
            cor
        })
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn preserva_cor_original_e_aplica_paleta_ciclica_com_alfa() {
        let original = Cor::rgba(20, 30, 40, 128);
        let padrao = AparenciaGrafico::default();
        assert_eq!(padrao.cor_serie(0, original), original);
        let estilo = AparenciaGrafico {
            paleta: vec![Cor::rgba(200, 90, 20, 128)],
            opacidade: 0.5,
            ..padrao
        };
        assert_eq!(estilo.cor_serie(2, original), Cor::rgba(200, 90, 20, 64));
        assert_eq!(estilo.marcador_para(Tipo::Dispersao), Marcador::Quadrado);
        assert_eq!(estilo.marcador_para(Tipo::Linhas), Marcador::Nenhum);
    }

    #[test]
    fn layout_responsivo_e_parametros_invalidos() {
        let layout = LayoutGrafico::default();
        let a = layout.area(500.0, 250.0).unwrap();
        let b = layout.area(1000.0, 500.0).unwrap();
        assert_eq!(b.x, 2.0 * a.x);
        assert_eq!(b.altura, 2.0 * a.altura);
        assert_eq!(layout.area(0.0, 10.0), Err(ErroGrafico::EixoInvalido));
        assert_eq!(
            LayoutGrafico {
                direita: 0.90,
                ..layout
            }
            .area(100.0, 100.0),
            Err(ErroGrafico::EixoInvalido)
        );
        assert_eq!(
            AparenciaGrafico {
                opacidade: f32::NAN,
                ..Default::default()
            }
            .validar(),
            Err(ErroGrafico::ValorInvalido)
        );
    }
}
