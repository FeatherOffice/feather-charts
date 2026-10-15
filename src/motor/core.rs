//! Contrato único para as 25 opções legadas do chart.ts.
//! Dados já extraídos/agregados: nenhum gráfico pode recalcular somas.
use feather_draw::{exportar_svg, Cena, Estilo, Figura, Ponto};
use crate::{AparenciaGrafico, Serie};
use super::tipos;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TipoGrafico {
    Colunas, ColunasEmpilhadas, Colunas100,
    Barras, BarrasEmpilhadas, Barras100,
    Linhas, Linha3d, Degraus, DegrausEmpilhados, Degraus100,
    Area, AreaDegraus, AreaDegrausEmpilhada, AreaDegraus100,
    Pizza, Dispersao, Bolhas, Combinado, MapaCalor, Cascata,
    Histograma, Radar, Candlestick, Gantt,
}
impl TipoGrafico {
    pub const TODOS: [Self; 25] = [
        Self::Colunas, Self::ColunasEmpilhadas, Self::Colunas100,
        Self::Barras, Self::BarrasEmpilhadas, Self::Barras100,
        Self::Linhas, Self::Linha3d, Self::Degraus,
        Self::DegrausEmpilhados, Self::Degraus100,
        Self::Area, Self::AreaDegraus, Self::AreaDegrausEmpilhada,
        Self::AreaDegraus100, Self::Pizza, Self::Dispersao,
        Self::Bolhas, Self::Combinado, Self::MapaCalor, Self::Cascata,
        Self::Histograma, Self::Radar, Self::Candlestick, Self::Gantt,
    ];
    pub const fn id(self) -> &'static str {
        match self {
            Self::Colunas => "column", Self::ColunasEmpilhadas => "column-stacked",
            Self::Colunas100 => "column-stacked-100",
            Self::Barras => "bar", Self::BarrasEmpilhadas => "bar-stacked",
            Self::Barras100 => "bar-stacked-100", Self::Linhas => "line",
            Self::Linha3d => "line-3d", Self::Degraus => "step-line",
            Self::DegrausEmpilhados => "step-line-stacked",
            Self::Degraus100 => "step-line-stacked-100",
            Self::Area => "area", Self::AreaDegraus => "step-area",
            Self::AreaDegrausEmpilhada => "step-area-stacked",
            Self::AreaDegraus100 => "step-area-stacked-100",
            Self::Pizza => "pie", Self::Dispersao => "scatter",
            Self::Bolhas => "bubble", Self::Combinado => "combo",
            Self::MapaCalor => "heatmap", Self::Cascata => "waterfall",
            Self::Histograma => "histogram", Self::Radar => "radar",
            Self::Candlestick => "candlestick", Self::Gantt => "gantt",
        }
    }
    pub fn pelo_id(id: &str) -> Option<Self> {
        Self::TODOS.iter().copied().find(|t| t.id() == id)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entrada {
    pub titulo: String,
    pub categorias: Vec<String>,
    pub series: Vec<Serie>,
    /// Valores originais de X para dispersão / bolhas / Linha 3D.
    pub x_values: Option<Vec<f64>>,
    pub z_values: Option<Vec<f64>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErroMotor {
    SemDados, TamanhosDiferentes, ValorInvalido, DimensoesInvalidas,
    EixoXInvalido, DadosInsuficientes, ExportacaoSvg,
}

impl Entrada {
    pub fn validar(&self) -> Result<(), ErroMotor> {
        if self.categorias.is_empty() || self.series.is_empty() {
            return Err(ErroMotor::SemDados);
        }
        let n = self.categorias.len();
        if self.series.iter().any(|s| s.valores.len() != n)
            || self.x_values.as_ref().is_some_and(|x| x.len() != n)
            || self.z_values.as_ref().is_some_and(|z| z.len() != n)
        { return Err(ErroMotor::TamanhosDiferentes); }
        if self.series.iter().flat_map(|s| &s.valores).any(|v| !v.is_finite())
            || self.x_values.as_ref().is_some_and(|x| x.iter().any(|v| !v.is_finite()))
            || self.z_values.as_ref().is_some_and(|z| z.iter().any(|v| !v.is_finite()))
        { return Err(ErroMotor::ValorInvalido); }
        Ok(())
    }
}

/// Desenho neutro em Cena. Os aplicativos fornecem séries já extraídas;
/// Iced, SVG, PDF e persistência são adaptadores externos.
pub fn renderizar(
    tipo: TipoGrafico, dados: &Entrada, largura: f32, altura: f32,
) -> Result<Cena, ErroMotor> {
    dados.validar()?;
    if !largura.is_finite() || !altura.is_finite() || largura < 40.0 || altura < 40.0 {
        return Err(ErroMotor::DimensoesInvalidas);
    }
    let mut cena = Cena::nova(largura, altura);
    let area = super::primitivas::Area::nova(largura, altura);
    match tipo {
        TipoGrafico::Colunas => tipos::colunas::desenhar(&mut cena, dados, area),
        TipoGrafico::ColunasEmpilhadas => tipos::colunas_empilhadas::desenhar(&mut cena, dados, area),
        TipoGrafico::Colunas100 => tipos::colunas_100::desenhar(&mut cena, dados, area),
        TipoGrafico::Barras => tipos::barras::desenhar(&mut cena, dados, area),
        TipoGrafico::BarrasEmpilhadas => tipos::barras_empilhadas::desenhar(&mut cena, dados, area),
        TipoGrafico::Barras100 => tipos::barras_100::desenhar(&mut cena, dados, area),
        TipoGrafico::Linhas => tipos::linhas::desenhar(&mut cena, dados, area),
        TipoGrafico::Linha3d => tipos::linha_3d::desenhar(&mut cena, dados, area)?,
        TipoGrafico::Degraus => tipos::degraus::desenhar(&mut cena, dados, area),
        TipoGrafico::DegrausEmpilhados => tipos::degraus_empilhados::desenhar(&mut cena, dados, area),
        TipoGrafico::Degraus100 => tipos::degraus_100::desenhar(&mut cena, dados, area),
        TipoGrafico::Area => tipos::area::desenhar(&mut cena, dados, area),
        TipoGrafico::AreaDegraus => tipos::area_degraus::desenhar(&mut cena, dados, area),
        TipoGrafico::AreaDegrausEmpilhada => tipos::area_degraus_empilhada::desenhar(&mut cena, dados, area),
        TipoGrafico::AreaDegraus100 => tipos::area_degraus_100::desenhar(&mut cena, dados, area),
        TipoGrafico::Pizza => tipos::pizza::desenhar(&mut cena, dados, area)?,
        TipoGrafico::Dispersao => tipos::dispersao::desenhar(&mut cena, dados, area)?,
        TipoGrafico::Bolhas => tipos::bolhas::desenhar(&mut cena, dados, area)?,
        TipoGrafico::Combinado => tipos::combinado::desenhar(&mut cena, dados, area),
        TipoGrafico::MapaCalor => tipos::mapa_calor::desenhar(&mut cena, dados, area),
        TipoGrafico::Cascata => tipos::cascata::desenhar(&mut cena, dados, area),
        TipoGrafico::Histograma => tipos::histograma::desenhar(&mut cena, dados, area),
        TipoGrafico::Radar => tipos::radar::desenhar(&mut cena, dados, area),
        TipoGrafico::Candlestick => tipos::candlestick::desenhar(&mut cena, dados, area)?,
        TipoGrafico::Gantt => tipos::gantt::desenhar(&mut cena, dados, area)?,
    }
    Ok(cena)
}

/// Como `renderizar`, com a aparência opcional: fundo, paleta, cantos
/// arredondados e brilho em colunas/barras (inclusive empilhadas e 100%),
/// suavização, gradiente e brilho em linhas e áreas. Os demais tipos
/// (pizza, dispersão, combinado…) ignoram os campos de estilo e desenham
/// como em `renderizar`. Com `AparenciaGrafico::default()` a cena é idêntica.
pub fn renderizar_com(
    tipo: TipoGrafico, dados: &Entrada, largura: f32, altura: f32, ap: &AparenciaGrafico,
) -> Result<Cena, ErroMotor> {
    use super::tipos::{barras as b, colunas as c, linhas as l};
    ap.validar().map_err(|_| ErroMotor::ValorInvalido)?;
    let area = super::primitivas::Area::nova(largura, altura);
    let (colunas, barras, linhas) = (
        match tipo {
            TipoGrafico::Colunas => Some(c::Modo::Agrupado),
            TipoGrafico::ColunasEmpilhadas => Some(c::Modo::Empilhado),
            TipoGrafico::Colunas100 => Some(c::Modo::Percentual),
            _ => None,
        },
        match tipo {
            TipoGrafico::Barras => Some(b::Modo::Agrupado),
            TipoGrafico::BarrasEmpilhadas => Some(b::Modo::Empilhado),
            TipoGrafico::Barras100 => Some(b::Modo::Percentual),
            _ => None,
        },
        match tipo {
            TipoGrafico::Linhas => Some((l::Modo::Simples, false, false)),
            TipoGrafico::Degraus => Some((l::Modo::Simples, true, false)),
            TipoGrafico::DegrausEmpilhados => Some((l::Modo::Empilhado, true, false)),
            TipoGrafico::Degraus100 => Some((l::Modo::Percentual, true, false)),
            TipoGrafico::Area => Some((l::Modo::Simples, false, true)),
            TipoGrafico::AreaDegraus => Some((l::Modo::Simples, true, true)),
            TipoGrafico::AreaDegrausEmpilhada => Some((l::Modo::Empilhado, true, true)),
            TipoGrafico::AreaDegraus100 => Some((l::Modo::Percentual, true, true)),
            _ => None,
        },
    );
    if colunas.is_none() && barras.is_none() && linhas.is_none() {
        let mut cena = renderizar(tipo, dados, largura, altura)?;
        if let Some(fundo) = ap.fundo {
            cena.figuras.insert(0, figura_fundo(largura, altura, fundo));
        }
        return Ok(cena);
    }
    dados.validar()?;
    if !largura.is_finite() || !altura.is_finite() || largura < 40.0 || altura < 40.0 {
        return Err(ErroMotor::DimensoesInvalidas);
    }
    let mut cena = Cena::nova(largura, altura);
    if let Some(fundo) = ap.fundo {
        cena.adicionar(figura_fundo(largura, altura, fundo));
    }
    if let Some(m) = colunas {
        c::desenhar_modo_ap(&mut cena, dados, area, m, ap);
    } else if let Some(m) = barras {
        b::desenhar_modo_ap(&mut cena, dados, area, m, ap);
    } else if let Some((m, degraus, preencher)) = linhas {
        l::desenhar_modo_ap(&mut cena, dados, area, m, degraus, preencher, ap);
    }
    Ok(cena)
}

fn figura_fundo(largura: f32, altura: f32, fundo: feather_draw::Cor) -> Figura {
    Figura::Polilinha {
        pontos: vec![
            Ponto::novo(0.0, 0.0), Ponto::novo(largura, 0.0),
            Ponto::novo(largura, altura), Ponto::novo(0.0, altura),
        ],
        fechada: true,
        estilo: Estilo { preenchimento: Some(fundo), contorno: None, espessura: 0.0 },
    }
}

pub fn renderizar_svg(
    tipo: TipoGrafico, dados: &Entrada, largura: f32, altura: f32,
) -> Result<String, ErroMotor> {
    let cena = renderizar(tipo, dados, largura, altura)?;
    exportar_svg(&cena).map_err(|_| ErroMotor::ExportacaoSvg)
}

#[cfg(test)]
mod testes {
    use super::*;
    use feather_draw::Cor;
    fn dados() -> Entrada {
        Entrada {
            titulo: "Medição".into(),
            categorias: vec!["1".into(), "2".into(), "3".into(), "4".into()],
            series: vec![Serie { nome: "A".into(), valores: vec![1., 2., 3., 4.],
                cor: Cor::rgb(34, 102, 190) }],
            x_values: Some(vec![1., 2., 3., 4.]),
            z_values: Some(vec![3., 1., 4., 2.]),
        }
    }
    #[test]
    fn os_25_tipos_têm_identidade_estável_e_cena_vetorial() {
        for tipo in TipoGrafico::TODOS {
            assert_eq!(TipoGrafico::pelo_id(tipo.id()), Some(tipo));
            let mut d = dados();
            if tipo == TipoGrafico::Candlestick {
                let fonte = d.series[0].clone();
                d.series = vec![
                    fonte.clone(),
                    crate::Serie { valores: vec![5., 6., 7., 8.], ..fonte.clone() },
                    crate::Serie { valores: vec![0., 1., 2., 3.], ..fonte.clone() },
                    crate::Serie { valores: vec![2., 3., 4., 5.], ..fonte },
                ];
            } else if tipo == TipoGrafico::Gantt {
                let fonte = d.series[0].clone();
                d.series.push(crate::Serie {
                    valores: vec![2., 3., 4., 5.], ..fonte
                });
            }
            let svg = renderizar_svg(tipo, &d, 480., 300.)
                .unwrap_or_else(|e| panic!("{}: {e:?}", tipo.id()));
            assert!(svg.contains("<svg"), "{}", tipo.id());
        }
        assert!(TipoGrafico::pelo_id("inexistente").is_none());
    }
    #[test]
    fn valida_dimensoes_nan_e_serie_irregular() {
        let mut d = dados();
        assert_eq!(renderizar(TipoGrafico::Linhas, &d, 0., 300.),
            Err(ErroMotor::DimensoesInvalidas));
        d.series[0].valores.push(1.);
        assert_eq!(d.validar(), Err(ErroMotor::TamanhosDiferentes));
        d.series[0].valores.pop();
        d.series[0].valores[0] = f64::NAN;
        assert_eq!(d.validar(), Err(ErroMotor::ValorInvalido));
    }
}
