//! Interação neutra de gráficos: hit-test, hover, seleção e edição.
//!
//! O app Iced converte seus eventos em `EventoGrafico` e renderiza o retorno;
//! aqui não há widgets, dependência de GUI nem alteração de serialização.

use crate::{AparenciaGrafico, ErroGrafico, FiltroLegenda, Grafico, Tipo, TipografiaGrafico};
use feather_draw::{Ponto, Retangulo};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErroInteracao {
    Grafico(ErroGrafico),
    ViewportInvalida,
    ToleranciaInvalida,
    IndiceInvalido,
    SemSelecao,
    SelecaoObsoleta,
}
impl From<ErroGrafico> for ErroInteracao {
    fn from(erro: ErroGrafico) -> Self { Self::Grafico(erro) }
}

/// Localização do Canvas do app Iced na janela, em pixels físicos ou lógicos.
/// As dimensões da `Cena` são informadas separadamente e NÃO são presumidas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportGrafico {
    pub x: f32,
    pub y: f32,
    pub largura: f32,
    pub altura: f32,
}
impl ViewportGrafico {
    pub fn converter(&self, posicao_tela: Ponto, largura_cena: f32, altura_cena: f32)
        -> Result<Option<Ponto>, ErroInteracao>
    {
        if ![self.x, self.y, self.largura, self.altura,
              posicao_tela.x, posicao_tela.y, largura_cena, altura_cena]
            .iter().all(|v| v.is_finite())
            || self.largura <= 0.0 || self.altura <= 0.0
            || largura_cena <= 0.0 || altura_cena <= 0.0
            || !((self.x as f64 + self.largura as f64).is_finite()
                && (self.y as f64 + self.altura as f64).is_finite())
        {
            return Err(ErroInteracao::ViewportInvalida);
        }
        let caixa = Retangulo::novo(self.x, self.y, self.largura, self.altura);
        if !caixa.contem(posicao_tela) { return Ok(None); }
        let p = Ponto::novo(
            (posicao_tela.x - self.x) / self.largura * largura_cena,
            (posicao_tela.y - self.y) / self.altura * altura_cena,
        );
        if !p.x.is_finite() || !p.y.is_finite() {
            return Err(ErroInteracao::ViewportInvalida);
        }
        Ok(Some(p))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlvoGrafico {
    pub serie: usize,
    pub categoria: usize,
    pub nome_serie: String,
    pub rotulo_categoria: String,
    pub valor: f64,
}
impl AlvoGrafico {
    pub fn tooltip(&self) -> String {
        format!("{} — {}: {}", self.nome_serie, self.rotulo_categoria, self.valor)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HitGrafico {
    pub alvo: AlvoGrafico,
    /// Distância do cursor ao centro/segmento em unidades de Cena.
    pub distancia: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventoGrafico {
    /// Coordenada do cursor na janela. Apenas atualiza hover.
    Mover(Ponto),
    /// Seleciona ponto, ou limpa seleção se estiver fora do elemento.
    Clicar(Ponto),
    /// Retira hover sem apagar seleção.
    Sair,
    /// Esc / clique na área externa implementado pelo consumidor.
    LimparSelecao,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MudancaInteracao {
    pub hover: bool,
    pub selecao: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct InteracaoGrafico {
    pub hover: Option<AlvoGrafico>,
    pub selecionado: Option<AlvoGrafico>,
    /// Itens da legenda escondidos (séries; fatias na pizza). Vazio = tudo visível.
    pub filtro: FiltroLegenda,
}

/// Projeção ortogonal no segmento, evitando divisões por zero.
fn distancia_segmento(p: Ponto, a: Ponto, b: Ponto) -> (f32, f32) {
    let vx = b.x - a.x;
    let vy = b.y - a.y;
    let d2 = vx * vx + vy * vy;
    if d2 <= f32::EPSILON {
        return (distancia(p, a), 0.0);
    }
    let t = (((p.x - a.x) * vx + (p.y - a.y) * vy) / d2).clamp(0.0, 1.0);
    (distancia(p, Ponto::novo(a.x + vx * t, a.y + vy * t)), t)
}
fn distancia(a: Ponto, b: Ponto) -> f32 {
    (a.x - b.x).hypot(a.y - b.y)
}

impl Grafico {
    fn alvo(&self, serie: usize, categoria: usize) -> AlvoGrafico {
        AlvoGrafico {
            serie, categoria,
            nome_serie: self.series[serie].nome.clone(),
            rotulo_categoria: self.categorias[categoria].clone(),
            valor: self.series[serie].valores[categoria],
        }
    }

    /// Teste de geometria fiel a `cena_estilizada` com margens/escala do app.
    /// Barras usam seus retângulos vetoriais, pizza os setores desenhados,
    /// linhas e dispersão as coordenadas dos pontos originais.
    ///
    /// A tolerância é em unidades da Cena (independente de DPI Iced).
    /// Não altera valores nem a visualização em `Grafico`.
    pub fn hit_test_dados(
        &self, largura: f32, altura: f32, ponto: Ponto,
        tolerancia: f32, aparencia: &AparenciaGrafico,
    ) -> Result<Option<HitGrafico>, ErroInteracao> {
        if !tolerancia.is_finite() || tolerancia < 0.0 {
            return Err(ErroInteracao::ToleranciaInvalida);
        }
        if !ponto.x.is_finite() || !ponto.y.is_finite() {
            return Err(ErroInteracao::ViewportInvalida);
        }
        self.validar()?;
        aparencia.validar()?;
        let comp = crate::moldura::Composicao::padrao(self, aparencia);
        let moldura = self.moldura(largura, altura, aparencia, &comp)?;
        if !Retangulo::novo(0.0, 0.0, largura, altura).contem(ponto) {
            return Ok(None);
        }
        match self.tipo {
            Tipo::Colunas | Tipo::Barras => {
                // Ordem série-maior/categoria-menor; o último retângulo fica
                // visualmente acima de qualquer anterior em caso de overlap.
                let barras = self.retangulos_barras(largura, altura, aparencia)?;
                for (serie, categoria, area) in barras.iter().rev() {
                    let exp = Retangulo::novo(
                        area.x - tolerancia,
                        area.y - tolerancia,
                        area.largura + 2.0 * tolerancia,
                        area.altura + 2.0 * tolerancia,
                    );
                    if exp.contem(ponto) {
                        let centro = Ponto::novo(
                            area.x + area.largura / 2.0,
                            area.y + area.altura / 2.0,
                        );
                        return Ok(Some(HitGrafico {
                            alvo: self.alvo(*serie, *categoria),
                            distancia: distancia(ponto, centro),
                        }));
                    }
                }
                Ok(None)
            }
            Tipo::Pizza => {
                // Geometria da fatia (giro, rosca e explosão), não a da figura.
                let geo = crate::desenho::GeometriaPizza::nova(
                    aparencia, &moldura, largura, altura, self.categorias.len());
                let fatias = self.fatias_pizza(largura, altura)?;
                // Fatias zero nunca geram área, índices originais preservados.
                for fatia in fatias.iter().rev() {
                    let (inicio, fim) = ((fatia.inicio + geo.giro) as f32, (fatia.fim + geo.giro) as f32);
                    if geo.contem(fatia.indice, inicio, fim, ponto, tolerancia) {
                        return Ok(Some(HitGrafico {
                            alvo: self.alvo(0, fatia.indice), distancia: 0.0,
                        }));
                    }
                }
                Ok(None)
            }
            Tipo::Linhas | Tipo::Area | Tipo::Degraus | Tipo::Dispersao => {
                let eixos = self.eixos_na_area(moldura.plot, aparencia)?
                    .ok_or(ErroInteracao::Grafico(ErroGrafico::EixoInvalido))?;
                let xs = self.xs_pontos(eixos.area, aparencia)?;
                let area = eixos.area;
                // Nenhum ponto fora da área dos eixos deve captar clique,
                // mesmo que esteja próximo de uma borda da janela.
                let plot = Retangulo::novo(
                    area.x - tolerancia, area.y - tolerancia,
                    area.largura + 2.0 * tolerancia,
                    area.altura + 2.0 * tolerancia,
                );
                if !plot.contem(ponto) { return Ok(None); }
                let escala_y = eixos.escala_y.ok_or(ErroGrafico::EixoInvalido)?;
                let mut melhor: Option<HitGrafico> = None;
                for (serie_idx, serie) in self.series.iter().enumerate() {
                    let mut anterior: Option<(usize, Ponto)> = None;
                    for (categoria_idx, &valor) in serie.valores.iter().enumerate() {
                        let x = xs[categoria_idx];
                        let y = area.y + area.altura * (1.0 - escala_y.fracao(valor));
                        let atual = Ponto::novo(x, y);
                        // Array fixo: nenhuma alocação por amostra, inclusive
                        // com séries de milhões de pontos para hover Iced.
                        let mut candidatos = [
                            (categoria_idx, distancia(ponto, atual)),
                            (categoria_idx, f32::INFINITY),
                        ];
                        if matches!(self.tipo, Tipo::Linhas | Tipo::Area | Tipo::Degraus) {
                            if let Some((idx_anterior, p_anterior)) = anterior {
                                if self.tipo == Tipo::Degraus {
                                    // No degrau o caminho horizontal termina em
                                    // (x_atual, y_anterior), depois cai/sobe.
                                    let dobra = Ponto::novo(atual.x, p_anterior.y);
                                    let (horizontal, _) = distancia_segmento(ponto, p_anterior, dobra);
                                    let (vertical, _) = distancia_segmento(ponto, dobra, atual);
                                    candidatos[1] = if horizontal <= vertical {
                                        (idx_anterior, horizontal)
                                    } else {
                                        (categoria_idx, vertical)
                                    };
                                } else {
                                    let (d, t) = distancia_segmento(ponto, p_anterior, atual);
                                    candidatos[1] = (if t <= 0.5 { idx_anterior } else { categoria_idx }, d);
                                }
                            }
                        }
                        for (indice, d) in candidatos {
                            if d <= tolerancia && melhor.as_ref().map_or(true, |h| d < h.distancia) {
                                melhor = Some(HitGrafico {
                                    alvo: self.alvo(serie_idx, indice), distancia: d,
                                });
                            }
                        }
                        anterior = Some((categoria_idx, atual));
                    }
                }
                Ok(melhor)
            }
        }
    }

    /// Como `hit_test_dados`, ignorando os itens escondidos pelo filtro da
    /// legenda. Os índices do alvo são os do gráfico original.
    pub fn hit_test_dados_filtrado(
        &self, largura: f32, altura: f32, ponto: Ponto,
        tolerancia: f32, aparencia: &AparenciaGrafico, filtro: &FiltroLegenda,
    ) -> Result<Option<HitGrafico>, ErroInteracao> {
        if !filtro.filtrando() {
            return self.hit_test_dados(largura, altura, ponto, tolerancia, aparencia);
        }
        self.validar()?;
        let (g, mapa) = self.filtrado(filtro)?;
        let hit = g.hit_test_dados(largura, altura, ponto, tolerancia, aparencia)?;
        Ok(hit.map(|mut h| {
            if self.tipo == Tipo::Pizza {
                h.alvo.categoria = mapa[h.alvo.categoria];
            } else {
                h.alvo.serie = mapa[h.alvo.serie];
            }
            h
        }))
    }

    /// Edição atômica de uma amostra pelo par (série, categoria).
    /// Regras de pizza e tipos numéricos são revalidadas sem alterar o resto.
    pub fn editar_valor(
        &mut self, serie: usize, categoria: usize, novo: f64,
    ) -> Result<f64, ErroInteracao> {
        self.validar()?;
        if !novo.is_finite() { return Err(ErroInteracao::Grafico(ErroGrafico::ValorInvalido)); }
        let anterior = *self.series.get(serie)
            .and_then(|s| s.valores.get(categoria))
            .ok_or(ErroInteracao::IndiceInvalido)?;
        self.series[serie].valores[categoria] = novo;
        if let Err(e) = self.validar() {
            self.series[serie].valores[categoria] = anterior;
            return Err(e.into());
        }
        Ok(anterior)
    }
}

impl InteracaoGrafico {
    /// Ponto da janela Iced -> Cena -> alvo. `false/false` permite ao app
    /// evitar `request_redraw` quando o ponteiro não mudou o estado visual.
    pub fn processar(
        &mut self, evento: EventoGrafico, grafico: &Grafico,
        largura_cena: f32, altura_cena: f32,
        viewport: ViewportGrafico, tolerancia: f32,
        aparencia: &AparenciaGrafico,
    ) -> Result<MudancaInteracao, ErroInteracao> {
        let hover_antigo = self.hover.clone();
        let selecao_antiga = self.selecionado.clone();
        match evento {
            EventoGrafico::Mover(tela) | EventoGrafico::Clicar(tela) => {
                let ponto = viewport.converter(tela, largura_cena, altura_cena)?;
                let novo = if let Some(ponto) = ponto {
                    grafico.hit_test_dados_filtrado(largura_cena, altura_cena, ponto,
                        tolerancia, aparencia, &self.filtro)?.map(|h| h.alvo)
                } else { None };
                self.hover = novo.clone();
                if matches!(evento, EventoGrafico::Clicar(_)) {
                    self.selecionado = novo;
                }
            }
            EventoGrafico::Sair => self.hover = None,
            EventoGrafico::LimparSelecao => self.selecionado = None,
        }
        Ok(MudancaInteracao {
            hover: self.hover != hover_antigo,
            selecao: self.selecionado != selecao_antiga,
        })
    }

    /// Clique na legenda. Sem `ctrl` alterna o item; com `ctrl` mostra só ele
    /// (repetir devolve tudo). Devolve o item acertado (índice da série, ou da
    /// categoria na pizza) ou `None` se o ponto não caiu em nenhum. Hover e
    /// seleção de itens que sumiram são limpos.
    #[allow(clippy::too_many_arguments)]
    pub fn clicar_legenda(
        &mut self, grafico: &Grafico, largura: f32, altura: f32,
        config: &TipografiaGrafico, aparencia: &AparenciaGrafico,
        ponto: Ponto, ctrl: bool,
    ) -> Result<Option<usize>, ErroInteracao> {
        grafico.validar()?;
        let Some(item) = grafico.hit_test_legenda(
            largura, altura, config, aparencia, &self.filtro, ponto) else { return Ok(None) };
        let total = grafico.total_itens_legenda();
        if ctrl {
            self.filtro.mostrar_so(item, total);
        } else {
            self.filtro.alternar(item, total);
        }
        let sumiu = |a: &AlvoGrafico| {
            self.filtro.esta_oculto(if grafico.tipo == Tipo::Pizza { a.categoria } else { a.serie })
        };
        if self.hover.as_ref().is_some_and(sumiu) { self.hover = None; }
        if self.selecionado.as_ref().is_some_and(sumiu) { self.selecionado = None; }
        Ok(Some(item))
    }

    /// Duplo clique (ou botão "limpar filtro"): volta a mostrar tudo.
    pub fn limpar_filtro(&mut self) {
        self.filtro.mostrar_todos();
    }

    /// Valida identidade/valor antes de editar. Não cria histórico: o
    /// consumidor de `Grafico` controla transações de undo/redo.
    pub fn editar_selecionado(
        &mut self, grafico: &mut Grafico, novo: f64,
    ) -> Result<f64, ErroInteracao> {
        let alvo = self.selecionado.clone().ok_or(ErroInteracao::SemSelecao)?;
        let serie = grafico.series.get(alvo.serie).ok_or(ErroInteracao::SelecaoObsoleta)?;
        if serie.nome != alvo.nome_serie ||
            grafico.categorias.get(alvo.categoria) != Some(&alvo.rotulo_categoria) ||
            serie.valores.get(alvo.categoria) != Some(&alvo.valor) {
            return Err(ErroInteracao::SelecaoObsoleta);
        }
        let anterior = grafico.editar_valor(alvo.serie, alvo.categoria, novo)?;
        if let Some(atual) = &mut self.selecionado {
            atual.valor = novo;
        }
        if let Some(hover) = &mut self.hover {
            if hover.serie == alvo.serie && hover.categoria == alvo.categoria {
                hover.valor = novo;
            }
        }
        Ok(anterior)
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use feather_draw::{Cor, Figura};

    fn chart(tipo: Tipo) -> Grafico {
        Grafico {
            titulo: "Interação".into(), tipo,
            categorias: vec!["0".into(), "1".into(), "2".into()],
            series: vec![
                crate::Serie { nome: "A".into(), valores: vec![10.0, 20.0, 30.0], cor: Cor::rgb(10, 20, 30) },
                crate::Serie { nome: "B".into(), valores: vec![5.0, 8.0, 12.0], cor: Cor::rgb(70, 80, 90) },
            ],
        }
    }

    fn clicavel(g: &Grafico, tipo: Tipo, serie: usize, categoria: usize) -> Ponto {
        let cena = g.cena(600.0, 400.0).unwrap();
        match tipo {
            Tipo::Colunas | Tipo::Barras => {
                let areas: Vec<_> = cena.figuras.iter().filter_map(|f| {
                    if let Figura::Retangulo { area, .. } = f { Some(*area) } else { None }
                }).collect();
                let a = areas[serie * g.categorias.len() + categoria];
                Ponto::novo(a.x + a.largura / 2.0, a.y + a.altura / 2.0)
            }
            Tipo::Linhas | Tipo::Area | Tipo::Degraus | Tipo::Dispersao => {
                let e = g.eixos(600.0, 400.0).unwrap().unwrap();
                let x = if tipo != Tipo::Dispersao {
                    e.area.x + e.area.largura * categoria as f32 / (g.categorias.len() - 1) as f32
                } else {
                    e.area.x + e.area.largura * e.escala_x.unwrap().fracao(
                        g.categorias[categoria].parse().unwrap())
                };
                let y = e.area.y + e.area.altura * (1.0
                    - e.escala_y.unwrap().fracao(g.series[serie].valores[categoria]));
                Ponto::novo(x, y)
            }
            Tipo::Pizza => unreachable!(),
        }
    }

    #[test]
    fn hit_colunas_barras_lin_disper_retorna_dados_originais() {
        for tipo in [Tipo::Colunas, Tipo::Barras, Tipo::Linhas, Tipo::Dispersao] {
            let g = chart(tipo);
            let p = clicavel(&g, tipo, 0, 1);
            let hit = g.hit_test_dados(600.0, 400.0, p, 0.1, &Default::default()).unwrap().unwrap();
            assert_eq!((hit.alvo.serie, hit.alvo.categoria), (0, 1), "{tipo:?}");
            assert_eq!(hit.alvo.valor, 20.0);
            assert_eq!(hit.alvo.rotulo_categoria, "1");
            assert!(g.hit_test_dados(600.0, 400.0,
                Ponto::novo(-50.0, -50.0), 6.0, &Default::default()).unwrap().is_none());
        }
    }

    #[test]
    fn pizza_mantem_indice_original_com_fatia_zero() {
        let mut g = chart(Tipo::Pizza);
        g.series.truncate(1);
        g.series[0].valores = vec![10.0, 0.0, 10.0];
        let fatias = g.fatias_pizza(600.0, 400.0).unwrap();
        assert_eq!(fatias.len(), 2);
        let centro = Ponto::novo(300.0, 200.0);
        for f in fatias {
            let a = (f.inicio + f.fim) * 0.5;
            let p = Ponto::novo(centro.x + a.cos() as f32 * 80.0,
                centro.y + a.sin() as f32 * 80.0);
            let hit = g.hit_test_dados(600.0, 400.0, p, 0.0, &Default::default()).unwrap().unwrap();
            assert_eq!(hit.alvo.categoria, f.indice);
        }
    }

    #[test]
    fn viewport_normaliza_posicao_e_clique_persistente() {
        let g = chart(Tipo::Colunas);
        let mut estado = InteracaoGrafico::default();
        let view = ViewportGrafico { x: 100.0, y: 50.0, largura: 1200.0, altura: 800.0 };
        let p = clicavel(&g, Tipo::Colunas, 0, 0);
        let tela = Ponto::novo(100.0 + p.x * 2.0, 50.0 + p.y * 2.0);
        let h = estado.processar(EventoGrafico::Mover(tela), &g, 600.0, 400.0,
            view, 2.0, &Default::default()).unwrap();
        assert!(h.hover && !h.selecao);
        let c = estado.processar(EventoGrafico::Clicar(tela), &g, 600.0, 400.0,
            view, 2.0, &Default::default()).unwrap();
        assert!(c.selecao && !c.hover);
        assert_eq!(estado.selecionado.as_ref().unwrap().valor, 10.0);
        estado.processar(EventoGrafico::Sair, &g, 600.0, 400.0,
            view, 2.0, &Default::default()).unwrap();
        assert!(estado.hover.is_none());
        assert!(estado.selecionado.is_some());
        estado.processar(EventoGrafico::LimparSelecao, &g, 600.0, 400.0,
            view, 2.0, &Default::default()).unwrap();
        assert!(estado.selecionado.is_none());
    }

    #[test]
    fn edicao_atomica_rejeita_pizza_negativa_e_preserva_estado() {
        let mut g = chart(Tipo::Pizza);
        g.series.truncate(1);
        let mut estado = InteracaoGrafico {
            hover: None, selecionado: Some(g.alvo(0, 0)), ..Default::default()
        };
        assert_eq!(estado.editar_selecionado(&mut g, -1.0),
            Err(ErroInteracao::Grafico(ErroGrafico::ValorInvalido)));
        assert_eq!(g.series[0].valores[0], 10.0);
        assert_eq!(estado.selecionado.as_ref().unwrap().valor, 10.0);
        assert_eq!(estado.editar_selecionado(&mut g, 18.0).unwrap(), 10.0);
        assert_eq!(g.series[0].valores[0], 18.0);
        assert_eq!(estado.selecionado.as_ref().unwrap().valor, 18.0);
        assert_eq!(estado.editar_selecionado(&mut g, f64::NAN),
            Err(ErroInteracao::Grafico(ErroGrafico::ValorInvalido)));
    }

    #[test]
    fn edicao_rejeita_selecao_obsoleta_e_indices_invalidos() {
        let mut g = chart(Tipo::Barras);
        let mut estado = InteracaoGrafico {
            hover: None, selecionado: Some(g.alvo(1, 2)), ..Default::default() };
        g.series[1].valores[2] = 100.0;
        assert_eq!(estado.editar_selecionado(&mut g, 90.0), Err(ErroInteracao::SelecaoObsoleta));
        assert_eq!(g.series[1].valores[2], 100.0);
        assert_eq!(g.editar_valor(5, 2, 1.0), Err(ErroInteracao::IndiceInvalido));
    }

    #[test]
    fn rejeita_tolerancia_viewport_ou_dados_invalidos() {
        let g = chart(Tipo::Linhas);
        assert_eq!(g.hit_test_dados(600.0, 400.0, Ponto::novo(0.0, 0.0),
            f32::NAN, &Default::default()), Err(ErroInteracao::ToleranciaInvalida));
        let view = ViewportGrafico { x: 0.0, y: 0.0, largura: 0.0, altura: 100.0 };
        assert_eq!(view.converter(Ponto::novo(0.0, 0.0), 600.0, 400.0),
            Err(ErroInteracao::ViewportInvalida));
        let mut ruim = chart(Tipo::Dispersao);
        ruim.categorias[0] = "abc".into();
        assert_eq!(ruim.hit_test_dados(600.0, 400.0, Ponto::novo(80.0, 80.0),
            2.0, &Default::default()), Err(ErroInteracao::Grafico(ErroGrafico::EixoInvalido)));
    }

    #[test]
    fn aparencia_altera_area_hit_sem_afetar_valores() {
        let g = chart(Tipo::Colunas);
        let ap = AparenciaGrafico {
            layout: crate::LayoutGrafico {
                esquerda: 0.25, direita: 0.15, superior: 0.2, inferior: 0.15,
            },
            ..Default::default()
        };
        let area = g.cena_estilizada(600.0, 400.0, &ap).unwrap();
        let pontos: Vec<_> = area.figuras.iter().filter_map(|fig| {
            if let Figura::Retangulo { area, .. } = fig {
                Some(Ponto::novo(area.x + area.largura / 2.0, area.y + area.altura / 2.0))
            } else { None }
        }).collect();
        let hit = g.hit_test_dados(600.0, 400.0, pontos[0], 0.01, &ap).unwrap().unwrap();
        assert_eq!((hit.alvo.serie, hit.alvo.categoria), (0, 0));
        assert_eq!(g.series[0].valores[0], 10.0);
    }

    #[test]
    fn degraus_hit_test_somente_na_geometria_ortogonal() {
        let mut g = chart(Tipo::Degraus);
        g.series.truncate(1);
        g.categorias.truncate(2);
        g.series[0].valores = vec![10.0, 30.0];
        let e = g.eixos(600.0, 400.0).unwrap().unwrap();
        let px0 = e.area.x;
        let px1 = e.area.x + e.area.largura;
        let py0 = e.area.y + e.area.altura * (1.0 - e.escala_y.unwrap().fracao(10.0));
        let py1 = e.area.y + e.area.altura * (1.0 - e.escala_y.unwrap().fracao(30.0));
        let horizontal = Ponto::novo((px0 + px1) / 2.0, py0);
        let h = g.hit_test_dados(600.0, 400.0, horizontal, 1.0, &Default::default()).unwrap();
        assert_eq!(h.unwrap().alvo.categoria, 0);
        let vertical = Ponto::novo(px1, (py0 + py1) / 2.0);
        let v = g.hit_test_dados(600.0, 400.0, vertical, 1.0, &Default::default()).unwrap();
        assert_eq!(v.unwrap().alvo.categoria, 1);
        let diagonal = Ponto::novo((px0 + px1) / 2.0, (py0 + py1) / 2.0);
        assert!(g.hit_test_dados(600.0, 400.0, diagonal, 1.0, &Default::default()).unwrap().is_none());
    }

    #[test]
    fn linha_aceita_hit_no_segmento_entre_pontos() {
        let g = chart(Tipo::Linhas);
        let a = clicavel(&g, Tipo::Linhas, 0, 0);
        let b = clicavel(&g, Tipo::Linhas, 0, 1);
        let meio = Ponto::novo((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
        let hit = g.hit_test_dados(600.0, 400.0, meio, 1.0, &Default::default()).unwrap().unwrap();
        assert_eq!(hit.alvo.serie, 0);
        assert!(hit.alvo.categoria == 0 || hit.alvo.categoria == 1);
    }
}
