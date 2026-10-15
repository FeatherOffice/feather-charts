//! Dados e geometria de gráficos reutilizados por Sheet, Slide e Docs.
//! A Cena é independente de Iced/SVG/PDF; renderizadores desenham as mesmas
//! formas. Análise avançada/agrupamento continua no motor de dados do Sheet.

use estilos::{brilho_barra, desenhar_serie_linha, figura_barra, raios_cantos, SerieLinha};
use feather_draw::{Cena, Cor, Estilo, EstiloVetor, Figura, Ponto, Retangulo};

pub mod motor;
pub mod amostragem;
pub mod contrato;
pub mod aparencia;
pub mod eixos;
pub mod estilos;
pub mod filtro;
pub mod interacao;
pub mod pdf;
pub mod pizza;
pub mod svg;
pub mod temas;
pub mod tipografia;
pub use aparencia::{AparenciaGrafico, LayoutGrafico, Marcador};
pub use contrato::{AmostraDados, ContratoGrafico, MetadadosSerie};
pub use amostragem::{indices_lttb, ErroAmostragem};
pub use estilos::{Arredondamento, CantosArredondados, GradienteArea, Neon, PapelBarra, RaioBarra, Suavizacao};
pub use filtro::{FiltroLegenda, ItemLegenda};
pub use temas::{aplicar_tema, temas, Tema};
pub use eixos::{Eixos, EscalaLinear, Tick};
pub use interacao::{AlvoGrafico, ErroInteracao, EventoGrafico, HitGrafico, InteracaoGrafico, MudancaInteracao, ViewportGrafico};
pub use pdf::ErroPdfGrafico;
pub use pizza::{FatiaPizza, RotuloPizza};
pub use svg::ErroSvgGrafico;
pub use tipografia::{AlinhamentoRotulo, CenaAnotada, ErroExportacaoTipografica, FormatoNumero, LocalidadeGrafico, RotuloGrafico, TipografiaGrafico};

