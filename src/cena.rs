//! Geração da cena de `Grafico` (colunas, barras, linhas, área, degraus,
//! pizza e dispersão) com toda a personalização. Com
//! `AparenciaGrafico::default()` o resultado é idêntico ao desenho original.
use crate::desenho::{self, GeometriaPizza, Marca};
use crate::eixos::Eixos;
use crate::estilos::{brilho_barra, desenhar_serie_linha, figura_barra, raios_cantos, PapelBarra, SerieLinha};
use crate::moldura::{Composicao, Moldura};
use crate::personalizacao::{LinhaGrade, TracoLinha};
use crate::{amostragem, AparenciaGrafico, EscalaLinear, ErroGrafico, Grafico, Marcador, Tipo};
use feather_draw::{Cena, Cor, Estilo, Figura, Ponto, Retangulo};

/// Cena mais o layout usado para gerá-la (os textos se alinham por ele).
pub(crate) struct Desenho {
    pub cena: Cena,
    pub moldura: Moldura,
    pub eixos: Option<Eixos>,
    pub pizza: Option<GeometriaPizza>,
}

fn extremos(v: &[f64]) -> (f64, f64) {
    v.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), x| (a.min(*x), b.max(*x)))
}

/// Marcador resolvido de um ponto: estilo da série ou o global.
fn marca(ap: &AparenciaGrafico, tipo: Tipo, serie: usize, cor: Cor, propria: Option<Cor>) -> Option<Marca> {
    let m = match ap.estilo_serie(serie).and_then(|e| e.marcador) {
        Some(em) => {
            let forma = if em.forma == Marcador::Automatico { Marcador::Circulo } else { em.forma };
            let op = ap.opacidade_da_serie(serie);
            let alfa = |mut c: Cor| {
                c.a = (f32::from(c.a) * op).round() as u8;
                c
            };
            let (contorno_serie, esp_serie) = ap.contorno_da_serie(serie);
            Marca {
                forma,
                tamanho: em.tamanho.unwrap_or(ap.tamanho_marcador),
                preenchimento: propria.unwrap_or_else(|| em.preenchimento.map_or(cor, alfa)),
                contorno: em.contorno.map(alfa).or(contorno_serie),
                espessura: em.espessura_contorno.unwrap_or(if em.contorno.is_some() { 1.5 } else { esp_serie }),
            }
        }
        None => Marca {
            forma: ap.marcador_para(tipo),
            tamanho: ap.tamanho_marcador,
            preenchimento: propria.unwrap_or(cor),
            contorno: ap.contorno(),
            espessura: ap.espessura_contorno,
        },
    };
    (m.forma != Marcador::Nenhum).then_some(m)
}

impl Grafico {
    /// Retângulos de colunas/barras na ordem série-maior, categoria-menor.
    /// Fonte única da geometria para desenho, rótulos e hit-test.
    pub(crate) fn retangulos_barras(
        &self,
        largura: f32,
        altura: f32,
        ap: &AparenciaGrafico,
    ) -> Result<Vec<(usize, usize, Retangulo)>, ErroGrafico> {
        let eixos = self
            .eixos_com_aparencia(largura, altura, ap)?
            .ok_or(ErroGrafico::EixoInvalido)?;
        self.retangulos_barras_em(&eixos, ap)
    }

