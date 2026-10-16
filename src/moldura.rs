//! Layout do quadro: onde ficam título, subtítulo, legenda, títulos de eixo e
//! a área de plotagem. Sem nenhuma opção nova (`AparenciaGrafico::usa_moldura`
//! falso) o layout antigo, por frações (`LayoutGrafico`), continua valendo e
//! a saída é idêntica à anterior. Com ela, a área de plotagem encolhe para
//! caber cada elemento, sem a legenda cair em cima dos rótulos do eixo X.
use crate::personalizacao::ConfigEixo;
use crate::quadro::PosicaoLegenda;
use crate::tipografia::{FormatoNumero, LocalidadeGrafico, TipografiaGrafico};
use crate::{AparenciaGrafico, ErroGrafico, Grafico, Tipo};
use feather_draw::Retangulo;

/// O que vai ser escrito em volta do gráfico. Vem da `TipografiaGrafico` em
/// `cena_anotada_*`; nas funções só de geometria usa os valores padrão.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Composicao {
    pub fonte: f32,
    pub titulo: bool,
    pub subtitulo: bool,
    pub unidade: bool,
    pub eixos: bool,
    /// Nomes dos itens da legenda (vazio = sem legenda).
    pub legenda: Vec<String>,
    pub formato: FormatoNumero,
    pub localidade: LocalidadeGrafico,
}

impl Composicao {
    /// `original`: o gráfico sem filtro (a legenda lista todos os itens).
    pub fn da_tipografia(original: &Grafico, cfg: &TipografiaGrafico, pizza_por_categoria: bool) -> Self {
        Self {
            fonte: cfg.tamanho,
            titulo: cfg.mostrar_titulo && !original.titulo.is_empty(),
            subtitulo: !cfg.subtitulo.is_empty(),
            unidade: cfg.mostrar_eixos && !cfg.unidade.is_empty(),
            eixos: cfg.mostrar_eixos,
            legenda: if cfg.mostrar_legenda {
                original.nomes_legenda(pizza_por_categoria)
            } else {
                Vec::new()
            },
            formato: cfg.formato,
            localidade: cfg.localidade,
        }
    }
    /// Geometria sem tipografia: supõe a tipografia padrão (fonte 10).
    pub fn padrao(original: &Grafico, ap: &AparenciaGrafico) -> Self {
        let cfg = TipografiaGrafico::default();
        let mut c = Self::da_tipografia(original, &cfg, true);
        c.subtitulo = ap.subtitulo.is_some();
        c
    }
}

/// Resultado do layout. Coordenadas da cena; `y` dos textos é a baseline.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Moldura {
    /// Layout antigo: nenhuma das posições abaixo vale, só `plot`.
    pub legado: bool,
    pub cartao: Retangulo,
    /// Cartão menos as margens internas: onde os textos se alinham.
    pub conteudo: Retangulo,
    pub plot: Retangulo,
    pub fonte: f32,
    pub titulo: Option<(f32, f32)>,
    pub subtitulo: Option<(f32, f32)>,
    pub unidade_y: f32,
    pub legenda: Option<(PosicaoLegenda, Retangulo, f32)>,
    /// Baseline do título do eixo horizontal e x do título do eixo vertical.
    pub titulo_eixo_h: Option<f32>,
    pub titulo_eixo_v: Option<f32>,
}

/// Largura aproximada de um texto (mesma métrica de `abreviar`).
pub(crate) fn largura_texto(t: &str, fonte: f32) -> f32 {
    t.chars().count() as f32 * fonte * 0.60
}

/// Largura de um item da legenda (marca, respiro e texto).
pub(crate) fn largura_item_legenda(t: &str, fonte: f32) -> f32 {
    largura_texto(t, fonte) + fonte * 2.3
}

/// Linhas em que os itens da legenda quebram, dada a largura disponível.
pub(crate) fn quebrar_legenda(nomes: &[String], fonte: f32, largura: f32) -> Vec<Vec<usize>> {
    let mut linhas: Vec<Vec<usize>> = Vec::new();
    let mut usado = f32::INFINITY;
    for (i, n) in nomes.iter().enumerate() {
        let w = largura_item_legenda(n, fonte).min(largura);
        if usado + w > largura {
            linhas.push(Vec::new());
            usado = 0.0;
        }
        linhas.last_mut().expect("linha criada").push(i);
        usado += w;
    }
    linhas
}