/// Cores das fatias quando nem a série nem a paleta definem.
pub(crate) const CORES_PIZZA: [Cor; 5] = [
    Cor::rgb(43, 122, 120),
    Cor::rgb(11, 87, 208),
    Cor::rgb(232, 113, 10),
    Cor::rgb(24, 128, 56),
    Cor::rgb(142, 36, 170),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tipo {
    Colunas,
    Barras,
    Linhas,
    /// Área preenchida entre cada série e a linha do zero.
    Area,
    /// Linhas com segmentos horizontais seguidos de saltos verticais.
    Degraus,
    Pizza,
    Dispersao,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Serie {
    pub nome: String,
    pub valores: Vec<f64>,
    pub cor: Cor,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Grafico {
    pub titulo: String,
    pub tipo: Tipo,
    pub categorias: Vec<String>,
    pub series: Vec<Serie>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErroGrafico {
    SemDados,
    TamanhosDiferentes,
    ValorInvalido,
    EixoInvalido,
}

impl Grafico {
    pub fn validar(&self) -> Result<(), ErroGrafico> {
        if self.categorias.is_empty() || self.series.is_empty() {
            return Err(ErroGrafico::SemDados);
        }
        if self
            .series
            .iter()
            .any(|s| s.valores.len() != self.categorias.len())
        {
            return Err(ErroGrafico::TamanhosDiferentes);
        }
        if self
            .series
            .iter()
            .flat_map(|s| s.valores.iter())
            .any(|n| !n.is_finite())
        {
            return Err(ErroGrafico::ValorInvalido);
        }
        if self.tipo == Tipo::Pizza {
            if self.series.len() != 1 || self.series[0].valores.iter().any(|v| *v < 0.0) {
                return Err(ErroGrafico::ValorInvalido);
            }
            if !self.series[0].valores.iter().any(|v| *v > 0.0) {
                return Err(ErroGrafico::SemDados);
            }
        }
        if self.tipo == Tipo::Dispersao
            && self
                .categorias
                .iter()
                .any(|c| c.parse::<f64>().map_or(true, |x| !x.is_finite()))
        {
            return Err(ErroGrafico::EixoInvalido);
        }
        Ok(())
    }

    /// Mantém a renderização original e todas as amostras para compatibilidade.
    pub fn cena(&self, largura: f32, altura: f32) -> Result<Cena, ErroGrafico> {
        self.cena_impl(largura, altura, None, &AparenciaGrafico::default(), None)
    }

    /// Gera geometria com paleta, estilos, marcadores e margens configuráveis.
    /// O contrato `cena` legado permanece com aparência padrão.
    pub fn cena_estilizada(
        &self,
        largura: f32,
        altura: f32,
        aparencia: &AparenciaGrafico,
    ) -> Result<Cena, ErroGrafico> {
        self.cena_impl(largura, altura, None, aparencia, None)
    }

    /// Reduz apenas os pontos visuais de linhas por LTTB.
    /// As séries originais nunca são mutadas, reordenadas ou agregadas.
    /// Outros tipos mantêm sua representação completa.
    pub fn cena_amostrada(
        &self,
        largura: f32,
        altura: f32,
        max_pontos: usize,
    ) -> Result<Cena, ErroGrafico> {
        if max_pontos < 3 {
            return Err(ErroGrafico::ValorInvalido);
        }
        self.cena_impl(largura, altura, Some(max_pontos), &AparenciaGrafico::default(), None)
    }

    /// Amostragem visual sem alterar os dados, mantendo opções de aparência.
    pub fn cena_amostrada_estilizada(
        &self,
        largura: f32,
        altura: f32,
        max_pontos: usize,
        aparencia: &AparenciaGrafico,
    ) -> Result<Cena, ErroGrafico> {
        if max_pontos < 3 {
            return Err(ErroGrafico::ValorInvalido);
        }
        self.cena_impl(largura, altura, Some(max_pontos), aparencia, None)
    }

    /// Retângulos de colunas/barras na ordem série-maior, categoria-menor.
    /// Fonte única da geometria para desenho, rótulos e hit-test.
    pub(crate) fn retangulos_barras(
        &self,
        largura: f32,
        altura: f32,
        layout: &LayoutGrafico,
    ) -> Result<Vec<(usize, usize, Retangulo)>, ErroGrafico> {
        let eixos = self
            .eixos_com_layout(largura, altura, layout)?
            .ok_or(ErroGrafico::EixoInvalido)?;
        let area_plot = eixos.area;
        let (x0, y0, w, h) = (area_plot.x, area_plot.y, area_plot.largura, area_plot.altura);
        let n = self.categorias.len();
        let n_series = self.series.len();
        let escala_valores = if self.tipo == Tipo::Barras {
            eixos.escala_x.ok_or(ErroGrafico::EixoInvalido)?
        } else {
            eixos.escala_y.ok_or(ErroGrafico::EixoInvalido)?
        };
        let mapear_y = |v: f64| y0 + h * (1.0 - escala_valores.fracao(v));
        let baseline = mapear_y(0.0);
        let largura_grupo = w / n as f32;
        // Mesmo com uma única categoria, a coluna não deve ocupar
        // quase toda a área de plotagem. Limitar a faixa em pixels do
        // SVG também preserva o visual ao reduzir o embed no Iced.
        let faixa_grupo = (largura_grupo * 0.72).min(112.0);
        let faixa = faixa_grupo / n_series as f32;
        let mut saida = Vec::with_capacity(n * n_series);
        for (s, serie) in self.series.iter().enumerate() {
            for (i, valor) in serie.valores.iter().enumerate() {
                let y = mapear_y(*valor);
                let x = x0 + largura_grupo * i as f32 + (largura_grupo - faixa_grupo) / 2.0
                    + faixa * s as f32;
                let area = if self.tipo == Tipo::Colunas {
                    Retangulo::novo(x, y.min(baseline), faixa * 0.94, (y - baseline).abs())
                } else {
                    // Barra horizontal: categorias no eixo Y.
                    let f = escala_valores.fracao(*valor);
                    let zero = escala_valores.fracao(0.0);
                    let yb = y0
                        + (h / n as f32) * (i as f32 + 0.09)
                        + (h / n as f32) * 0.82 * s as f32 / n_series as f32;
                    let px = x0 + w * f.min(zero);
                    Retangulo::novo(
                        px,
                        yb,
                        w * (f - zero).abs(),
                        (h / n as f32) * 0.82 / n_series as f32 * 0.94,
                    )
                };
                saida.push((s, i, area));
            }
        }
        Ok(saida)
    }

    /// `mapa`: índice original de cada série (ou fatia, na pizza) quando o
    /// gráfico foi filtrado; mantém as cores da paleta estáveis.
    fn cena_impl(
        &self,
        largura: f32,
        altura: f32,
        max_pontos: Option<usize>,
        aparencia: &AparenciaGrafico,
        mapa: Option<&[usize]>,
    ) -> Result<Cena, ErroGrafico> {
        self.validar()?;
        aparencia.validar()?;
        let area_plot = aparencia.layout.area(largura, altura)?;
        let mut cena = Cena::nova(largura, altura);
        let original = |i: usize| mapa.map_or(i, |m| m[i]);
        if let Some(fundo) = aparencia.fundo {
            // Polilinha fechada (e não Retangulo): `Retangulo` na cena é só dado.
            cena.adicionar(Figura::Polilinha {
                pontos: vec![
                    Ponto::novo(0.0, 0.0),
                    Ponto::novo(largura, 0.0),
                    Ponto::novo(largura, altura),
                    Ponto::novo(0.0, altura),
                ],
                fechada: true,
                estilo: Estilo { preenchimento: Some(fundo), contorno: None, espessura: 0.0 },
            });
        }
        let x0 = area_plot.x;
        let y0 = area_plot.y;
        let w = area_plot.largura;
        let h = area_plot.altura;
        let n = self.categorias.len();

        if self.tipo == Tipo::Pizza {
            let centro = Ponto::novo(largura * 0.5, altura * 0.5);
            let raio = largura.min(altura) * 0.38;
            let cores = CORES_PIZZA;
            for fatia in self.fatias_pizza(largura, altura)? {
                let inicio = fatia.inicio as f32;
                let fim = fatia.fim as f32;
                let arco = fim - inicio;
                if arco <= 0.0 {
                    continue;
                }
                let passos = ((arco * 24.0).ceil() as usize).clamp(2, 4096);
                let mut pontos = vec![centro];
                for j in 0..=passos {
                    let a = inicio + arco * j as f32 / passos as f32;
                    pontos.push(Ponto::novo(
                        centro.x + raio * a.cos(),
                        centro.y + raio * a.sin(),
                    ));
                }
                let indice = original(fatia.indice);
                cena.adicionar(Figura::Polilinha {
                    pontos,
                    fechada: true,
                    estilo: Estilo {
                        preenchimento: Some(aparencia.cor_serie(indice, cores[indice % cores.len()])),
                        contorno: aparencia.contorno(),
                        espessura: aparencia.espessura_contorno,
                    },
                });
                // A geometria do texto fica em FatiaPizza::rotulo, disponível
                // aos backends que desenham tipografia (CHT-007).
            }
            return Ok(cena);
        }

        let eixos = self
            .eixos_com_layout(largura, altura, &aparencia.layout)?
            .expect("somente pizza não tem eixos");
        let escala_valores = if self.tipo == Tipo::Barras {
            eixos.escala_x.expect("barras têm escala X")
        } else {
            eixos.escala_y.expect("gráficos cartesianos têm escala Y")
        };
        let mapear_y = |v: f64| y0 + h * (1.0 - escala_valores.fracao(v));
        let baseline = mapear_y(0.0);
        let eixo = Estilo {
            preenchimento: None,
            contorno: Some(aparencia.cor_eixos.unwrap_or(Cor::rgb(95, 99, 104))),
            espessura: 1.0,
        };
        if let Some(cor_grade) = aparencia.cor_grade {
            let grade = Estilo { preenchimento: None, contorno: Some(cor_grade), espessura: 1.0 };
            // Grade do eixo de valores: vertical nas barras, horizontal nos demais.
            let (ticks, vertical) = if self.tipo == Tipo::Barras {
                (&eixos.ticks_x, true)
            } else {
                (&eixos.ticks_y, false)
            };
            for tick in ticks {
                let pontos = if vertical {
                    vec![Ponto::novo(tick.posicao, y0), Ponto::novo(tick.posicao, y0 + h)]
                } else {
                    vec![Ponto::novo(x0, tick.posicao), Ponto::novo(x0 + w, tick.posicao)]
                };
                cena.adicionar(Figura::Polilinha { pontos, fechada: false, estilo: grade.clone() });
            }
        }
        // Ticks como segmentos vetoriais; texto e formatação pertencem ao backend.
        for tick in &eixos.ticks_x {
            cena.adicionar(Figura::Polilinha {
                pontos: vec![
                    Ponto::novo(tick.posicao, y0 + h),
                    Ponto::novo(tick.posicao, y0 + h - 4.0_f32.min(h)),
                ],
                fechada: false,
                estilo: eixo.clone(),
            });
        }
        for tick in &eixos.ticks_y {
            cena.adicionar(Figura::Polilinha {
                pontos: vec![
                    Ponto::novo(x0, tick.posicao),
                    Ponto::novo(x0 + 4.0_f32.min(w), tick.posicao),
                ],
                fechada: false,
                estilo: eixo.clone(),
            });
        }
        cena.adicionar(Figura::Polilinha {
            pontos: vec![
                Ponto::novo(x0, y0),
                Ponto::novo(x0, y0 + h),
                Ponto::novo(x0 + w, y0 + h),
            ],
            fechada: false,
            estilo: eixo.clone(),
        });
        // O zero atravessa a área do gráfico. Para barras é vertical.
        cena.adicionar(Figura::Polilinha {
            pontos: if eixos.zero_vertical {
                vec![
                    Ponto::novo(eixos.zero, y0),
                    Ponto::novo(eixos.zero, y0 + h),
                ]
            } else {
                vec![
                    Ponto::novo(x0, eixos.zero),
                    Ponto::novo(x0 + w, eixos.zero),
                ]
            },
            fechada: false,
            estilo: eixo,
        });

        match self.tipo {
            Tipo::Colunas | Tipo::Barras => {
                let vertical = self.tipo == Tipo::Colunas;
                for (s, i, area) in self.retangulos_barras(largura, altura, &aparencia.layout)? {
                    let serie = &self.series[s];
                    let cor = aparencia.cor_serie(original(s), serie.cor);
                    let estilo = Estilo {
                        preenchimento: Some(cor),
                        contorno: aparencia.contorno(),
                        espessura: aparencia.espessura_contorno,
                    };
                    let raios = aparencia.arredondamento.map_or([0.0; 4], |ar| {
                        raios_cantos(area, vertical, serie.valores[i] >= 0.0, PapelBarra::ISOLADA, &ar)
                    });
                    if let Some(neon) = &aparencia.neon {
                        brilho_barra(&mut cena, area, raios, aparencia.espessura_contorno.max(1.0), cor, neon);
                    }
                    cena.adicionar(figura_barra(area, raios, estilo));
                }
            }
            Tipo::Linhas | Tipo::Area | Tipo::Degraus | Tipo::Dispersao => {
                let xs = if self.tipo == Tipo::Dispersao {
                    self.categorias
                        .iter()
                        .map(|c| c.parse::<f64>().unwrap_or(0.0))
                        .collect::<Vec<_>>()
                } else {
                    (0..n).map(|v| v as f64).collect::<Vec<_>>()
                };
                let xmin = xs.iter().copied().fold(f64::INFINITY, f64::min);
                let xmax = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                let escala_x = EscalaLinear::nova(xmin, xmax)?;
                for (indice_serie, serie) in self.series.iter().enumerate() {
                    let amostras = if self.tipo == Tipo::Linhas {
                        max_pontos.and_then(|limite| {
                            (n > limite).then(|| amostragem::indices_lttb(&serie.valores, limite))
                        })
                    } else {
                        None
                    };
                    let pontos = if let Some(amostras) = amostras {
                        amostras
                            .map_err(|_| ErroGrafico::ValorInvalido)?
                            .into_iter()
                            .map(|i| {
                                let x = if n == 1 {
                                    x0 + w / 2.0
                                } else {
                                    x0 + w * escala_x.fracao(xs[i])
                                };
                                Ponto::novo(x, mapear_y(serie.valores[i]))
                            })
                            .collect::<Vec<_>>()
                    } else {
                        serie
                            .valores
                            .iter()
                            .enumerate()
                            .map(|(i, v)| {
                                let x = if n == 1 {
                                    x0 + w / 2.0
                                } else {
                                    x0 + w * escala_x.fracao(xs[i])
                                };
                                Ponto::novo(x, mapear_y(*v))
                            })
                            .collect::<Vec<_>>()
                    };
                    let cor = aparencia.cor_serie(original(indice_serie), serie.cor);
                    if self.tipo != Tipo::Dispersao {
                        // Mantém os dados originais intactos. Apenas a geometria
                        // de Área e Degraus muda; a extração/agregação é da origem.
                        let com_marcadores = aparencia.marcador_para(self.tipo) != Marcador::Nenhum;
                        if com_marcadores {
                            adicionar_marcadores(&mut cena, &pontos, cor, self.tipo, aparencia);
                        }
                        let eh_area = self.tipo == Tipo::Area;
                        let estilizada = aparencia.estiliza_linha(
                            original(indice_serie), eh_area, self.tipo != Tipo::Degraus);
                        let pontos = if self.tipo == Tipo::Degraus {
                            let mut degraus =
                                Vec::with_capacity(pontos.len().saturating_mul(2).saturating_sub(1));
                            if let Some(primeiro) = pontos.first() {
                                degraus.push(*primeiro);
                            }
                            for par in pontos.windows(2) {
                                degraus.push(Ponto::novo(par[1].x, par[0].y));
                                degraus.push(par[1]);
                            }
                            degraus
                        } else {
                            pontos
                        };
                        if estilizada {
                            desenhar_serie_linha(&mut cena, &pontos, &SerieLinha {
                                cor,
                                indice_serie: original(indice_serie),
                                baseline,
                                area: eh_area,
                                permite_suavizar: self.tipo != Tipo::Degraus,
                                aparencia,
                            });
                            continue;
                        }
                        let (pontos, fechada, preenchimento) = if eh_area {
                            // Fecha o polígono no zero REAL (inclusive com
                            // dados negativos), sem transformar valores.
                            let mut contorno = Vec::with_capacity(pontos.len() + 2);
                            if let (Some(primeiro), Some(ultimo)) =
                                (pontos.first().copied(), pontos.last().copied())
                            {
                                contorno.push(Ponto::novo(primeiro.x, baseline));
                                contorno.extend(pontos);
                                contorno.push(Ponto::novo(ultimo.x, baseline));
                            }
                            let alfa = ((f32::from(cor.a) * 0.35).round() as u8).max(1);
                            (contorno, true, Some(Cor::rgba(cor.r, cor.g, cor.b, alfa)))
                        } else {
                            (pontos, false, None)
                        };
                        cena.adicionar(Figura::Polilinha {
                            pontos,
                            fechada,
                            estilo: Estilo {
                                preenchimento,
                                contorno: Some(cor),
                                espessura: aparencia.espessura_linha,
                            },
                        });
                    } else {
                        adicionar_marcadores(&mut cena, &pontos, cor, self.tipo, aparencia);
                    }
                }
            }
            Tipo::Pizza => unreachable!(),
        }
        Ok(cena)
    }
}

fn adicionar_marcadores(
    cena: &mut Cena,
    pontos: &[Ponto],
    cor: Cor,
    tipo: Tipo,
    aparencia: &AparenciaGrafico,
) {
    let marcador = aparencia.marcador_para(tipo);
    if marcador == Marcador::Nenhum {
        return;
    }
    let raio = aparencia.tamanho_marcador / 2.0;
    let estilo = Estilo {
        preenchimento: Some(cor),
        contorno: aparencia.contorno(),
        espessura: aparencia.espessura_contorno,
    };
    for ponto in pontos {
        match marcador {
            Marcador::Quadrado => cena.adicionar(Figura::Retangulo {
                area: Retangulo::novo(
                    ponto.x - raio,
                    ponto.y - raio,
                    aparencia.tamanho_marcador,
                    aparencia.tamanho_marcador,
                ),
                estilo: estilo.clone(),
            }),
            Marcador::Circulo => cena.adicionar(Figura::Elipse {
                centro: *ponto,
                raio_x: raio,
                raio_y: raio,
                estilo: EstiloVetor::from(&estilo),
            }),
            Marcador::Nenhum | Marcador::Automatico => {}
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    fn exemplo(tipo: Tipo) -> Grafico {
        Grafico {
            titulo: "Receita".into(),
            tipo,
            categorias: vec!["1".into(), "2".into()],
            series: vec![Serie {
                nome: "Vendas".into(),
                valores: vec![10.0, 20.0],
                cor: Cor::rgb(11, 87, 208),
            }],
        }
    }
    #[test]
    fn personalizacao_paleta_contorno_marcadores_e_eixos_negativos() {
        let mut g = exemplo(Tipo::Barras);
        g.series[0].valores = vec![-10.0, 20.0];
        let cor = Cor::rgba(200, 50, 30, 128);
        let ap = AparenciaGrafico {
            paleta: vec![cor],
            opacidade: 0.5,
            cor_contorno: Some(Cor::rgb(1, 2, 3)),
            espessura_contorno: 1.0,
            ..Default::default()
        };
        let cena = g.cena_estilizada(600.0, 400.0, &ap).unwrap();
        let eixos = g.eixos_com_layout(600.0, 400.0, &ap.layout).unwrap().unwrap();
        let barras = cena.figuras.iter().filter_map(|figura| {
            if let Figura::Retangulo { area, estilo } = figura {
                assert_eq!(estilo.preenchimento, Some(Cor::rgba(200, 50, 30, 64)));
                assert_eq!(estilo.espessura, 1.0);
                Some(*area)
            } else {
                None
            }
        }).collect::<Vec<_>>();
        assert_eq!(barras.len(), 2);
        assert!((barras[0].x + barras[0].largura - eixos.zero).abs() < 0.01);
        assert!((barras[1].x - eixos.zero).abs() < 0.01);
        g.tipo = Tipo::Dispersao;
        let circulos = g.cena_estilizada(400.0, 300.0, &AparenciaGrafico {
            marcador: Marcador::Circulo,
            ..Default::default()
        }).unwrap();
        assert_eq!(circulos.figuras.iter().filter(|f| matches!(f, Figura::Elipse { .. })).count(), 2);
    }

    #[test]
    fn coluna_isolada_fica_centralizada_sem_encostar_nas_bordas() {
        let mut g = exemplo(Tipo::Colunas);
        g.categorias = vec!["b".into()];
        g.series[0].valores = vec![2.0];
        let cena = g.cena(800.0, 450.0).unwrap();
        let barras: Vec<_> = cena.figuras.iter().filter_map(|f| match f {
            Figura::Retangulo { area, estilo } if estilo.preenchimento.is_some() =>
                Some(*area),
            _ => None,
        }).collect();
        assert_eq!(barras.len(), 1);
        let eixo = g.eixos(800., 450.).unwrap().unwrap();
        let barra = barras[0];
        assert!(barra.largura <= 112.0, "a única barra não deve encher o plot");
        assert!(barra.x > eixo.area.x + 20.0, "deve respeitar margem esquerda");
        assert!(barra.x + barra.largura < eixo.area.x + eixo.area.largura - 20.0);
    }

    #[test]
    fn mesmos_dados_em_tres_tipos_de_cena() {
        for tipo in [
            Tipo::Colunas,
            Tipo::Linhas,
            Tipo::Pizza,
            Tipo::Barras,
            Tipo::Dispersao,
        ] {
            assert!(exemplo(tipo).cena(640.0, 480.0).unwrap().figuras.len() >= 2);
        }
    }
    #[test]
    fn cenas_de_graficos_exportam_svg_vetorial_reprodutivel() {
        for tipo in [
            Tipo::Colunas,
            Tipo::Linhas,
            Tipo::Pizza,
            Tipo::Barras,
            Tipo::Dispersao,
        ] {
            let cena = exemplo(tipo).cena(640.0, 480.0).unwrap();
            let svg = feather_draw::exportar_svg(&cena).unwrap();
            assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""));
            assert!(svg.ends_with("</svg>\n"));
            assert_eq!(svg, feather_draw::exportar_svg(&cena).unwrap());
            assert_eq!(svg.matches("<path d=").count(), cena.figuras.len());
        }
    }

    #[test]
    fn rejeita_series_irregulares_e_nan() {
        let mut g = exemplo(Tipo::Colunas);
        g.series[0].valores.pop();
        assert_eq!(g.validar(), Err(ErroGrafico::TamanhosDiferentes));
        g.series[0].valores = vec![1.0, f64::NAN];
        assert_eq!(g.validar(), Err(ErroGrafico::ValorInvalido));
    }
    #[test]
    fn lttb_e_aplicado_ao_desenho_sem_modificar_series() {
        let mut g = exemplo(Tipo::Linhas);
        let n = 5_000;
        g.categorias = (0..n).map(|i| format!("T{i}")).collect();
        g.series[0].valores = (0..n)
            .map(|i| if i == 2_456 { 10_000.0 } else { 0.0 })
            .collect();
        let original = g.clone();
        let completa = g.cena(1024., 768.).unwrap();
        let reduzida = g.cena_amostrada(1024., 768., 48).unwrap();
        let pontos_da_serie = |cena: &Cena| match cena.figuras.last().unwrap() {
            Figura::Polilinha { pontos, .. } => pontos.clone(),
            _ => panic!("linha deve permanecer vetorial"),
        };
        let todos = pontos_da_serie(&completa);
        let menos = pontos_da_serie(&reduzida);
        assert_eq!(todos.len(), n);
        assert_eq!(menos.len(), 48);
        assert_eq!(todos.first(), menos.first());
        assert_eq!(todos.last(), menos.last());
        assert!(menos
            .iter()
            .any(|p| p.y == todos[2_456].y && p.x == todos[2_456].x));
        assert_eq!(g, original);
        assert_eq!(reduzida, g.cena_amostrada(1024., 768., 48).unwrap());
    }

    #[test]
    fn area_fecha_na_linha_zero_com_valores_positivos_e_negativos() {
        let mut g = exemplo(Tipo::Area);
        g.series[0].valores = vec![-8.0, 12.0];
        let eixos = g.eixos(600.0, 400.0).unwrap().unwrap();
        let cena = g.cena(600.0, 400.0).unwrap();
        match cena.figuras.last().unwrap() {
            Figura::Polilinha { pontos, fechada, estilo } => {
                assert!(*fechada);
                assert_eq!(pontos.len(), 4);
                assert!((pontos[0].y - eixos.zero).abs() < 0.001);
                assert!((pontos[3].y - eixos.zero).abs() < 0.001);
                assert!(pontos[1].y > eixos.zero);
                assert!(pontos[2].y < eixos.zero);
                assert!(estilo.preenchimento.is_some());
            }
            _ => panic!("area deve ser poligono vetorial"),
        }
        assert_eq!(g.series[0].valores, [-8.0, 12.0]);
        assert!(g.exportar_svg(600.0, 400.0).unwrap().contains("<path"));
    }

    #[test]
    fn degraus_horizontal_depois_vertical_sem_mudar_amostras() {
        let mut g = exemplo(Tipo::Degraus);
        g.categorias = vec!["A".into(), "B".into(), "C".into()];
        g.series[0].valores = vec![10.0, 30.0, -5.0];
        let cena = g.cena(640.0, 360.0).unwrap();
        match cena.figuras.last().unwrap() {
            Figura::Polilinha { pontos, fechada, estilo } => {
                assert!(!*fechada);
                assert_eq!(pontos.len(), 5);
                assert_eq!(pontos[0].y, pontos[1].y);
                assert_eq!(pontos[1].x, pontos[2].x);
                assert_eq!(pontos[2].y, pontos[3].y);
                assert_eq!(pontos[3].x, pontos[4].x);
                assert!(estilo.preenchimento.is_none());
            }
            _ => panic!("degraus devem ser segmentos vetoriais"),
        }
        assert_eq!(g.series[0].valores, [10.0, 30.0, -5.0]);
        assert_eq!(g.cena(640.0, 360.0).unwrap(), cena);
    }

    #[test]
    fn lttb_nao_muda_pizza_ou_agregacao_e_rejeita_limites_invalidos() {
        let pizza = exemplo(Tipo::Pizza);
        assert_eq!(
            pizza.cena(640., 480.).unwrap(),
            pizza.cena_amostrada(640., 480., 3).unwrap()
        );
        let barras = exemplo(Tipo::Barras);
        assert_eq!(
            barras.cena(640., 480.).unwrap(),
            barras.cena_amostrada(640., 480., 3).unwrap()
        );
        assert_eq!(
            exemplo(Tipo::Linhas).cena_amostrada(640., 480., 2),
            Err(ErroGrafico::ValorInvalido)
        );
    }
}
