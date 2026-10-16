//! Domínios e ticks de eixos, reutilizáveis por backends SVG, Iced e PDF.
//! Texto é somente metadado: a tipografia pertence aos adaptadores (CHT-007).
use crate::personalizacao::ConfigEixo;
use crate::{aparencia::LayoutGrafico, AparenciaGrafico, ErroGrafico, Grafico, Tipo};
use feather_draw::Retangulo;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EscalaLinear {
    pub minimo: f64,
    pub maximo: f64,
    /// Escala logarítmica (base 10); exige `minimo > 0`.
    pub logaritmica: bool,
    /// Inverte o sentido (o máximo fica onde estava o mínimo).
    pub invertida: bool,
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
        Ok(Self { minimo, maximo, logaritmica: false, invertida: false })
    }

    /// Escala com as opções do eixo. Na logarítmica o mínimo precisa ser positivo.
    pub fn com_opcoes(minimo: f64, maximo: f64, logaritmica: bool, invertida: bool) -> Result<Self, ErroGrafico> {
        let mut e = Self::nova(minimo, maximo)?;
        if logaritmica && (e.minimo <= 0.0 || e.maximo <= e.minimo) {
            return Err(ErroGrafico::EixoInvalido);
        }
        e.logaritmica = logaritmica;
        e.invertida = invertida;
        Ok(e)
    }

    /// Interpolação robusta mesmo quando max - min ultrapassa f64::MAX.
    pub fn fracao(&self, valor: f64) -> f32 {
        let f = self.fracao_direta(valor);
        if self.invertida { 1.0 - f } else { f }
    }

    fn fracao_direta(&self, valor: f64) -> f32 {
        if self.minimo == self.maximo {
            return 0.5;
        }
        if self.logaritmica {
            if valor <= 0.0 {
                return 0.0;
            }
            let f = (valor.ln() - self.minimo.ln()) / (self.maximo.ln() - self.minimo.ln());
            return f.clamp(0.0, 1.0) as f32;
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

    /// Ticks principais respeitando o intervalo fixo (múltiplos dele) ou, na
    /// escala logarítmica, as potências de 10 (com 2 e 5 quando há menos de duas).
    pub fn ticks_principais(&self, intervalo: Option<f64>) -> Vec<f64> {
        if self.logaritmica {
            return self.potencias_log(&[1.0], 1);
        }
        match intervalo {
            Some(i) if i > 0.0 && i.is_finite() => {
                self.multiplos(i, 0.0).unwrap_or_else(|| self.ticks(5))
            }
            _ => self.ticks(5),
        }
    }

    /// Ticks das linhas secundárias: sem repetir os principais.
    pub fn ticks_secundarios(&self, intervalo: Option<f64>, subdivisoes: u8) -> Vec<f64> {
        let principais = self.ticks_principais(intervalo);
        let sub = f64::from(subdivisoes.clamp(2, 10));
        let candidatos = if self.logaritmica {
            self.potencias_log(&[2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0], 1)
        } else {
            let passo = match intervalo {
                Some(i) if i > 0.0 && i.is_finite() => i,
                _ => (self.maximo - self.minimo) / 5.0,
            };
            let ancora = if intervalo.is_some() { 0.0 } else { self.minimo };
            self.multiplos(passo / sub, ancora).unwrap_or_default()
        };
        let tol = (self.maximo - self.minimo).abs() * 1e-9;
        candidatos
            .into_iter()
            .filter(|v| !principais.iter().any(|p| (p - v).abs() <= tol.max(f64::EPSILON * v.abs())))
            .collect()
    }

    fn multiplos(&self, passo: f64, ancora: f64) -> Option<Vec<f64>> {
        if passo <= 0.0 || !passo.is_finite() || self.maximo <= self.minimo {
            return None;
        }
        let eps = 1e-9;
        let k0 = ((self.minimo - ancora) / passo - eps).ceil();
        let k1 = ((self.maximo - ancora) / passo + eps).floor();
        if !(k1 >= k0) || k1 - k0 > 400.0 {
            return None;
        }
        Some((k0 as i64..=k1 as i64).map(|k| ancora + k as f64 * passo).collect())
    }

    fn potencias_log(&self, mantissas: &[f64], _minimo: usize) -> Vec<f64> {
        let (a, b) = (self.minimo.log10().floor() as i32, self.maximo.log10().ceil() as i32);
        let mut v = Vec::new();
        for k in a..=b.min(a + 60) {
            for m in mantissas {
                let x = m * 10f64.powi(k);
                if x >= self.minimo * (1.0 - 1e-9) && x <= self.maximo * (1.0 + 1e-9) {
                    v.push(x);
                }
            }
        }
        if mantissas == [1.0] && v.len() < 3 {
            // Menos de duas décadas: acrescenta 2 e 5 para ter referência.
            return self.potencias_log(&[1.0, 2.0, 5.0], 1);
        }
        v
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
    /// Posições das linhas secundárias no eixo X / Y (só eixos numéricos).
    pub ticks_secundarios_x: Vec<f32>,
    pub ticks_secundarios_y: Vec<f32>,
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

fn posicao_numerica(escala: &EscalaLinear, v: f64, inicio: f32, tamanho: f32, inverter: bool) -> f32 {
    let t = escala.fracao(v);
    inicio + tamanho * if inverter { 1.0 - t } else { t }
}

fn ticks_numericos(escala: EscalaLinear, cfg: &ConfigEixo, inicio: f32, tamanho: f32, inverter: bool) -> (Vec<Tick>, Vec<f32>) {
    let principais = escala
        .ticks_principais(cfg.intervalo)
        .into_iter()
        .map(|v| Tick {
            posicao: posicao_numerica(&escala, v, inicio, tamanho, inverter),
            rotulo: rotulo_numero(v),
            numero: Some(v),
        })
        .collect();
    let secundarios = if cfg.grade_secundaria.is_some() {
        escala
            .ticks_secundarios(cfg.intervalo, cfg.subdivisoes)
            .into_iter()
            .map(|v| posicao_numerica(&escala, v, inicio, tamanho, inverter))
            .collect()
    } else {
        Vec::new()
    };
    (principais, secundarios)
}

fn ticks_categorias(categorias: &[String], inicio: f32, tamanho: f32, centro: bool, invertido: bool) -> Vec<Tick> {
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
        .map(|i| {
            let k = if invertido { n - 1 - i } else { i };
            Tick {
                posicao: inicio
                    + tamanho
                        * if centro {
                            (k as f32 + 0.5) / n as f32
                        } else if n == 1 {
                            0.5
                        } else {
                            k as f32 / (n - 1) as f32
                        },
                rotulo: categorias[i].clone(),
                numero: None,
            }
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
        let ap = AparenciaGrafico { layout: *layout, ..AparenciaGrafico::default() };
        self.eixos_com_aparencia(largura, altura, &ap)
    }

    /// Escala do eixo de valores: automática (inclui o zero) ou com os limites,
    /// a escala logarítmica e a ordem inversa pedidos em `cfg`.
    pub(crate) fn escala_de_valores(&self, cfg: &ConfigEixo) -> Result<EscalaLinear, ErroGrafico> {
        let todos = || self.series.iter().flat_map(|s| s.valores.iter()).copied();
        let (auto_min, auto_max) = if cfg.logaritmica {
            let pos_min = todos().filter(|v| *v > 0.0).fold(f64::INFINITY, f64::min);
            let pos_max = todos().filter(|v| *v > 0.0).fold(0.0f64, f64::max);
            if !pos_min.is_finite() {
                (1.0, 10.0)
            } else {
                let lo = 10f64.powf(pos_min.log10().floor());
                let mut hi = 10f64.powf(pos_max.log10().ceil());
                if hi <= lo {
                    hi = lo * 10.0;
                }
                (lo, hi)
            }
        } else {
            (todos().fold(0.0f64, f64::min), todos().fold(0.0f64, f64::max))
        };
        let base = EscalaLinear::nova(auto_min, auto_max)?;
        let min = cfg.minimo.unwrap_or(base.minimo);
        let max = cfg.maximo.unwrap_or(base.maximo);
        if cfg.minimo.is_some() || cfg.maximo.is_some() {
            if !(min < max) {
                return Err(ErroGrafico::EixoInvalido);
            }
        }
        EscalaLinear::com_opcoes(min, max, cfg.logaritmica, cfg.invertido)
    }

    /// Eixos com a escala, o intervalo e a ordem escolhidos na aparência.
    /// Com legenda, títulos ou cartão na aparência, a área de plotagem é a do
    /// layout novo (supondo a tipografia padrão); senão, a de `ap.layout`.
    pub fn eixos_com_aparencia(
        &self,
        largura: f32,
        altura: f32,
        ap: &AparenciaGrafico,
    ) -> Result<Option<Eixos>, ErroGrafico> {
        self.validar()?;
        let comp = crate::moldura::Composicao::padrao(self, ap);
        let area = self.moldura(largura, altura, ap, &comp)?.plot;
        self.eixos_na_area(area, ap)
    }

    /// Eixos dentro de uma área de plotagem já decidida.
    pub(crate) fn eixos_na_area(
        &self,
        area: Retangulo,
        ap: &AparenciaGrafico,
    ) -> Result<Option<Eixos>, ErroGrafico> {
        if self.tipo == Tipo::Pizza {
            return Ok(None);
        }
        let (cv, cc) = (&ap.eixos.valores, &ap.eixos.categorias);
        let eixo_valores = self.escala_de_valores(cv)?;
        let (escala_x, escala_y, tx, ty, zero, zero_vertical) = match self.tipo {
            Tipo::Barras => {
                let (tv, sv) = ticks_numericos(eixo_valores, cv, area.x, area.largura, false);
                (
                    Some(eixo_valores),
                    None,
                    (tv, sv),
                    (ticks_categorias(&self.categorias, area.y, area.altura, true, cc.invertido), Vec::new()),
                    area.x + area.largura * eixo_valores.fracao(0.0),
                    true,
                )
            }
            Tipo::Colunas | Tipo::Linhas | Tipo::Area | Tipo::Degraus => (
                None,
                Some(eixo_valores),
                (ticks_categorias(&self.categorias, area.x, area.largura, self.tipo == Tipo::Colunas, cc.invertido), Vec::new()),
                ticks_numericos(eixo_valores, cv, area.y, area.altura, true),
                area.y + area.altura * (1.0 - eixo_valores.fracao(0.0)),
                false,
            ),
            Tipo::Dispersao => {
                let xs: Vec<f64> = self.categorias.iter().map(|c| c.parse::<f64>().expect("validar garante eixo numérico")).collect();
                let mut ex = EscalaLinear::nova(xs.iter().copied().fold(f64::INFINITY, f64::min), xs.iter().copied().fold(f64::NEG_INFINITY, f64::max))?;
                ex.invertida = cc.invertido;
                (
                    Some(ex),
                    Some(eixo_valores),
                    ticks_numericos(ex, cc, area.x, area.largura, false),
                    ticks_numericos(eixo_valores, cv, area.y, area.altura, true),
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
            ticks_x: tx.0,
            ticks_y: ty.0,
            ticks_secundarios_x: tx.1,
            ticks_secundarios_y: ty.1,
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