/// Texto de um tick do eixo de valores, com o formato do eixo ou o da tipografia.
pub(crate) fn texto_tick(cfg: &ConfigEixo, comp: &Composicao, v: f64) -> String {
    match &cfg.formato {
        Some(f) => f.formatar(v),
        None => comp.formato.formatar(v, comp.localidade).unwrap_or_default(),
    }
}

impl Grafico {
    /// Nomes da legenda: séries, ou categorias na pizza.
    pub(crate) fn nomes_legenda(&self, pizza_por_categoria: bool) -> Vec<String> {
        if self.tipo == Tipo::Pizza && pizza_por_categoria {
            self.categorias.clone()
        } else {
            self.series.iter().map(|s| s.nome.clone()).collect()
        }
    }

    pub(crate) fn moldura(
        &self,
        largura: f32,
        altura: f32,
        ap: &AparenciaGrafico,
        comp: &Composicao,
    ) -> Result<Moldura, ErroGrafico> {
        let cena = Retangulo::novo(0.0, 0.0, largura, altura);
        if !ap.usa_moldura() {
            return Ok(Moldura {
                legado: true,
                cartao: cena,
                conteudo: cena,
                plot: ap.layout.area(largura, altura)?,
                fonte: comp.fonte,
                titulo: None,
                subtitulo: None,
                unidade_y: 0.0,
                legenda: None,
                titulo_eixo_h: None,
                titulo_eixo_v: None,
            });
        }
        if !largura.is_finite() || !altura.is_finite() || largura <= 0.0 || altura <= 0.0 {
            return Err(ErroGrafico::EixoInvalido);
        }
        let f = comp.fonte;
        let cartao = ap.cartao.map_or(cena, |c| c.area(largura, altura));
        let respiro = ap.cartao.map_or(f * 0.8, |c| c.margem_interna.max(f * 0.4));
        let conteudo = Retangulo::novo(
            cartao.x + respiro,
            cartao.y + respiro,
            (cartao.largura - 2.0 * respiro).max(1.0),
            (cartao.altura - 2.0 * respiro).max(1.0),
        );
        let (mut topo, mut base) = (conteudo.y, conteudo.y + conteudo.altura);
        let (mut esq, mut dir) = (conteudo.x, conteudo.x + conteudo.largura);

        let mut titulo = None;
        if comp.titulo {
            let t = ap.titulo.and_then(|e| e.fonte.tamanho).unwrap_or(f * 1.4);
            titulo = Some((topo + t, t));
            topo += t * 1.45;
        }
        let mut subtitulo = None;
        if comp.subtitulo {
            let t = ap.subtitulo.and_then(|e| e.fonte.tamanho).unwrap_or(f);
            subtitulo = Some((topo + t, t));
            topo += t * 1.5;
        }
        if titulo.is_some() || subtitulo.is_some() {
            topo += f * 0.5;
        }

        let posicao = if comp.legenda.is_empty() {
            PosicaoLegenda::Nenhuma
        } else {
            ap.legenda.map_or(PosicaoLegenda::Baixo, |l| l.posicao)
        };
        let fl = ap.legenda.and_then(|l| l.fonte.tamanho).unwrap_or(f);
        let mut legenda = None;
        match posicao {
            PosicaoLegenda::Cima | PosicaoLegenda::Baixo => {
                let linhas = quebrar_legenda(&comp.legenda, fl, conteudo.largura).len() as f32;
                let h = linhas * fl * 1.4 + fl * 0.3;
                let y = if posicao == PosicaoLegenda::Cima {
                    topo += h + f * 0.4;
                    topo - h - f * 0.4
                } else {
                    base -= h;
                    base
                };
                legenda = Some((posicao, Retangulo::novo(conteudo.x, y, conteudo.largura, h), fl));
                if posicao == PosicaoLegenda::Baixo {
                    base -= f * 0.4;
                }
            }
            PosicaoLegenda::Esquerda | PosicaoLegenda::Direita => {
                let w = comp
                    .legenda
                    .iter()
                    .map(|n| largura_item_legenda(n, fl))
                    .fold(0.0, f32::max)
                    .min(conteudo.largura * 0.35);
                let x = if posicao == PosicaoLegenda::Esquerda {
                    esq += w + f;
                    conteudo.x
                } else {
                    dir -= w + f;
                    dir + f
                };
                legenda = Some((posicao, Retangulo::novo(x, topo, w, (base - topo).max(1.0)), fl));
            }
            PosicaoLegenda::Nenhuma => {}
        }

        let unidade_y = topo + f;
        if comp.unidade {
            topo += f * 1.4;
        }

        let (mut titulo_eixo_h, mut titulo_eixo_v) = (None, None);
        if self.tipo != Tipo::Pizza {
            // Eixo horizontal: valores nas barras, categorias nos demais.
            let barras = self.tipo == Tipo::Barras;
            let (cfg_h, cfg_v) = if barras {
                (&ap.eixos.valores, &ap.eixos.categorias)
            } else {
                (&ap.eixos.categorias, &ap.eixos.valores)
            };
            let textos_valores = || -> Vec<String> {
                self.escala_de_valores(&ap.eixos.valores)
                    .map(|e| {
                        e.ticks_principais(ap.eixos.valores.intervalo)
                            .into_iter()
                            .map(|v| texto_tick(&ap.eixos.valores, comp, v))
                            .collect()
                    })
                    .unwrap_or_default()
            };
            let textos_categorias = || -> Vec<String> {
                if self.tipo == Tipo::Dispersao {
                    let xs: Vec<f64> = self.categorias.iter().filter_map(|c| c.parse().ok()).collect();
                    let (a, b) = xs.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), v| (a.min(*v), b.max(*v)));
                    crate::EscalaLinear::nova(a, b)
                        .map(|e| {
                            e.ticks_principais(ap.eixos.categorias.intervalo)
                                .into_iter()
                                .map(|v| texto_tick(&ap.eixos.categorias, comp, v))
                                .collect()
                        })
                        .unwrap_or_default()
                } else {
                    self.categorias.clone()
                }
            };
            let (h_textos, v_textos) = if barras {
                (textos_valores(), textos_categorias())
            } else {
                (textos_categorias(), textos_valores())
            };
            if cfg_h.titulo.is_some() {
                let t = cfg_h.fonte_titulo.tamanho.unwrap_or(f);
                titulo_eixo_h = Some(base - t * 0.3);
                base -= t * 1.5;
            }
            if cfg_v.titulo.is_some() {
                let t = cfg_v.fonte_titulo.tamanho.unwrap_or(f);
                titulo_eixo_v = Some(esq + t);
                esq += t * 1.6;
            }
            if comp.eixos && cfg_h.mostrar {
                let maior = h_textos
                    .iter()
                    .map(|t| largura_texto(t, f))
                    .fold(0.0, f32::max)
                    .min(conteudo.altura * 0.3);
                base -= match cfg_h.rotacao.graus() as i32 {
                    45 => maior * std::f32::consts::FRAC_1_SQRT_2 + f * 1.4,
                    90 => maior + f * 0.9,
                    _ => f * 1.7,
                };
            } else {
                base -= f * 0.3;
            }
            if comp.eixos && cfg_v.mostrar {
                let maior = v_textos
                    .iter()
                    .map(|t| largura_texto(t, f))
                    .fold(0.0, f32::max)
                    .min(conteudo.largura * 0.3);
                esq += maior + f * 0.8;
            } else {
                esq += f * 0.3;
            }
            // Respiro para o último rótulo do eixo X não ser cortado.
            dir -= f * 1.2;
            topo += f * 0.6;
        }
        let plot = Retangulo::novo(esq, topo, dir - esq, base - topo);
        if !(plot.largura > 1.0 && plot.altura > 1.0) {
            return Err(ErroGrafico::EixoInvalido);
        }
        Ok(Moldura {
            legado: false,
            cartao,
            conteudo,
            plot,
            fonte: f,
            titulo,
            subtitulo,
            unidade_y,
            legenda,
            titulo_eixo_h,
            titulo_eixo_v,
        })
    }
}
