//! Estilos visuais opcionais: cantos arredondados, curva monótona, gradiente
//! de área e brilho neon. Tudo é desligado por padrão (`AparenciaGrafico::default`
//! reproduz o desenho anterior). A saída continua sendo `feather_draw::Cena`.
use crate::{AparenciaGrafico, ErroGrafico};
use serde::{Deserialize, Serialize};
use feather_draw::{
    Caminho, Cena, Comando, Cor, Estilo, EstiloVetor, Extremo, Figura, Juncao, Parada, Ponto,
    Preenchimento, Retangulo, Stroke,
};

/// Raio dos cantos de uma barra ou coluna.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RaioBarra {
    /// Raio fixo, em unidades da cena.
    Pixels(f32),
    /// Fração da espessura da barra (largura da coluna, altura da barra
    /// horizontal). `0.5` deixa a ponta totalmente redonda.
    Fracao(f32),
}

/// Quais cantos arredondam.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CantosArredondados {
    /// Só a ponta (lado do valor). Em barras empilhadas, só o segmento da ponta.
    Topo,
    /// A ponta e a base. Em empilhadas: ponta do último segmento e base do primeiro.
    TopoEBase,
    /// Os quatro cantos de todos os segmentos.
    Todos,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Arredondamento {
    pub raio: RaioBarra,
    pub onde: CantosArredondados,
}
impl Arredondamento {
    pub fn validar(&self) -> Result<(), ErroGrafico> {
        let v = match self.raio {
            RaioBarra::Pixels(v) | RaioBarra::Fracao(v) => v,
        };
        if !v.is_finite() || v < 0.0 {
            return Err(ErroGrafico::ValorInvalido);
        }
        Ok(())
    }
}

/// Posição de um segmento dentro da pilha da sua categoria.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PapelBarra {
    /// É o segmento mais afastado do zero (a ponta da pilha).
    pub ponta: bool,
    /// É o segmento encostado no zero.
    pub base: bool,
}
impl PapelBarra {
    pub const ISOLADA: Self = Self { ponta: true, base: true };
}

/// Raios dos cantos na ordem `[sup-esq, sup-dir, inf-dir, inf-esq]`.
/// `vertical`: coluna (ponta em cima para valor positivo); caso contrário
/// barra horizontal (ponta à direita para valor positivo). Para valor
/// negativo, a ponta é o lado oposto.
pub fn raios_cantos(
    area: Retangulo,
    vertical: bool,
    positivo: bool,
    papel: PapelBarra,
    ar: &Arredondamento,
) -> [f32; 4] {
    if area.largura <= 0.0 || area.altura <= 0.0 {
        return [0.0; 4];
    }
    let (espessura, comprimento) = if vertical {
        (area.largura, area.altura)
    } else {
        (area.altura, area.largura)
    };
    let (ponta, base) = match ar.onde {
        CantosArredondados::Topo => (papel.ponta, false),
        CantosArredondados::TopoEBase => (papel.ponta, papel.base),
        CantosArredondados::Todos => (true, true),
    };
    let r = match ar.raio {
        RaioBarra::Pixels(v) => v,
        RaioBarra::Fracao(f) => f * espessura,
    };
    let extremos = u32::from(ponta) + u32::from(base);
    if extremos == 0 || r <= 0.0 {
        return [0.0; 4];
    }
    let r = r.min(espessura / 2.0).min(comprimento / extremos as f32);
    let (rp, rb) = (if ponta { r } else { 0.0 }, if base { r } else { 0.0 });
    // [sup-esq, sup-dir, inf-dir, inf-esq]
    match (vertical, positivo) {
        (true, true) => [rp, rp, rb, rb],
        (true, false) => [rb, rb, rp, rp],
        (false, true) => [rb, rp, rp, rb],
        (false, false) => [rp, rb, rb, rp],
    }
}

const KAPPA: f32 = 0.552_284_8;

