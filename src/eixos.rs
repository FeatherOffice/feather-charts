//! Domínios e ticks de eixos, reutilizáveis por backends SVG, Iced e PDF.
//! Texto é somente metadado: a tipografia pertence aos adaptadores (CHT-007).
use crate::{ErroGrafico, Grafico, Tipo, aparencia::LayoutGrafico};
use feather_draw::Retangulo;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EscalaLinear {
    pub minimo: f64,
    pub maximo: f64,
}
impl EscalaLinear {
    pub fn nova(minimo: f64, maximo: f64) -> Result<Self, ErroGrafico> {
        if !minimo.is_finite() || !maximo.is_finite() || minimo > maximo {
            return Err(ErroGrafico::EixoInvalido);
        }
        // Séries constantes de zero usam eixo 0..1 e baseline inferior.
        let maximo = if minimo == maximo {
            if minimo == 0.0 { 1.0 } else { maximo }
        } else {
            maximo
        };
        Ok(Self { minimo, maximo })
    }

    /// Interpolação robusta mesmo quando max - min ultrapassa f64::MAX.
    pub fn fracao(&self, valor: f64) -> f32 {
        if self.minimo == self.maximo {
            return 0.5;
        }
        let a = valor - self.minimo;
        let b = self.maximo - self.minimo;
        let f = if a.is_finite() && b.is_finite() {
            a / b
        } else {
            (valor / 2.0 - self.minimo / 2.0)
                / (self.maximo / 2.0 - self.minimo / 2.0)
        };
        f.clamp(0.0, 1.0) as f32
    }