    pub(crate) fn retangulos_barras_em(
        &self,
        eixos: &Eixos,
        ap: &AparenciaGrafico,
    ) -> Result<Vec<(usize, usize, Retangulo)>, ErroGrafico> {
        let area_plot = eixos.area;
        let (x0, y0, w, h) = (area_plot.x, area_plot.y, area_plot.largura, area_plot.altura);
        let n = self.categorias.len();
        let n_series = self.series.len();
        let escala_valores = if self.tipo == Tipo::Barras {
            eixos.escala_x.ok_or(ErroGrafico::EixoInvalido)?
        } else {
            eixos.escala_y.ok_or(ErroGrafico::EixoInvalido)?
        };
        let invertido = ap.eixos.categorias.invertido;
        let posicao = |i: usize| if invertido { n - 1 - i } else { i };
        let mapear_y = |v: f64| y0 + h * (1.0 - escala_valores.fracao(v));
        let baseline = mapear_y(0.0);
        let mut saida = Vec::with_capacity(n * n_series);
        if let Some(b) = ap.barras.filter(|b| b.mexe_na_geometria()) {
            // Como o Excel: largura da barra b, espaço entre grupos g·b e
            // sobreposição o·b entre séries vizinhas.
            let g = f64::from(b.espaco_grupos.unwrap_or(150.0)) / 100.0;
            let o = f64::from(b.sobreposicao.unwrap_or(0.0)) / 100.0;
            let ns = n_series as f64;
            let categoria = if self.tipo == Tipo::Colunas { w } else { h } / n as f32;
            let barra = (f64::from(categoria) / (ns - (ns - 1.0) * o + g)) as f32;
            for (s, serie) in self.series.iter().enumerate() {
                for (i, valor) in serie.valores.iter().enumerate() {
                    let inicio = categoria * posicao(i) as f32
                        + barra * (g as f32 / 2.0 + s as f32 * (1.0 - o as f32));
                    let area = if self.tipo == Tipo::Colunas {
                        let y = mapear_y(*valor);
                        Retangulo::novo(x0 + inicio, y.min(baseline), barra, (y - baseline).abs())
                    } else {
                        let f = escala_valores.fracao(*valor);
                        let zero = escala_valores.fracao(0.0);
                        Retangulo::novo(x0 + w * f.min(zero), y0 + inicio, w * (f - zero).abs(), barra)
                    };
                    saida.push((s, i, area));
                }
            }
            return Ok(saida);
        }
        let largura_grupo = w / n as f32;
        // Mesmo com uma única categoria, a coluna não deve ocupar
        // quase toda a área de plotagem. Limitar a faixa em pixels do
        // SVG também preserva o visual ao reduzir o embed no Iced.
        let faixa_grupo = (largura_grupo * 0.72).min(112.0);
        let faixa = faixa_grupo / n_series as f32;
        for (s, serie) in self.series.iter().enumerate() {
            for (i, valor) in serie.valores.iter().enumerate() {
                let k = posicao(i);
                let y = mapear_y(*valor);
                let x = x0 + largura_grupo * k as f32 + (largura_grupo - faixa_grupo) / 2.0
                    + faixa * s as f32;
                let area = if self.tipo == Tipo::Colunas {
                    Retangulo::novo(x, y.min(baseline), faixa * 0.94, (y - baseline).abs())
                } else {
                    // Barra horizontal: categorias no eixo Y.
                    let f = escala_valores.fracao(*valor);
                    let zero = escala_valores.fracao(0.0);
                    let yb = y0
                        + (h / n as f32) * (k as f32 + 0.09)
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

    /// Posição horizontal dos pontos de linhas/área/degraus/dispersão.
    pub(crate) fn xs_pontos(&self, area: Retangulo, ap: &AparenciaGrafico) -> Result<Vec<f32>, ErroGrafico> {
        let n = self.categorias.len();
        let xs = if self.tipo == Tipo::Dispersao {
            self.categorias.iter().map(|c| c.parse::<f64>().unwrap_or(0.0)).collect::<Vec<_>>()
        } else {
            (0..n).map(|v| v as f64).collect::<Vec<_>>()
        };
        let (xmin, xmax) = extremos(&xs);
        let mut escala_x = EscalaLinear::nova(xmin, xmax)?;
        escala_x.invertida = ap.eixos.categorias.invertido;
        Ok(xs
            .iter()
            .map(|x| if n == 1 { area.x + area.largura / 2.0 } else { area.x + area.largura * escala_x.fracao(*x) })
            .collect())
    }

    /// `mapa`: índice original de cada série (ou fatia, na pizza) quando o
    /// gráfico foi filtrado; mantém as cores e estilos estáveis.
    pub(crate) fn desenhar(
        &self,
        largura: f32,
        altura: f32,
        max_pontos: Option<usize>,
        ap: &AparenciaGrafico,
        mapa: Option<&[usize]>,
        comp: &Composicao,
    ) -> Result<Desenho, ErroGrafico> {
        self.validar()?;
        ap.validar()?;
        let moldura = self.moldura(largura, altura, ap, comp)?;
        let mut cena = Cena::nova(largura, altura);
        let original = |i: usize| mapa.map_or(i, |m| m[i]);
        desenho::fundos(&mut cena, ap, &moldura, largura, altura);

        if self.tipo == Tipo::Pizza {
            let geo = GeometriaPizza::nova(ap, &moldura, largura, altura, self.categorias.len());
            let (vmin, vmax) = extremos(&self.series[0].valores);
            for fatia in self.fatias_pizza(largura, altura)? {
                let inicio = (fatia.inicio + geo.giro) as f32;
                let fim = (fatia.fim + geo.giro) as f32;
                if fim - inicio <= 0.0 {
                    continue;
                }
                let indice = original(fatia.indice);
                let pontos = geo.poligono(indice, inicio, fim);
                let cor = ap.cor_fatia(indice, fatia.valor, vmin, vmax);
                let (contorno, espessura) = match geo.estilo.borda {
                    Some(b) => (Some(b.cor), b.espessura),
                    None => (ap.contorno(), ap.espessura_contorno),
                };
                if let Some(neon) = &ap.neon {
                    for (w, c) in neon.camadas_de(espessura.max(1.0), cor) {
                        cena.adicionar(Figura::Polilinha {
                            pontos: pontos.clone(),
                            fechada: true,
                            estilo: Estilo { preenchimento: None, contorno: Some(c), espessura: w },
                        });
                    }
                }
                cena.adicionar(Figura::Polilinha {
                    pontos,
                    fechada: true,
                    estilo: Estilo { preenchimento: Some(cor), contorno, espessura },
                });
            }
            return Ok(Desenho { cena, moldura, eixos: None, pizza: Some(geo) });
        }

        let eixos = self.eixos_na_area(moldura.plot, ap)?.expect("somente pizza não tem eixos");
        let (x0, y0, w, h) = (eixos.area.x, eixos.area.y, eixos.area.largura, eixos.area.altura);
        let barras = self.tipo == Tipo::Barras;
        let escala_valores = if barras {
            eixos.escala_x.expect("barras têm escala X")
        } else {
            eixos.escala_y.expect("gráficos cartesianos têm escala Y")
        };
        let mapear_y = |v: f64| y0 + h * (1.0 - escala_valores.fracao(v));
        let baseline = mapear_y(0.0);
        let eixo = Estilo {
            preenchimento: None,
            contorno: Some(ap.cor_eixos.unwrap_or(Cor::rgb(95, 99, 104))),
            espessura: 1.0,
        };
        let (cv, cc) = (&ap.eixos.valores, &ap.eixos.categorias);
        // Grades: secundárias por baixo, depois as principais.
        let (ticks_v, sec_v, ticks_c, sec_c) = if barras {
            (&eixos.ticks_x, &eixos.ticks_secundarios_x, &eixos.ticks_y, &eixos.ticks_secundarios_y)
        } else {
            (&eixos.ticks_y, &eixos.ticks_secundarios_y, &eixos.ticks_x, &eixos.ticks_secundarios_x)
        };
        let pos = |t: &[crate::Tick]| t.iter().map(|t| t.posicao).collect::<Vec<_>>();
        if let Some(g) = &cv.grade_secundaria {
            desenho::grade(&mut cena, eixos.area, sec_v, barras, g);
        }
        if let Some(g) = &cc.grade_secundaria {
            desenho::grade(&mut cena, eixos.area, sec_c, !barras, g);
        }
        let grade_valores = cv.grade_principal.or_else(|| {
            ap.cor_grade.map(|cor| LinhaGrade { cor, espessura: 1.0, traco: TracoLinha::Continuo })
        });
        if let Some(g) = &grade_valores {
            // Grade do eixo de valores: vertical nas barras, horizontal nos demais.
            desenho::grade(&mut cena, eixos.area, &pos(ticks_v), barras, g);
        }
        if let Some(g) = &cc.grade_principal {
            desenho::grade(&mut cena, eixos.area, &pos(ticks_c), !barras, g);
        }
        // Ticks como segmentos vetoriais; texto e formatação pertencem ao backend.
        let (cfg_x, cfg_y) = if barras { (cv, cc) } else { (cc, cv) };
        if cfg_x.mostrar && cfg_x.mostrar_marcas {
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
        }
        if cfg_y.mostrar && cfg_y.mostrar_marcas {
            for tick in &eixos.ticks_y {
                cena.adicionar(Figura::Polilinha {
                    pontos: vec![Ponto::novo(x0, tick.posicao), Ponto::novo(x0 + 4.0_f32.min(w), tick.posicao)],
                    fechada: false,
                    estilo: eixo.clone(),
                });
            }
        }
        let linha_eixos = match (cfg_x.mostrar, cfg_y.mostrar) {
            (true, true) => vec![Ponto::novo(x0, y0), Ponto::novo(x0, y0 + h), Ponto::novo(x0 + w, y0 + h)],
            (true, false) => vec![Ponto::novo(x0, y0 + h), Ponto::novo(x0 + w, y0 + h)],
            (false, true) => vec![Ponto::novo(x0, y0), Ponto::novo(x0, y0 + h)],
            (false, false) => Vec::new(),
        };
        if !linha_eixos.is_empty() {
            cena.adicionar(Figura::Polilinha { pontos: linha_eixos, fechada: false, estilo: eixo.clone() });
        }
        // O zero atravessa a área do gráfico. Para barras é vertical.
        if cv.mostrar {
            cena.adicionar(Figura::Polilinha {
                pontos: if eixos.zero_vertical {
                    vec![Ponto::novo(eixos.zero, y0), Ponto::novo(eixos.zero, y0 + h)]
                } else {
                    vec![Ponto::novo(x0, eixos.zero), Ponto::novo(x0 + w, eixos.zero)]
                },
                fechada: false,
                estilo: eixo,
            });
        }

        match self.tipo {
            Tipo::Colunas | Tipo::Barras => {
                let vertical = self.tipo == Tipo::Colunas;
                let limites: Vec<(f64, f64)> = self.series.iter().map(|s| extremos(&s.valores)).collect();
                let gradiente = ap.barras.as_ref().and_then(|b| b.gradiente.as_ref());
                for (s, i, area) in self.retangulos_barras_em(&eixos, ap)? {
                    let serie = &self.series[s];
                    let so = original(s);
                    let valor = serie.valores[i];
                    let (lo, hi) = limites[s];
                    let cor = ap.cor_ponto(so, i, valor, ap.cor_serie(so, serie.cor), lo, hi);
                    let (contorno, espessura) = ap.contorno_da_serie(so);
                    // Com o eixo de valores invertido a ponta troca de lado.
                    let positivo = (valor >= 0.0) != cv.invertido;
                    let raios = ap
                        .arredondamento
                        .map_or([0.0; 4], |ar| raios_cantos(area, vertical, positivo, PapelBarra::ISOLADA, &ar));
                    if let Some(neon) = &ap.neon {
                        brilho_barra(&mut cena, area, raios, ap.espessura_contorno.max(1.0), cor, neon);
                    }
                    if gradiente.is_some() {
                        let p = desenho::preenchimento_barra(area, vertical, positivo, cor, gradiente);
                        cena.adicionar(desenho::barra_vetorial(area, raios, p, contorno, espessura));
                    } else {
                        cena.adicionar(figura_barra(area, raios, Estilo { preenchimento: Some(cor), contorno, espessura }));
                    }
                }
            }
            Tipo::Linhas | Tipo::Area | Tipo::Degraus | Tipo::Dispersao => {
                let n = self.categorias.len();
                let xs = self.xs_pontos(eixos.area, ap)?;
                for (indice_serie, serie) in self.series.iter().enumerate() {
                    let so = original(indice_serie);
                    let amostras = if self.tipo == Tipo::Linhas {
                        max_pontos.and_then(|limite| {
                            (n > limite).then(|| amostragem::indices_lttb(&serie.valores, limite))
                        })
                    } else {
                        None
                    };
                    let indices: Vec<usize> = match amostras {
                        Some(a) => a.map_err(|_| ErroGrafico::ValorInvalido)?,
                        None => (0..n).collect(),
                    };
                    let pontos: Vec<Ponto> =
                        indices.iter().map(|&i| Ponto::novo(xs[i], mapear_y(serie.valores[i]))).collect();
                    let cor = ap.cor_serie(so, serie.cor);
                    let (lo, hi) = extremos(&serie.valores);
                    let marcadores = |cena: &mut Cena| {
                        for (p, &i) in pontos.iter().zip(&indices) {
                            let propria = ap.cor_propria_do_ponto(so, i, serie.valores[i], lo, hi);
                            if let Some(m) = marca(ap, self.tipo, so, cor, propria) {
                                desenho::marcador(cena, *p, &m);
                            }
                        }
                    };
                    if self.tipo == Tipo::Dispersao {
                        marcadores(&mut cena);
                        continue;
                    }
                    // Marcador global vem antes da linha (como no desenho
                    // original); o da série, por cima dela.
                    let marcador_da_serie = ap.estilo_serie(so).and_then(|e| e.marcador).is_some();
                    if !marcador_da_serie {
                        marcadores(&mut cena);
                    }
                    // Mantém os dados originais intactos. Apenas a geometria
                    // de Área e Degraus muda; a extração/agregação é da origem.
                    let eh_area = self.tipo == Tipo::Area;
                    let estilizada = ap.estiliza_linha(so, eh_area, self.tipo != Tipo::Degraus);
                    let caminho = if self.tipo == Tipo::Degraus {
                        let mut degraus = Vec::with_capacity(pontos.len().saturating_mul(2).saturating_sub(1));
                        if let Some(primeiro) = pontos.first() {
                            degraus.push(*primeiro);
                        }
                        for par in pontos.windows(2) {
                            degraus.push(Ponto::novo(par[1].x, par[0].y));
                            degraus.push(par[1]);
                        }
                        degraus
                    } else {
                        pontos.clone()
                    };
                    if estilizada {
                        desenhar_serie_linha(&mut cena, &caminho, &SerieLinha {
                            cor,
                            indice_serie: so,
                            baseline,
                            area: eh_area,
                            permite_suavizar: self.tipo != Tipo::Degraus,
                            aparencia: ap,
                        });
                    } else {
                        let (caminho, fechada, preenchimento) = if eh_area {
                            // Fecha o polígono no zero REAL (inclusive com
                            // dados negativos), sem transformar valores.
                            let mut contorno = Vec::with_capacity(caminho.len() + 2);
                            if let (Some(primeiro), Some(ultimo)) = (caminho.first().copied(), caminho.last().copied()) {
                                contorno.push(Ponto::novo(primeiro.x, baseline));
                                contorno.extend(caminho);
                                contorno.push(Ponto::novo(ultimo.x, baseline));
                            }
                            let alfa = ((f32::from(cor.a) * 0.35).round() as u8).max(1);
                            (contorno, true, Some(Cor::rgba(cor.r, cor.g, cor.b, alfa)))
                        } else {
                            (caminho, false, None)
                        };
                        cena.adicionar(Figura::Polilinha {
                            pontos: caminho,
                            fechada,
                            estilo: Estilo { preenchimento, contorno: Some(cor), espessura: ap.espessura_da_serie(so) },
                        });
                    }
                    if marcador_da_serie {
                        marcadores(&mut cena);
                    }
                }
            }
            Tipo::Pizza => unreachable!(),
        }
        Ok(Desenho { cena, moldura, eixos: Some(eixos), pizza: None })
    }

    pub(crate) fn cena_impl(
        &self,
        largura: f32,
        altura: f32,
        max_pontos: Option<usize>,
        aparencia: &AparenciaGrafico,
        mapa: Option<&[usize]>,
        original: &Grafico,
    ) -> Result<Cena, ErroGrafico> {
        let comp = Composicao::padrao(original, aparencia);
        Ok(self.desenhar(largura, altura, max_pontos, aparencia, mapa, &comp)?.cena)
    }
}