/// Retângulo com cantos arredondados como caminho de Béziers cúbicas.
pub fn caminho_retangulo_arredondado(a: Retangulo, r: [f32; 4]) -> Caminho {
    let [tl, tr, br, bl] = r;
    let (x, y, w, h) = (a.x, a.y, a.largura, a.altura);
    let p = Ponto::novo;
    let mut c = Caminho::novo();
    c.adicionar(Comando::Mover(p(x + tl, y)));
    c.adicionar(Comando::Linha(p(x + w - tr, y)));
    c.adicionar(Comando::Cubica {
        controle1: p(x + w - tr + tr * KAPPA, y),
        controle2: p(x + w, y + tr - tr * KAPPA),
        fim: p(x + w, y + tr),
    });
    c.adicionar(Comando::Linha(p(x + w, y + h - br)));
    c.adicionar(Comando::Cubica {
        controle1: p(x + w, y + h - br + br * KAPPA),
        controle2: p(x + w - br + br * KAPPA, y + h),
        fim: p(x + w - br, y + h),
    });
    c.adicionar(Comando::Linha(p(x + bl, y + h)));
    c.adicionar(Comando::Cubica {
        controle1: p(x + bl - bl * KAPPA, y + h),
        controle2: p(x, y + h - bl + bl * KAPPA),
        fim: p(x, y + h - bl),
    });
    c.adicionar(Comando::Linha(p(x, y + tl)));
    c.adicionar(Comando::Cubica {
        controle1: p(x, y + tl - tl * KAPPA),
        controle2: p(x + tl - tl * KAPPA, y),
        fim: p(x + tl, y),
    });
    c.adicionar(Comando::Fechar);
    c
}

/// `Retangulo` quando não há raio; caso contrário `Caminho` arredondado.
pub fn figura_barra(area: Retangulo, raios: [f32; 4], estilo: Estilo) -> Figura {
    if raios.iter().all(|r| *r <= 0.0) {
        Figura::Retangulo { area, estilo }
    } else {
        Figura::Caminho {
            caminho: caminho_retangulo_arredondado(area, raios),
            estilo: EstiloVetor::from(&estilo),
        }
    }
}

// ---------------------------------------------------------------- suavização

/// Quais séries usam curva suave.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum Suavizacao {
    #[default]
    Nenhuma,
    Todas,
    /// Índices (da série original) com curva.
    Series(Vec<usize>),
}
impl Suavizacao {
    pub fn inclui(&self, serie: usize) -> bool {
        match self {
            Self::Nenhuma => false,
            Self::Todas => true,
            Self::Series(v) => v.contains(&serie),
        }
    }
}

/// Pontos de controle (c1, c2) de cada segmento entre pontos consecutivos,
/// por interpolação cúbica monótona de Fritsch–Carlson em X crescente. A curva
/// de cada segmento nunca sai do intervalo vertical dos seus dois extremos,
/// então não ultrapassa os valores reais. Se X não for crescente num segmento,
/// ele vira reta.
pub fn controles_monotonos(p: &[Ponto]) -> Vec<(Ponto, Ponto)> {
    let n = p.len();
    if n < 2 {
        return Vec::new();
    }
    let h: Vec<f64> = p.windows(2).map(|w| f64::from(w[1].x - w[0].x)).collect();
    let d: Vec<f64> = p
        .windows(2)
        .zip(&h)
        .map(|(w, h)| if *h > 0.0 { f64::from(w[1].y - w[0].y) / h } else { 0.0 })
        .collect();
    let mut m = vec![0.0_f64; n];
    m[0] = d[0];
    m[n - 1] = d[n - 2];
    for k in 1..n - 1 {
        m[k] = if d[k - 1] * d[k] <= 0.0 { 0.0 } else { (d[k - 1] + d[k]) / 2.0 };
    }
    for k in 0..n - 1 {
        if d[k] == 0.0 {
            m[k] = 0.0;
            m[k + 1] = 0.0;
            continue;
        }
        let (a, b) = (m[k] / d[k], m[k + 1] / d[k]);
        let s = a * a + b * b;
        if s > 9.0 {
            let t = 3.0 / s.sqrt();
            m[k] = t * a * d[k];
            m[k + 1] = t * b * d[k];
        }
    }
    (0..n - 1)
        .map(|k| {
            if h[k] <= 0.0 {
                return (p[k], p[k + 1]);
            }
            let hk = h[k] as f32;
            (
                Ponto::novo(p[k].x + hk / 3.0, p[k].y + (m[k] as f32) * hk / 3.0),
                Ponto::novo(p[k + 1].x - hk / 3.0, p[k + 1].y - (m[k + 1] as f32) * hk / 3.0),
            )
        })
        .collect()
}

