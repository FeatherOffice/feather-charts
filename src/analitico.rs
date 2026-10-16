//! Elementos analíticos de uma série: linha de tendência (com equação e R²),
//! barras de erro e linhas de máximo, mínimo e média.
//!
//! Aqui ficam o modelo (vai no [`crate::estilo::EstiloSerie`], serializável)
//! e as contas; o desenho está em [`crate::formatado`]. Como no Excel, nos
//! gráficos de categorias o X da tendência é a posição do ponto (1, 2, 3…);
//! na dispersão é o X de verdade.
use crate::estilo::{CorEstilo, Traco};
use serde::{Deserialize, Serialize};

/// Tipo da linha de tendência.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TipoTendencia {
    #[default]
    Linear,
    Exponencial,
    Logaritmica,
    /// Grau 2 a 6, em [`Tendencia::grau`].
    Polinomial,
    Potencia,
    /// Período em [`Tendencia::periodo`].
    MediaMovel,
}
impl TipoTendencia {
    pub const TODOS: [Self; 6] = [
        Self::Linear, Self::Exponencial, Self::Logaritmica, Self::Polinomial, Self::Potencia, Self::MediaMovel,
    ];
    pub fn nome(self) -> &'static str {
        match self {
            Self::Linear => "Linear",
            Self::Exponencial => "Exponencial",
            Self::Logaritmica => "Logarítmica",
            Self::Polinomial => "Polinomial",
            Self::Potencia => "Potência",
            Self::MediaMovel => "Média móvel",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Tendencia {
    pub tipo: TipoTendencia,
    /// Polinomial: 2 a 6.
    pub grau: u8,
    /// Média móvel: 2 ou mais pontos.
    pub periodo: u8,
    /// Estende a linha para a frente e para trás, em unidades de X.
    pub avancar: f64,
    pub recuar: f64,
    pub mostrar_equacao: bool,
    pub mostrar_r2: bool,
    /// `None` = cor da série.
    pub cor: Option<CorEstilo>,
    pub espessura: Option<f32>,
    pub traco: Option<Traco>,
}
impl Default for Tendencia {
    fn default() -> Self {
        Self {
            tipo: TipoTendencia::Linear, grau: 2, periodo: 2, avancar: 0.0, recuar: 0.0,
            mostrar_equacao: false, mostrar_r2: false, cor: None, espessura: None, traco: None,
        }
    }
}

/// Quanto cada barra de erro se estende.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "tipo", content = "valor")]
pub enum QuantiaErro {
    /// Valor fixo, nas unidades do eixo.
    Fixo(f64),
    /// Porcentagem do próprio valor.
    Percentual(f64),
    /// Múltiplos do desvio-padrão da série (centrado na média, como no Excel).
    DesvioPadrao(f64),
    /// Erro-padrão da série.
    ErroPadrao,
}
impl Default for QuantiaErro {
    fn default() -> Self {
        Self::Percentual(5.0)
    }
}
impl QuantiaErro {
    pub fn nome(self) -> &'static str {
        match self {
            Self::Fixo(_) => "Valor fixo",
            Self::Percentual(_) => "Porcentagem",
            Self::DesvioPadrao(_) => "Desvio-padrão",
            Self::ErroPadrao => "Erro-padrão",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DirecaoErro {
    #[default]
    Ambas,
    Mais,
    Menos,
}
impl DirecaoErro {
    pub const TODAS: [Self; 3] = [Self::Ambas, Self::Mais, Self::Menos];
    pub fn nome(self) -> &'static str {
        match self {
            Self::Ambas => "Ambas",
            Self::Mais => "Mais",
            Self::Menos => "Menos",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BarrasErro {
    pub quantia: QuantiaErro,
    pub direcao: DirecaoErro,
    /// Sem o tracinho na ponta.
    pub sem_ponta: bool,
    pub cor: Option<CorEstilo>,
    pub espessura: Option<f32>,
}

/// Tudo que se acrescenta por cima de uma série.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Analise {
    pub tendencia: Option<Tendencia>,
    pub erro: Option<BarrasErro>,
    pub maximo: bool,
    pub minimo: bool,
    pub media: bool,
}
impl Analise {
    pub fn vazia(&self) -> bool {
        self.tendencia.is_none() && self.erro.is_none() && !self.maximo && !self.minimo && !self.media
    }
}

// ----------------------------------------------------------------- contas

/// Curva ajustada: avalia em X, guarda a equação e o R².
#[derive(Debug, Clone, PartialEq)]
pub enum Ajuste {
    /// y = a + b·x
    Linear { a: f64, b: f64, r2: f64 },
    /// y = a·e^(b·x)
    Exponencial { a: f64, b: f64, r2: f64 },
    /// y = a + b·ln(x)
    Logaritmica { a: f64, b: f64, r2: f64 },
    /// y = c0 + c1·x + … ; `c[k]` multiplica x^k
    Polinomial { c: Vec<f64>, r2: f64 },
    /// y = a·x^b
    Potencia { a: f64, b: f64, r2: f64 },
}

impl Ajuste {
    pub fn valor(&self, x: f64) -> f64 {
        match self {
            Self::Linear { a, b, .. } => a + b * x,
            Self::Exponencial { a, b, .. } => a * (b * x).exp(),
            Self::Logaritmica { a, b, .. } => a + b * x.ln(),
            Self::Polinomial { c, .. } => c.iter().rev().fold(0.0, |acc, k| acc * x + k),
            Self::Potencia { a, b, .. } => a * x.powf(*b),
        }
    }
    pub fn r2(&self) -> f64 {
        match self {
            Self::Linear { r2, .. } | Self::Exponencial { r2, .. } | Self::Logaritmica { r2, .. }
            | Self::Polinomial { r2, .. } | Self::Potencia { r2, .. } => *r2,
        }
    }
    /// Equação no jeito do Excel em pt-BR (vírgula decimal).
    pub fn equacao(&self) -> String {
        let n = |v: f64| num(v);
        let soma = |v: f64| if v < 0.0 { format!(" − {}", num(-v)) } else { format!(" + {}", num(v)) };
        match self {
            Self::Linear { a, b, .. } => format!("y = {}x{}", n(*b), soma(*a)),
            Self::Exponencial { a, b, .. } => format!("y = {}e^({}x)", n(*a), n(*b)),
            Self::Logaritmica { a, b, .. } => format!("y = {}ln(x){}", n(*b), soma(*a)),
            Self::Potencia { a, b, .. } => format!("y = {}x^{}", n(*a), n(*b)),
            Self::Polinomial { c, .. } => {
                let mut s = String::from("y =");
                let mut primeiro = true;
                for (k, &v) in c.iter().enumerate().rev() {
                    if v == 0.0 && c.len() > 1 {
                        continue;
                    }
                    let termo = match k {
                        0 => String::new(),
                        1 => "x".into(),
                        _ => format!("x^{k}"),
                    };
                    if primeiro {
                        s.push_str(&format!(" {}{termo}", n(v)));
                        primeiro = false;
                    } else {
                        s.push_str(&format!("{}{termo}", soma(v)));
                    }
                }
                s
            }
        }
    }
}

/// Até 4 algarismos significativos, vírgula decimal.
fn num(v: f64) -> String {
    if v == 0.0 || !v.is_finite() {
        return "0".into();
    }
    let ordem = v.abs().log10().floor() as i32;
    if !(-4..=6).contains(&ordem) {
        return format!("{v:.3e}").replace('.', ",");
    }
    let casas = (3 - ordem).clamp(0, 8) as usize;
    let s = format!("{v:.casas$}");
    let s = if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_owned() } else { s };
    s.replace('.', ",")
}

fn r2_de(xs: &[f64], ys: &[f64], f: impl Fn(f64) -> f64) -> f64 {
    let media = ys.iter().sum::<f64>() / ys.len() as f64;
    let total: f64 = ys.iter().map(|y| (y - media).powi(2)).sum();
    let resto: f64 = xs.iter().zip(ys).map(|(x, y)| (y - f(*x)).powi(2)).sum();
    if total == 0.0 { 1.0 } else { 1.0 - resto / total }
}

/// Mínimos quadrados de y em x: (a, b) de y = a + b·x.
fn reta(xs: &[f64], ys: &[f64]) -> Option<(f64, f64)> {
    let n = xs.len() as f64;
    if xs.len() < 2 {
        return None;
    }
    let (sx, sy) = (xs.iter().sum::<f64>(), ys.iter().sum::<f64>());
    let sxx: f64 = xs.iter().map(|x| x * x).sum();
    let sxy: f64 = xs.iter().zip(ys).map(|(x, y)| x * y).sum();
    let det = n * sxx - sx * sx;
    if det.abs() < 1e-12 {
        return None;
    }
    let b = (n * sxy - sx * sy) / det;
    Some(((sy - b * sx) / n, b))
}

/// Polinômio de grau `g` por equações normais (eliminação de Gauss com pivô).
fn polinomio(xs: &[f64], ys: &[f64], g: usize) -> Option<Vec<f64>> {
    let m = g + 1;
    if xs.len() < m {
        return None;
    }
    let mut a = vec![vec![0.0; m + 1]; m];
    for (x, y) in xs.iter().zip(ys) {
        let pot: Vec<f64> = (0..=2 * g).map(|k| x.powi(k as i32)).collect();
        for i in 0..m {
            for j in 0..m {
                a[i][j] += pot[i + j];
            }
            a[i][m] += y * pot[i];
        }
    }
    for col in 0..m {
        let piv = (col..m).max_by(|&p, &q| a[p][col].abs().total_cmp(&a[q][col].abs()))?;
        if a[piv][col].abs() < 1e-12 {
            return None;
        }
        a.swap(col, piv);
        for lin in 0..m {
            if lin != col {
                let f = a[lin][col] / a[col][col];
                for k in col..=m {
                    a[lin][k] -= f * a[col][k];
                }
            }
        }
    }
    Some((0..m).map(|i| a[i][m] / a[i][i]).collect())
}

/// Ajusta a curva pedida. `None` quando os dados não servem (poucos pontos,
/// X ou Y não positivos onde há logaritmo) — aí a linha simplesmente não aparece.
pub fn ajustar(t: &Tendencia, xs: &[f64], ys: &[f64]) -> Option<Ajuste> {
    let pares: Vec<(f64, f64)> = xs.iter().zip(ys).filter(|(x, y)| x.is_finite() && y.is_finite()).map(|(x, y)| (*x, *y)).collect();
    let (xs, ys): (Vec<f64>, Vec<f64>) = pares.into_iter().unzip();
    let ajuste = match t.tipo {
        TipoTendencia::Linear => {
            let (a, b) = reta(&xs, &ys)?;
            Ajuste::Linear { a, b, r2: r2_de(&xs, &ys, |x| a + b * x) }
        }
        TipoTendencia::Exponencial => {
            if ys.iter().any(|y| *y <= 0.0) {
                return None;
            }
            let ln: Vec<f64> = ys.iter().map(|y| y.ln()).collect();
            let (la, b) = reta(&xs, &ln)?;
            // R² no espaço linearizado, como o Excel.
            Ajuste::Exponencial { a: la.exp(), b, r2: r2_de(&xs, &ln, |x| la + b * x) }
        }
        TipoTendencia::Logaritmica => {
            if xs.iter().any(|x| *x <= 0.0) {
                return None;
            }
            let lx: Vec<f64> = xs.iter().map(|x| x.ln()).collect();
            let (a, b) = reta(&lx, &ys)?;
            Ajuste::Logaritmica { a, b, r2: r2_de(&lx, &ys, |x| a + b * x) }
        }
        TipoTendencia::Potencia => {
            if xs.iter().chain(&ys).any(|v| *v <= 0.0) {
                return None;
            }
            let lx: Vec<f64> = xs.iter().map(|x| x.ln()).collect();
            let ly: Vec<f64> = ys.iter().map(|y| y.ln()).collect();
            let (la, b) = reta(&lx, &ly)?;
            Ajuste::Potencia { a: la.exp(), b, r2: r2_de(&lx, &ly, |x| la + b * x) }
        }
        TipoTendencia::Polinomial => {
            let g = usize::from(t.grau.clamp(2, 6));
            let c = polinomio(&xs, &ys, g)?;
            let f = |x: f64| c.iter().rev().fold(0.0, |acc, k| acc * x + k);
            let r2 = r2_de(&xs, &ys, f);
            Ajuste::Polinomial { c, r2 }
        }
        TipoTendencia::MediaMovel => return None,
    };
    Some(ajuste)
}

/// Média móvel: um ponto a partir do `periodo`-ésimo, na mesma posição de X.
pub fn media_movel(xs: &[f64], ys: &[f64], periodo: u8) -> Vec<(f64, f64)> {
    let p = usize::from(periodo.max(2));
    if ys.len() < p {
        return Vec::new();
    }
    (p - 1..ys.len()).map(|i| (xs[i], ys[i + 1 - p..=i].iter().sum::<f64>() / p as f64)).collect()
}

/// (para baixo, para cima) de cada ponto, já com a direção aplicada.
pub fn extensao_erro(b: &BarrasErro, ys: &[f64]) -> Vec<(f64, f64)> {
    let n = ys.len() as f64;
    let media = ys.iter().sum::<f64>() / n.max(1.0);
    let dp = if ys.len() > 1 { (ys.iter().map(|y| (y - media).powi(2)).sum::<f64>() / (n - 1.0)).sqrt() } else { 0.0 };
    let quanto = |y: f64| -> f64 {
        match b.quantia {
            QuantiaErro::Fixo(v) => v.abs(),
            QuantiaErro::Percentual(p) => (y * p / 100.0).abs(),
            QuantiaErro::DesvioPadrao(k) => dp * k.abs(),
            QuantiaErro::ErroPadrao => if n > 0.0 { dp / n.sqrt() } else { 0.0 },
        }
    };
    ys.iter().map(|&y| {
        let q = quanto(y);
        // Desvio-padrão no Excel: a barra parte da média, não do ponto.
        let centro = if matches!(b.quantia, QuantiaErro::DesvioPadrao(_)) { media } else { y };
        let (baixo, cima) = (centro - q, centro + q);
        match b.direcao {
            DirecaoErro::Ambas => (baixo, cima),
            DirecaoErro::Mais => (y, cima),
            DirecaoErro::Menos => (baixo, y),
        }
    }).collect()
}

#[cfg(test)]
mod testes {
    use super::*;

    fn t(tipo: TipoTendencia) -> Tendencia {
        Tendencia { tipo, ..Default::default() }
    }

    #[test]
    fn reta_exata_tem_r2_um_e_equacao_em_pt() {
        let xs = [1.0, 2.0, 3.0, 4.0];
        let ys = [3.0, 5.0, 7.0, 9.0];
        let a = ajustar(&t(TipoTendencia::Linear), &xs, &ys).unwrap();
        assert!((a.valor(10.0) - 21.0).abs() < 1e-9);
        assert!((a.r2() - 1.0).abs() < 1e-12);
        assert_eq!(a.equacao(), "y = 2x + 1");
    }

    #[test]
    fn exponencial_logaritmica_potencia_e_polinomial() {
        let xs: Vec<f64> = (1..=6).map(f64::from).collect();
        let exp: Vec<f64> = xs.iter().map(|x| 2.0 * (0.5 * x).exp()).collect();
        let a = ajustar(&t(TipoTendencia::Exponencial), &xs, &exp).unwrap();
        assert!((a.valor(3.0) - exp[2]).abs() < 1e-6);
        let log: Vec<f64> = xs.iter().map(|x| 1.0 + 3.0 * x.ln()).collect();
        assert!((ajustar(&t(TipoTendencia::Logaritmica), &xs, &log).unwrap().valor(5.0) - log[4]).abs() < 1e-9);
        let pot: Vec<f64> = xs.iter().map(|x| 4.0 * x.powf(1.5)).collect();
        assert!((ajustar(&t(TipoTendencia::Potencia), &xs, &pot).unwrap().valor(2.0) - pot[1]).abs() < 1e-6);
        let quad: Vec<f64> = xs.iter().map(|x| x * x - 2.0 * x + 3.0).collect();
        let p = ajustar(&Tendencia { tipo: TipoTendencia::Polinomial, grau: 2, ..Default::default() }, &xs, &quad).unwrap();
        assert!((p.valor(7.0) - 38.0).abs() < 1e-6);
        assert_eq!(p.equacao(), "y = 1x^2 − 2x + 3");
    }

    #[test]
    fn dados_impossiveis_nao_quebram() {
        assert!(ajustar(&t(TipoTendencia::Exponencial), &[1.0, 2.0], &[1.0, -1.0]).is_none());
        assert!(ajustar(&t(TipoTendencia::Linear), &[1.0], &[1.0]).is_none());
        assert!(ajustar(&t(TipoTendencia::Linear), &[1.0, 1.0], &[1.0, 2.0]).is_none());
    }

    #[test]
    fn media_movel_e_barras_de_erro() {
        let mm = media_movel(&[1.0, 2.0, 3.0, 4.0], &[2.0, 4.0, 6.0, 8.0], 2);
        assert_eq!(mm, vec![(2.0, 3.0), (3.0, 5.0), (4.0, 7.0)]);
        let b = BarrasErro { quantia: QuantiaErro::Percentual(10.0), ..Default::default() };
        assert_eq!(extensao_erro(&b, &[100.0]), vec![(90.0, 110.0)]);
        let b = BarrasErro { quantia: QuantiaErro::Fixo(1.0), direcao: DirecaoErro::Mais, ..Default::default() };
        assert_eq!(extensao_erro(&b, &[5.0]), vec![(5.0, 6.0)]);
    }

    #[test]
    fn analise_serializa_e_volta() {
        let a = Analise { tendencia: Some(Tendencia { mostrar_r2: true, ..Default::default() }),
            erro: Some(BarrasErro { quantia: QuantiaErro::DesvioPadrao(1.0), ..Default::default() }), maximo: true, ..Default::default() };
        let j = serde_json::to_string(&a).unwrap();
        assert_eq!(serde_json::from_str::<Analise>(&j).unwrap(), a);
    }
}
