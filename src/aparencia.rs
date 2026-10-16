//! Aparência declarativa dos gráficos: motor puro, sem Iced.
use crate::{ErroGrafico, Tipo};
use crate::estilos::{Arredondamento, GradienteArea, Neon, Suavizacao};
use crate::personalizacao::{
    ConfigEixos, EstiloBarras, EstiloMarcador, EstiloSerie, RotulosDados, TracoLinha,
};
use crate::quadro::{Cartao, ConfigLegenda, EstiloPizza, EstiloTexto, Fundo};
use feather_draw::{Cor, Retangulo};
use serde::{Deserialize, Serialize};

/// Margens relativas ao tamanho da cena, para manter o layout proporcional.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Marcador {
    /// Nenhum nas linhas e quadrado na dispersão, como no renderer original.
    Automatico,
    Nenhum,
    Quadrado,
    Circulo,
    Losango,
    Triangulo,
    /// Sinal de mais (traços cruzados).
    Cruz,
}

/// Opções de desenho que não alteram os valores das séries nem o modelo `Grafico`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AparenciaGrafico {
    /// Paleta cíclica; vazia preserva `Serie::cor`.
    #[serde(with = "crate::serde_cor::lista")]
    pub paleta: Vec<Cor>,
    pub opacidade: f32,
    pub espessura_linha: f32,
    #[serde(with = "crate::serde_cor::opt")]
    pub cor_contorno: Option<Cor>,
    pub espessura_contorno: f32,
    pub marcador: Marcador,
    pub tamanho_marcador: f32,
    pub layout: LayoutGrafico,
    /// Fundo da área inteira do gráfico; `None` deixa transparente (padrão).
    #[serde(with = "crate::serde_cor::opt")]
    pub fundo: Option<Cor>,
    /// Cor dos textos (título, eixos, legenda); `None` mantém o cinza-escuro padrão.
    #[serde(with = "crate::serde_cor::opt")]
    pub cor_texto: Option<Cor>,
    /// Linhas de grade do eixo de valores; `None` não desenha (padrão).
    #[serde(with = "crate::serde_cor::opt")]
    pub cor_grade: Option<Cor>,
    /// Cor dos eixos e do zero; `None` mantém o cinza padrão.
    #[serde(with = "crate::serde_cor::opt")]
    pub cor_eixos: Option<Cor>,
    /// Cantos arredondados em colunas e barras; `None` mantém retângulos.
    pub arredondamento: Option<Arredondamento>,
    /// Curva monótona nas linhas e áreas das séries escolhidas.
    pub suavizacao: Suavizacao,
    /// Preenchimento da área em gradiente vertical até a base.
    pub gradiente_area: Option<GradienteArea>,
    /// Brilho neon atrás de linhas, bordas de área e colunas/barras.
    pub neon: Option<Neon>,
    /// Estilo por série (índice da série no gráfico original): cor, linha,
    /// traço, transparência, marcador, cor por ponto e por regra.
    pub series: Vec<EstiloSerie>,
    /// Gradiente, espaçamento entre grupos e sobreposição das colunas/barras.
    pub barras: Option<EstiloBarras>,
    /// Rótulos de dados (conteúdo, posição, formato, fonte). `None` mantém o
    /// comportamento antigo (`TipografiaGrafico::mostrar_valores`).
    pub rotulos: Option<RotulosDados>,
    /// Escala, formato, rotação, título e grades dos eixos.
    pub eixos: ConfigEixos,
    /// Fonte, tamanho, cor, negrito e alinhamento do título e do subtítulo.
    pub titulo: Option<EstiloTexto>,
    pub subtitulo: Option<EstiloTexto>,
    /// Posição da legenda; ligá-la ajusta a área de plotagem ao espaço dela.
    pub legenda: Option<ConfigLegenda>,
    /// Rosca, ângulo inicial, explosão e borda da pizza.
    pub pizza: Option<EstiloPizza>,
    /// Fundo do gráfico inteiro (cor ou gradiente); vale mais que `fundo`.
    pub fundo_grafico: Option<Fundo>,
    /// Fundo só da área de plotagem.
    pub fundo_plotagem: Option<Fundo>,
    /// Cartão: cantos arredondados, borda, sombra e margens internas.
    pub cartao: Option<Cartao>,
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
            fundo: None,
            cor_texto: None,
            cor_grade: None,
            cor_eixos: None,
            arredondamento: None,
            suavizacao: Suavizacao::Nenhuma,
            gradiente_area: None,
            neon: None,
            series: Vec::new(),
            barras: None,
            rotulos: None,
            eixos: ConfigEixos::default(),
            titulo: None,
            subtitulo: None,
            legenda: None,
            pizza: None,
            fundo_grafico: None,
            fundo_plotagem: None,
            cartao: None,
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
        if let Some(a) = &self.arredondamento {
            a.validar()?;
        }
        if let Some(g) = &self.gradiente_area {
            g.validar()?;
        }
        if let Some(n) = &self.neon {
            n.validar()?;
        }
        for s in &self.series {
            s.validar()?;
        }
        if let Some(b) = &self.barras {
            b.validar()?;
        }
        if let Some(r) = &self.rotulos {
            r.validar()?;
        }
        self.eixos.validar()?;
        for t in [&self.titulo, &self.subtitulo].into_iter().flatten() {
            t.fonte.validar()?;
        }
        if let Some(l) = &self.legenda {
            l.fonte.validar()?;
        }
        if let Some(p) = &self.pizza {
            p.validar()?;
        }
        for f in [&self.fundo_grafico, &self.fundo_plotagem].into_iter().flatten() {
            f.validar()?;
        }
        if let Some(c) = &self.cartao {
            c.validar()?;
        }
        Ok(())
    }

    /// Cor da paleta (ou a original) com a transparência global; ignora o
    /// estilo por série. A pizza usa esta, porque seu índice é o da fatia.
    pub fn cor_base(&self, indice: usize, original: Cor) -> Cor {
        let mut cor = if self.paleta.is_empty() {
            original
        } else {
            self.paleta[indice % self.paleta.len()]
        };
        cor.a = (f32::from(cor.a) * self.opacidade).round() as u8;
        cor
    }

    pub fn estilo_serie(&self, indice: usize) -> Option<&EstiloSerie> {
        self.series.get(indice)
    }

    /// Transparência global vezes a da série.
    pub fn opacidade_da_serie(&self, indice: usize) -> f32 {
        self.opacidade * self.estilo_serie(indice).and_then(|e| e.opacidade).unwrap_or(1.0)
    }

    pub fn cor_serie(&self, indice: usize, original: Cor) -> Cor {
        let mut cor = match self.estilo_serie(indice).and_then(|e| e.cor) {
            Some(c) => c,
            None if self.paleta.is_empty() => original,
            None => self.paleta[indice % self.paleta.len()],
        };
        cor.a = (f32::from(cor.a) * self.opacidade_da_serie(indice)).round() as u8;
        cor
    }

    /// Cor própria de um ponto (a individual, depois a regra), já com a
    /// transparência da série; `None` quando o ponto usa a cor da série.
    /// `serie_min`/`serie_max` servem à escala contínua.
    pub fn cor_propria_do_ponto(
        &self,
        serie: usize,
        ponto: usize,
        valor: f64,
        serie_min: f64,
        serie_max: f64,
    ) -> Option<Cor> {
        let es = self.estilo_serie(serie)?;
        let mut c = es
            .cor_do_ponto(ponto)
            .or_else(|| es.regra.as_ref().map(|r| r.cor(valor, serie_min, serie_max)))?;
        c.a = (f32::from(c.a) * self.opacidade_da_serie(serie)).round() as u8;
        Some(c)
    }

    /// Cor de um ponto: a individual, depois a regra, depois `base` (a da série).
    pub fn cor_ponto(
        &self,
        serie: usize,
        ponto: usize,
        valor: f64,
        base: Cor,
        serie_min: f64,
        serie_max: f64,
    ) -> Cor {
        self.cor_propria_do_ponto(serie, ponto, valor, serie_min, serie_max).unwrap_or(base)
    }

    /// Cor de uma fatia da pizza (índice da categoria original): paleta ou
    /// cor padrão, depois cor por ponto ou regra da série 0.
    pub fn cor_fatia(&self, indice: usize, valor: f64, minimo: f64, maximo: f64) -> Cor {
        let base = self.cor_base(indice, crate::CORES_PIZZA[indice % crate::CORES_PIZZA.len()]);
        self.cor_ponto(0, indice, valor, base, minimo, maximo)
    }

    pub fn espessura_da_serie(&self, indice: usize) -> f32 {
        self.estilo_serie(indice).and_then(|e| e.espessura).unwrap_or(self.espessura_linha)
    }

    /// Usa o caminho estilizado (traço ou espessura própria) na série.
    pub fn serie_personalizada(&self, indice: usize) -> bool {
        self.estilo_serie(indice).is_some_and(|e| e.traco != TracoLinha::Continuo)
    }

    pub fn traco_da_serie(&self, indice: usize) -> TracoLinha {
        self.estilo_serie(indice).map_or(TracoLinha::Continuo, |e| e.traco)
    }

    /// Marcador de uma série: a forma efetiva e o estilo, se houver.
    pub fn marcador_da_serie(&self, tipo: Tipo, indice: usize) -> (Marcador, Option<EstiloMarcador>) {
        match self.estilo_serie(indice).and_then(|e| e.marcador) {
            Some(m) => (m.forma, Some(m)),
            None => (self.marcador_para(tipo), None),
        }
    }

    /// Contorno de uma coluna/barra/marcador da série.
    pub fn contorno_da_serie(&self, indice: usize) -> (Option<Cor>, f32) {
        let es = self.estilo_serie(indice);
        let cor = match es.and_then(|e| e.contorno) {
            Some(mut c) => {
                c.a = (f32::from(c.a) * self.opacidade_da_serie(indice)).round() as u8;
                Some(c)
            }
            None => self.contorno(),
        };
        (cor, es.and_then(|e| e.espessura_contorno).unwrap_or(self.espessura_contorno))
    }

    /// Alguma opção liga o layout novo (legenda, títulos, margens do cartão,
    /// títulos de eixo ou rótulos girados) e a plotagem passa a se ajustar a eles.
    pub fn usa_moldura(&self) -> bool {
        self.legenda.is_some()
            || self.titulo.is_some()
            || self.subtitulo.is_some()
            || self.cartao.is_some()
            || self.eixos.valores.titulo.is_some()
            || self.eixos.categorias.titulo.is_some()
            || self.eixos.categorias.rotacao.graus() > 0.0
            || self.eixos.valores.rotacao.graus() > 0.0
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