/// Comandos de caminho passando por todos os pontos: retas ou curva monótona.
pub fn comandos_linha(pontos: &[Ponto], suave: bool) -> Vec<Comando> {
    let mut cmds = Vec::with_capacity(pontos.len() + 1);
    let Some(primeiro) = pontos.first() else { return cmds };
    cmds.push(Comando::Mover(*primeiro));
    if suave && pontos.len() >= 3 {
        for (par, (c1, c2)) in pontos.windows(2).zip(controles_monotonos(pontos)) {
            cmds.push(Comando::Cubica { controle1: c1, controle2: c2, fim: par[1] });
        }
    } else {
        cmds.extend(pontos[1..].iter().map(|p| Comando::Linha(*p)));
    }
    cmds
}

// ----------------------------------------------------------------- gradiente

/// Preenchimento vertical da área: da opacidade inicial (junto à série) até a
/// final (junto à linha de base). Multiplica o alfa da cor da série.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GradienteArea {
    pub opacidade_inicial: f32,
    pub opacidade_final: f32,
    /// 0 = gradiente linear nativo da cena (SVG). O PDF do `feather-draw`
    /// recusa gradiente com opacidade variável; com 2 ou mais, a área é
    /// desenhada em tantas faixas horizontais de opacidade decrescente,
    /// recortadas pelo contorno, que saem iguais em SVG, PDF e telas.
    /// Os exportadores de PDF `*_estilizado` usam 32 faixas sozinhos.
    pub faixas: u8,
}
impl Default for GradienteArea {
    fn default() -> Self {
        Self { opacidade_inicial: 0.6, opacidade_final: 0.0, faixas: 0 }
    }
}
impl GradienteArea {
    pub fn validar(&self) -> Result<(), ErroGrafico> {
        let ok = |v: f32| v.is_finite() && (0.0..=1.0).contains(&v);
        if ok(self.opacidade_inicial) && ok(self.opacidade_final) && self.faixas != 1 {
            Ok(())
        } else {
            Err(ErroGrafico::ValorInvalido)
        }
    }
    /// `None` se a série é rasa demais (topo igual à base) para ter gradiente.
    pub fn preenchimento(&self, topo_y: f32, base_y: f32, cor: Cor) -> Option<Preenchimento> {
        if (topo_y - base_y).abs() < 1e-3 {
            return None;
        }
        let com = |o: f32| {
            Cor::rgba(cor.r, cor.g, cor.b, (f32::from(cor.a) * o).round().clamp(0.0, 255.0) as u8)
        };
        Some(Preenchimento::Linear {
            de: Ponto::novo(0.0, topo_y),
            ate: Ponto::novo(0.0, base_y),
            paradas: vec![
                Parada { posicao: 0.0, cor: com(self.opacidade_inicial) },
                Parada { posicao: 1.0, cor: com(self.opacidade_final) },
            ],
        })
    }
}

// ---------------------------------------------------------------------- neon

/// Brilho: a mesma geometria desenhada `camadas` vezes mais grossa e quase
/// transparente antes do traço normal.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Neon {
    /// 0 a 1; opacidade total do brilho.
    pub intensidade: f32,
    /// Largura extra da camada mais externa, em unidades da cena.
    pub espessura: f32,
    /// 1 a 6 camadas concêntricas.
    pub camadas: u8,
}
impl Default for Neon {
    fn default() -> Self {
        Self { intensidade: 0.8, espessura: 8.0, camadas: 3 }
    }
}
impl Neon {
    pub fn validar(&self) -> Result<(), ErroGrafico> {
        if !self.intensidade.is_finite()
            || !(0.0..=1.0).contains(&self.intensidade)
            || !self.espessura.is_finite()
            || self.espessura < 0.0
            || !(1..=6).contains(&self.camadas)
        {
            return Err(ErroGrafico::ValorInvalido);
        }
        Ok(())
    }
    /// (largura, cor) da mais externa para a mais interna.
    pub fn camadas_de(&self, base: f32, cor: Cor) -> Vec<(f32, Cor)> {
        let n = self.camadas as usize;
        (1..=n)
            .rev()
            .map(|k| {
                let largura = base + self.espessura * k as f32 / n as f32;
                let alfa = (f32::from(cor.a) * self.intensidade * 0.6 / n as f32)
                    .round()
                    .clamp(1.0, 255.0) as u8;
                (largura, Cor::rgba(cor.r, cor.g, cor.b, alfa))
            })
            .collect()
    }
}