    pub fn ticks(&self, divisao: usize) -> Vec<f64> {
        let n = divisao.clamp(1, 12);
        if self.minimo == self.maximo {
            return vec![self.minimo];
        }
        let mut valores = (0..=n)
            .map(|i| {
                let t = i as f64 / n as f64;
                // Lerp estável para extremos com sinais opostos.
                self.minimo * (1.0 - t) + self.maximo * t
            })
            .collect::<Vec<_>>();
        if self.minimo < 0.0 && self.maximo > 0.0 {
            valores.push(0.0);
        }
        valores.sort_by(|a, b| a.total_cmp(b));
        valores.dedup_by(|a, b| *a == *b);
        valores
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tick {
    /// Coordenada no eixo correspondente, em unidades de cena.
    pub posicao: f32,
    pub rotulo: String,
    pub numero: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Eixos {
    pub area: Retangulo,
    pub escala_x: Option<EscalaLinear>,
    pub escala_y: Option<EscalaLinear>,
    pub ticks_x: Vec<Tick>,
    pub ticks_y: Vec<Tick>,
    /// Linha de valor zero em X para barras, em Y para os demais tipos.
    pub zero: f32,
    pub zero_vertical: bool,
}

fn rotulo_numero(v: f64) -> String {
    if v == 0.0 {
        return "0".to_string();
    }
    if v.abs() >= 1e7 || v.abs() < 1e-3 {
        return format!("{v:.2e}");
    }
    let txt = format!("{v:.3}");
    txt.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn ticks_numericos(escala: EscalaLinear, inicio: f32, tamanho: f32, inverter: bool) -> Vec<Tick> {
    escala
        .ticks(5)
        .into_iter()
        .map(|v| {
            let t = escala.fracao(v);
            Tick {
                posicao: inicio + tamanho * if inverter { 1.0 - t } else { t },
                rotulo: rotulo_numero(v),
                numero: Some(v),
            }
        })
        .collect()
}

fn ticks_categorias(categorias: &[String], inicio: f32, tamanho: f32, centro: bool) -> Vec<Tick> {
    let n = categorias.len();
    // Renderização de milhares de rótulos seria impraticável; mantém extremos.
    let maximo = ((tamanho / 65.0).floor() as usize).clamp(2, 24);
    let passo = if n <= maximo { 1 } else { (n - 1).div_ceil(maximo - 1) };
    let mut indices = (0..n).step_by(passo).collect::<Vec<_>>();
    if indices.last().copied() != Some(n - 1) {
        indices.push(n - 1);
    }
    indices
        .into_iter()
        .map(|i| Tick {
            posicao: inicio
                + tamanho
                    * if centro {
                        (i as f32 + 0.5) / n as f32
                    } else if n == 1 {
                        0.5
                    } else {
                        i as f32 / (n - 1) as f32
                    },
            rotulo: categorias[i].clone(),
            numero: None,
        })
        .collect()
}

impl Grafico {
    pub fn eixos(&self, largura: f32, altura: f32) -> Result<Option<Eixos>, ErroGrafico> {
        self.eixos_com_layout(largura, altura, &LayoutGrafico::default())
    }

    pub fn eixos_com_layout(
        &self,
        largura: f32,
        altura: f32,
        layout: &LayoutGrafico,
    ) -> Result<Option<Eixos>, ErroGrafico> {
        self.validar()?;
        let area = layout.area(largura, altura)?;
        if self.tipo == Tipo::Pizza {
            return Ok(None);
        }
        let minimo = self.series.iter().flat_map(|s| s.valores.iter()).copied().fold(0.0f64, f64::min);
        let maximo = self.series.iter().flat_map(|s| s.valores.iter()).copied().fold(0.0f64, f64::max);
        let eixo_valores = EscalaLinear::nova(minimo, maximo)?;
        let (escala_x, escala_y, ticks_x, ticks_y, zero, zero_vertical) = match self.tipo {
            Tipo::Barras => (
                Some(eixo_valores),
                None,
                ticks_numericos(eixo_valores, area.x, area.largura, false),
                ticks_categorias(&self.categorias, area.y, area.altura, true),
                area.x + area.largura * eixo_valores.fracao(0.0),
                true,
            ),
            Tipo::Colunas | Tipo::Linhas | Tipo::Area | Tipo::Degraus => (
                None,
                Some(eixo_valores),
                ticks_categorias(&self.categorias, area.x, area.largura, self.tipo == Tipo::Colunas),
                ticks_numericos(eixo_valores, area.y, area.altura, true),
                area.y + area.altura * (1.0 - eixo_valores.fracao(0.0)),
                false,
            ),
            Tipo::Dispersao => {
                let xs: Vec<f64> = self.categorias.iter().map(|c| c.parse::<f64>().expect("validar garante eixo numérico")).collect();
                let ex = EscalaLinear::nova(xs.iter().copied().fold(f64::INFINITY, f64::min), xs.iter().copied().fold(f64::NEG_INFINITY, f64::max))?;
                (
                    Some(ex),
                    Some(eixo_valores),
                    ticks_numericos(ex, area.x, area.largura, false),
                    ticks_numericos(eixo_valores, area.y, area.altura, true),
                    area.y + area.altura * (1.0 - eixo_valores.fracao(0.0)),
                    false,
                )
            }
            Tipo::Pizza => unreachable!(),
        };
        Ok(Some(Eixos {
            area,
            escala_x,
            escala_y,
            ticks_x,
            ticks_y,
            zero,
            zero_vertical,
        }))
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::Serie;
    use feather_draw::Cor;

    fn grafico(tipo: Tipo) -> Grafico {
        Grafico {
            titulo: String::new(),
            tipo,
            categorias: vec!["-2".into(), "3".into()],
            series: vec![Serie {
                nome: "S".into(),
                valores: vec![-5.0, 10.0],
                cor: Cor::rgb(2, 4, 6),
            }],
        }
    }

    #[test]
    fn baseline_misto_x_y_e_tiques() {
        let col = grafico(Tipo::Colunas).eixos(500.0, 200.0).unwrap().unwrap();
        assert!(col.zero > col.area.y && col.zero < col.area.y + col.area.altura);
        assert!(col.ticks_y.iter().any(|t| t.numero == Some(0.0)));
        assert_eq!(col.ticks_x[0].rotulo, "-2");
        let barras = grafico(Tipo::Barras).eixos(500.0, 200.0).unwrap().unwrap();
        assert!(barras.zero_vertical);
        assert!(barras.zero > barras.area.x && barras.zero < barras.area.x + barras.area.largura);
        assert!(barras.ticks_x.iter().any(|t| t.numero == Some(0.0)));
    }

    #[test]
    fn dispersao_eixo_x_numerico_e_sem_overflow() {
        let e = grafico(Tipo::Dispersao).eixos(480.0, 320.0).unwrap().unwrap();
        assert_eq!(e.escala_x.unwrap().minimo, -2.0);
        assert_eq!(e.escala_x.unwrap().maximo, 3.0);
        let enorme = EscalaLinear::nova(-f64::MAX, f64::MAX).unwrap();
        assert!((enorme.fracao(0.0) - 0.5).abs() < 0.0001);
        assert!(enorme.ticks(5).iter().all(|n| n.is_finite()));
    }

    #[test]
    fn limita_categorias_e_trata_dominio_constante() {
        let mut g = grafico(Tipo::Linhas);
        g.categorias = (0..100_000).map(|i| i.to_string()).collect();
        g.series[0].valores = vec![0.0; 100_000];
        let e = g.eixos(240.0, 200.0).unwrap().unwrap();
        assert!(e.ticks_x.len() <= 24);
        assert_eq!(e.ticks_x.last().unwrap().rotulo, "99999");
        assert_eq!(e.zero, e.area.y + e.area.altura);
        assert_eq!(EscalaLinear::nova(f64::NAN, 1.0), Err(ErroGrafico::EixoInvalido));
    }
}