fn estilo_traco(cor: Cor, largura: f32) -> EstiloVetor {
    EstiloVetor {
        preenchimento: None,
        stroke: Some(Stroke {
            extremo: Extremo::Redondo,
            juncao: Juncao::Redonda,
            ..Stroke::novo(cor, largura)
        }),
        ..EstiloVetor::default()
    }
}

/// Camadas de brilho de um caminho aberto.
pub fn brilho_caminho(cena: &mut Cena, cmds: &[Comando], base: f32, cor: Cor, neon: &Neon) {
    for (largura, c) in neon.camadas_de(base, cor) {
        cena.adicionar(Figura::Caminho {
            caminho: Caminho { comandos: cmds.to_vec(), ..Caminho::novo() },
            estilo: estilo_traco(c, largura),
        });
    }
}

/// Camadas de brilho do contorno de uma barra (retangular ou arredondada).
pub fn brilho_barra(cena: &mut Cena, area: Retangulo, raios: [f32; 4], base: f32, cor: Cor, neon: &Neon) {
    for (largura, c) in neon.camadas_de(base, cor) {
        cena.adicionar(figura_barra(
            area,
            raios,
            Estilo { preenchimento: None, contorno: Some(c), espessura: largura },
        ));
    }
}

// ------------------------------------------------- linha/área estilizadas

/// Parâmetros de uma série de linha ou área desenhada com estilo.
pub struct SerieLinha<'a> {
    pub cor: Cor,
    pub indice_serie: usize,
    pub baseline: f32,
    pub area: bool,
    /// Pode desligar a curva (degraus nunca são suavizados).
    pub permite_suavizar: bool,
    pub aparencia: &'a AparenciaGrafico,
}

impl AparenciaGrafico {
    /// Há algum estilo de linha que exige o caminho estilizado para a série.
    pub fn estiliza_linha(&self, indice_serie: usize, area: bool, permite_suavizar: bool) -> bool {
        (permite_suavizar && self.suavizacao.inclui(indice_serie))
            || (area && self.gradiente_area.is_some())
            || self.neon.is_some()
            || self.serie_personalizada(indice_serie)
    }
}

/// Desenha preenchimento (área), brilho e traço de uma série. `pontos` já é o
/// caminho final (com degraus, se for o caso).
pub fn desenhar_serie_linha(cena: &mut Cena, pontos: &[Ponto], s: &SerieLinha) {
    let (Some(primeiro), Some(ultimo)) = (pontos.first().copied(), pontos.last().copied()) else {
        return;
    };
    let ap = s.aparencia;
    let suave = s.permite_suavizar && ap.suavizacao.inclui(s.indice_serie) && pontos.len() >= 3;
    let linha = comandos_linha(pontos, suave);
    if s.area {
        let mut cmds = vec![Comando::Mover(Ponto::novo(primeiro.x, s.baseline))];
        cmds.push(Comando::Linha(primeiro));
        cmds.extend(linha.iter().skip(1).copied());
        cmds.push(Comando::Linha(Ponto::novo(ultimo.x, s.baseline)));
        cmds.push(Comando::Fechar);
        let topo = pontos
            .iter()
            .map(|p| p.y)
            .max_by(|a, b| (a - s.baseline).abs().total_cmp(&(b - s.baseline).abs()))
            .unwrap_or(s.baseline);
        let contorno = Caminho { comandos: cmds, ..Caminho::novo() };
        let faixas = ap.gradiente_area.filter(|g| g.faixas >= 2 && (topo - s.baseline).abs() >= 1e-3);
        if let Some(g) = faixas {
            // Faixas de opacidade decrescente, recortadas pelo contorno da área.
            let (xmin, xmax) = pontos.iter().fold((f32::MAX, f32::MIN), |(a, b), p| (a.min(p.x), b.max(p.x)));
            let n = usize::from(g.faixas);
            for k in 0..n {
                let (y0, y1) = (
                    topo + (s.baseline - topo) * k as f32 / n as f32,
                    topo + (s.baseline - topo) * (k + 1) as f32 / n as f32,
                );
                let t = (k as f32 + 0.5) / n as f32;
                let o = g.opacidade_inicial + (g.opacidade_final - g.opacidade_inicial) * t;
                let alfa = (f32::from(s.cor.a) * o).round().clamp(0.0, 255.0) as u8;
                if alfa == 0 {
                    continue;
                }
                cena.adicionar(Figura::Recorte {
                    figura: Box::new(Figura::Retangulo {
                        area: Retangulo::novo(xmin, y0.min(y1), (xmax - xmin).max(0.0), (y1 - y0).abs()),
                        estilo: Estilo {
                            preenchimento: Some(Cor::rgba(s.cor.r, s.cor.g, s.cor.b, alfa)),
                            contorno: None,
                            espessura: 0.0,
                        },
                    }),
                    recorte: contorno.clone(),
                });
            }
        } else {
            let preench = ap
                .gradiente_area
                .and_then(|g| g.preenchimento(topo, s.baseline, s.cor))
                .unwrap_or_else(|| {
                    let alfa = ((f32::from(s.cor.a) * 0.35).round() as u8).max(1);
                    Preenchimento::Solido(Cor::rgba(s.cor.r, s.cor.g, s.cor.b, alfa))
                });
            cena.adicionar(Figura::Caminho {
                caminho: contorno,
                estilo: EstiloVetor { preenchimento: Some(preench), stroke: None, ..EstiloVetor::default() },
            });
        }
    }
    let espessura = ap.espessura_da_serie(s.indice_serie);
    if let Some(neon) = &ap.neon {
        brilho_caminho(cena, &linha, espessura, s.cor, neon);
    }
    let mut estilo = estilo_traco(s.cor, espessura);
    let traco = ap.traco_da_serie(s.indice_serie);
    if let Some(st) = estilo.stroke.as_mut().filter(|_| traco != crate::personalizacao::TracoLinha::Continuo) {
        st.dash = traco.padrao(espessura);
        st.extremo = Extremo::Plano;
    }
    cena.adicionar(Figura::Caminho {
        caminho: Caminho { comandos: linha, ..Caminho::novo() },
        estilo,
    });
}

#[cfg(test)]
mod testes {
    use super::*;

    fn amostra(p0: Ponto, c1: Ponto, c2: Ponto, p1: Ponto, t: f32) -> Ponto {
        let u = 1.0 - t;
        Ponto::novo(
            u * u * u * p0.x + 3.0 * u * u * t * c1.x + 3.0 * u * t * t * c2.x + t * t * t * p1.x,
            u * u * u * p0.y + 3.0 * u * u * t * c1.y + 3.0 * u * t * t * c2.y + t * t * t * p1.y,
        )
    }

    #[test]
    fn curva_monotona_nao_ultrapassa_os_valores() {
        let ys = [0.0_f32, 10.0, 10.0, 0.0, 5.0, 5.0, 40.0, 38.0];
        let pts: Vec<_> = ys.iter().enumerate().map(|(i, y)| Ponto::novo(i as f32 * 20.0, *y)).collect();
        let ctrl = controles_monotonos(&pts);
        assert_eq!(ctrl.len(), pts.len() - 1);
        for (k, (c1, c2)) in ctrl.iter().enumerate() {
            let (lo, hi) = (pts[k].y.min(pts[k + 1].y), pts[k].y.max(pts[k + 1].y));
            for i in 0..=50 {
                let q = amostra(pts[k], *c1, *c2, pts[k + 1], i as f32 / 50.0);
                assert!(q.y >= lo - 1e-3 && q.y <= hi + 1e-3, "segmento {k} passou de [{lo},{hi}]: {}", q.y);
                assert!(q.x >= pts[k].x - 1e-3 && q.x <= pts[k + 1].x + 1e-3);
            }
        }
    }

    #[test]
    fn comandos_linha_passam_pelos_pontos() {
        let pts = [Ponto::novo(0.0, 5.0), Ponto::novo(10.0, 1.0), Ponto::novo(20.0, 9.0)];
        let reta = comandos_linha(&pts, false);
        assert!(matches!(reta[1], Comando::Linha(_)));
        let curva = comandos_linha(&pts, true);
        assert_eq!(curva.len(), 3);
        assert!(matches!(curva[2], Comando::Cubica { fim, .. } if fim == pts[2]));
    }

    #[test]
    fn raios_so_na_ponta_inclusive_para_negativos() {
        let a = Retangulo::novo(0.0, 0.0, 20.0, 50.0);
        let ar = Arredondamento { raio: RaioBarra::Pixels(6.0), onde: CantosArredondados::Topo };
        assert_eq!(raios_cantos(a, true, true, PapelBarra::ISOLADA, &ar), [6.0, 6.0, 0.0, 0.0]);
        assert_eq!(raios_cantos(a, true, false, PapelBarra::ISOLADA, &ar), [0.0, 0.0, 6.0, 6.0]);
        // barra horizontal: ponta à direita (positivo) ou à esquerda (negativo)
        let h = Retangulo::novo(0.0, 0.0, 50.0, 20.0);
        assert_eq!(raios_cantos(h, false, true, PapelBarra::ISOLADA, &ar), [0.0, 6.0, 6.0, 0.0]);
        assert_eq!(raios_cantos(h, false, false, PapelBarra::ISOLADA, &ar), [6.0, 0.0, 0.0, 6.0]);
        // segmento do meio de uma pilha não arredonda no modo Topo
        let meio = PapelBarra { ponta: false, base: false };
        assert_eq!(raios_cantos(a, true, true, meio, &ar), [0.0; 4]);
        // Todos arredonda até o do meio
        let todos = Arredondamento { onde: CantosArredondados::Todos, ..ar };
        assert_eq!(raios_cantos(a, true, true, meio, &todos), [6.0; 4]);
        // fração da largura e limite pela metade da espessura
        let fr = Arredondamento { raio: RaioBarra::Fracao(0.25), onde: CantosArredondados::Topo };
        assert_eq!(raios_cantos(a, true, true, PapelBarra::ISOLADA, &fr)[0], 5.0);
        let grande = Arredondamento { raio: RaioBarra::Pixels(99.0), onde: CantosArredondados::Topo };
        assert_eq!(raios_cantos(a, true, true, PapelBarra::ISOLADA, &grande)[0], 10.0);
        // barra baixa: raio limitado pela altura
        let baixa = Retangulo::novo(0.0, 0.0, 20.0, 3.0);
        assert_eq!(raios_cantos(baixa, true, true, PapelBarra::ISOLADA, &grande)[0], 3.0);
    }

    #[test]
    fn figura_sem_raio_continua_retangulo() {
        let a = Retangulo::novo(1.0, 2.0, 3.0, 4.0);
        let e = Estilo::default();
        assert!(matches!(figura_barra(a, [0.0; 4], e.clone()), Figura::Retangulo { .. }));
        assert!(matches!(figura_barra(a, [1.0, 1.0, 0.0, 0.0], e), Figura::Caminho { .. }));
    }

    #[test]
    fn neon_camadas_ficam_mais_finas_e_validam() {
        let n = Neon { intensidade: 1.0, espessura: 9.0, camadas: 3 };
        let c = n.camadas_de(2.0, Cor::rgb(0, 255, 255));
        assert_eq!(c.len(), 3);
        assert!(c[0].0 > c[1].0 && c[1].0 > c[2].0 && c[2].0 > 2.0);
        assert!(c.iter().all(|(_, cor)| cor.a < 255));
        assert!(Neon { camadas: 0, ..n }.validar().is_err());
        assert!(Neon { intensidade: 2.0, ..n }.validar().is_err());
        assert!(n.validar().is_ok());
    }

    #[test]
    fn gradiente_vai_da_cor_ate_transparente() {
        let g = GradienteArea { opacidade_inicial: 0.8, opacidade_final: 0.0, faixas: 0 };
        match g.preenchimento(10.0, 100.0, Cor::rgb(10, 20, 30)).unwrap() {
            Preenchimento::Linear { paradas, de, ate } => {
                assert_eq!((de.y, ate.y), (10.0, 100.0));
                assert_eq!(paradas[0].cor.a, 204);
                assert_eq!(paradas[1].cor.a, 0);
            }
            _ => panic!(),
        }
        assert!(g.preenchimento(5.0, 5.0, Cor::rgb(0, 0, 0)).is_none());
    }
}
