//! Peças de desenho da personalização: cartão e fundos, marcadores, traço
//! tracejado, barras com gradiente e geometria da pizza/rosca.
use crate::estilos::caminho_retangulo_arredondado;
use crate::moldura::Moldura;
use crate::personalizacao::{clarear, GradienteBarra, LinhaGrade, TracoLinha};
use crate::quadro::{Cartao, EstiloPizza};
use crate::{AparenciaGrafico, Marcador};
use feather_draw::{
    Caminho, Cena, Comando, Cor, Estilo, EstiloVetor, Extremo, Figura, Juncao, Parada, Ponto,
    Preenchimento, Retangulo, Stroke,
};

/// Traço com padrão de tracejado (vazio = contínuo).
pub(crate) fn stroke(cor: Cor, largura: f32, traco: TracoLinha) -> Stroke {
    Stroke {
        dash: traco.padrao(largura),
        extremo: if traco == TracoLinha::Continuo { Extremo::Redondo } else { Extremo::Plano },
        juncao: Juncao::Redonda,
        ..Stroke::novo(cor, largura)
    }
}

/// Segmento ou polilinha aberta com traço configurável.
pub(crate) fn linha(cena: &mut Cena, pontos: &[Ponto], cor: Cor, largura: f32, traco: TracoLinha) {
    let Some(primeiro) = pontos.first() else { return };
    if traco == TracoLinha::Continuo {
        cena.adicionar(Figura::Polilinha {
            pontos: pontos.to_vec(),
            fechada: false,
            estilo: Estilo { preenchimento: None, contorno: Some(cor), espessura: largura },
        });
        return;
    }
    let mut cmds = vec![Comando::Mover(*primeiro)];
    cmds.extend(pontos[1..].iter().map(|p| Comando::Linha(*p)));
    cena.adicionar(Figura::Caminho {
        caminho: Caminho { comandos: cmds, ..Caminho::novo() },
        estilo: EstiloVetor { preenchimento: None, stroke: Some(stroke(cor, largura, traco)), ..EstiloVetor::default() },
    });
}

/// Linhas de grade numa lista de posições (verticais ou horizontais).
pub(crate) fn grade(cena: &mut Cena, plot: Retangulo, posicoes: &[f32], vertical: bool, g: &LinhaGrade) {
    for &p in posicoes {
        let pts = if vertical {
            [Ponto::novo(p, plot.y), Ponto::novo(p, plot.y + plot.altura)]
        } else {
            [Ponto::novo(plot.x, p), Ponto::novo(plot.x + plot.largura, p)]
        };
        linha(cena, &pts, g.cor, g.espessura, g.traco);
    }
}

fn retangulo_cheio(area: Retangulo, raio: f32, preenchimento: Option<Preenchimento>, stroke: Option<Stroke>) -> Figura {
    let r = raio.min(area.largura / 2.0).min(area.altura / 2.0).max(0.0);
    Figura::Caminho {
        caminho: caminho_retangulo_arredondado(area, [r; 4]),
        estilo: EstiloVetor { preenchimento, stroke, ..EstiloVetor::default() },
    }
}

/// Fundo do gráfico (cartão ou cena inteira) e da área de plotagem.
/// Sem nenhuma opção nova, desenha o `fundo` antigo exatamente como antes.
pub(crate) fn fundos(cena: &mut Cena, ap: &AparenciaGrafico, m: &Moldura, largura: f32, altura: f32) {
    match (&ap.cartao, &ap.fundo_grafico) {
        (None, None) => {
            if let Some(fundo) = ap.fundo {
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
        }
        (cartao, fundo) => {
            let c: Cartao = cartao.unwrap_or(Cartao { raio: 0.0, ..Cartao::default() });
            let area = c.area(largura, altura);
            if let Some(s) = c.sombra {
                // Desfoque aproximado: camadas concêntricas quase transparentes.
                let passos = if s.desfoque > 0.0 { 6 } else { 1 };
                for k in (0..passos).rev() {
                    let extra = s.desfoque * k as f32 / passos as f32;
                    let alfa = (f32::from(s.cor.a) / passos as f32 * if passos == 1 { 1.0 } else { 1.6 })
                        .round()
                        .clamp(1.0, 255.0) as u8;
                    let r = Retangulo::novo(
                        area.x + s.dx - extra,
                        area.y + s.dy - extra,
                        area.largura + 2.0 * extra,
                        area.altura + 2.0 * extra,
                    );
                    cena.adicionar(retangulo_cheio(
                        r,
                        c.raio + extra,
                        Some(Preenchimento::Solido(Cor::rgba(s.cor.r, s.cor.g, s.cor.b, alfa))),
                        None,
                    ));
                }
            }
            let preench = match fundo {
                Some(f) => f.preenchimento(area),
                None => Preenchimento::Solido(ap.fundo.unwrap_or(Cor::rgb(255, 255, 255))),
            };
            let borda = c.borda.filter(|b| b.espessura > 0.0).map(|b| Stroke::novo(b.cor, b.espessura));
            cena.adicionar(retangulo_cheio(area, c.raio, Some(preench), borda));
        }
    }
    if let Some(f) = &ap.fundo_plotagem {
        let p = m.plot;
        cena.adicionar(retangulo_cheio(p, 0.0, Some(f.preenchimento(p)), None));
    }
}

// ------------------------------------------------------------- marcadores

/// Estilo resolvido de um marcador.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Marca {
    pub forma: Marcador,
    pub tamanho: f32,
    pub preenchimento: Cor,
    pub contorno: Option<Cor>,
    pub espessura: f32,
}

pub(crate) fn marcador(cena: &mut Cena, centro: Ponto, m: &Marca) {
    let r = m.tamanho / 2.0;
    let estilo = Estilo { preenchimento: Some(m.preenchimento), contorno: m.contorno, espessura: m.espessura };
    let poligono = |pontos: Vec<Ponto>| Figura::Polilinha { pontos, fechada: true, estilo: estilo.clone() };
    let (x, y) = (centro.x, centro.y);
    match m.forma {
        Marcador::Quadrado => cena.adicionar(Figura::Retangulo {
            area: Retangulo::novo(x - r, y - r, m.tamanho, m.tamanho),
            estilo: estilo.clone(),
        }),
        Marcador::Circulo => cena.adicionar(Figura::Elipse {
            centro,
            raio_x: r,
            raio_y: r,
            estilo: EstiloVetor::from(&estilo),
        }),
        Marcador::Losango => cena.adicionar(poligono(vec![
            Ponto::novo(x, y - r * 1.2),
            Ponto::novo(x + r * 1.2, y),
            Ponto::novo(x, y + r * 1.2),
            Ponto::novo(x - r * 1.2, y),
        ])),
        Marcador::Triangulo => cena.adicionar(poligono(vec![
            Ponto::novo(x, y - r * 1.25),
            Ponto::novo(x + r * 1.15, y + r * 0.85),
            Ponto::novo(x - r * 1.15, y + r * 0.85),
        ])),
        Marcador::Cruz => {
            // Sinal de mais: dois traços na cor de preenchimento.
            let w = (m.tamanho / 4.0).max(1.5);
            let cmds = vec![
                Comando::Mover(Ponto::novo(x - r, y)),
                Comando::Linha(Ponto::novo(x + r, y)),
                Comando::Mover(Ponto::novo(x, y - r)),
                Comando::Linha(Ponto::novo(x, y + r)),
            ];
            cena.adicionar(Figura::Caminho {
                caminho: Caminho { comandos: cmds, ..Caminho::novo() },
                estilo: EstiloVetor {
                    preenchimento: None,
                    stroke: Some(Stroke { extremo: Extremo::Plano, ..Stroke::novo(m.preenchimento, w) }),
                    ..EstiloVetor::default()
                },
            });
        }
        Marcador::Nenhum | Marcador::Automatico => {}
    }
}

// ------------------------------------------------------------------ barras

/// Preenchimento de uma barra: cor sólida ou gradiente da base à ponta.
/// `vertical`/`positivo` dizem onde fica a ponta.
pub(crate) fn preenchimento_barra(
    area: Retangulo,
    vertical: bool,
    positivo: bool,
    cor: Cor,
    g: Option<&GradienteBarra>,
) -> Preenchimento {
    let Some(g) = g else { return Preenchimento::Solido(cor) };
    let mut fim = g.cor_final.unwrap_or_else(|| clarear(cor, 0.55));
    // Mesmo alfa nas duas paradas: o PDF não aceita gradiente de opacidade.
    fim.a = cor.a;
    let (de, ate) = if g.invertido { (fim, cor) } else { (cor, fim) };
    let (base, ponta) = match (vertical, positivo) {
        (true, true) => (Ponto::novo(0.0, area.y + area.altura), Ponto::novo(0.0, area.y)),
        (true, false) => (Ponto::novo(0.0, area.y), Ponto::novo(0.0, area.y + area.altura)),
        (false, true) => (Ponto::novo(area.x, 0.0), Ponto::novo(area.x + area.largura, 0.0)),
        (false, false) => (Ponto::novo(area.x + area.largura, 0.0), Ponto::novo(area.x, 0.0)),
    };
    Preenchimento::Linear {
        de: base,
        ate: ponta,
        paradas: vec![Parada { posicao: 0.0, cor: de }, Parada { posicao: 1.0, cor: ate }],
    }
}

/// Barra como caminho (permite gradiente); cantos como em `figura_barra`.
pub(crate) fn barra_vetorial(area: Retangulo, raios: [f32; 4], preenchimento: Preenchimento, contorno: Option<Cor>, espessura: f32) -> Figura {
    let stroke = contorno.filter(|_| espessura > 0.0).map(|c| Stroke::novo(c, espessura));
    Figura::Caminho {
        caminho: caminho_retangulo_arredondado(area, raios),
        estilo: EstiloVetor { preenchimento: Some(preenchimento), stroke, ..EstiloVetor::default() },
    }
}

/// Branco ou quase preto, o que contrastar mais com `fundo`.
pub(crate) fn cor_contraste(fundo: Cor) -> Cor {
    let l = 0.299 * f32::from(fundo.r) + 0.587 * f32::from(fundo.g) + 0.114 * f32::from(fundo.b);
    if l < 150.0 { Cor::rgb(255, 255, 255) } else { Cor::rgb(32, 32, 32) }
}

// ------------------------------------------------------------------- pizza

/// Centro, raio e opções da pizza/rosca já resolvidos.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GeometriaPizza {
    pub centro: Ponto,
    pub raio: f32,
    /// Raio do furo (0 = pizza cheia).
    pub furo: f32,
    /// Somado aos ângulos de `fatias_pizza` (radianos).
    pub giro: f64,
    pub estilo: EstiloPizza,
}

impl GeometriaPizza {
    /// Sem opções novas: centro da cena e raio de 38% do menor lado, como antes.
    pub fn nova(ap: &AparenciaGrafico, m: &Moldura, largura: f32, altura: f32, fatias: usize) -> Self {
        let estilo = ap.pizza.clone().unwrap_or_default();
        let explosao = estilo.maior_deslocamento(fatias);
        let (centro, mut raio) = if m.legado {
            (Ponto::novo(largura * 0.5, altura * 0.5), largura.min(altura) * 0.38)
        } else {
            let p = m.plot;
            (Ponto::novo(p.x + p.largura / 2.0, p.y + p.altura / 2.0), p.largura.min(p.altura) * 0.44)
        };
        let rotulo_fora = ap.rotulos.as_ref().is_some_and(|r| {
            r.ativo && r.posicao == crate::personalizacao::PosicaoRotulo::Fora
        });
        if !m.legado && rotulo_fora {
            raio *= 0.78;
        }
        raio /= 1.0 + explosao;
        Self {
            centro,
            raio,
            furo: raio * estilo.furo(),
            giro: f64::from(estilo.angulo_inicial).to_radians(),
            estilo,
        }
    }

    /// Deslocamento do centro de uma fatia explodida.
    pub fn deslocamento(&self, indice: usize, meio: f32) -> (f32, f32) {
        let d = self.estilo.deslocamento(indice) * self.raio;
        (d * meio.cos(), d * meio.sin())
    }

    /// Polígono da fatia (setor ou pedaço de anel).
    pub fn poligono(&self, indice: usize, inicio: f32, fim: f32) -> Vec<Ponto> {
        let arco = fim - inicio;
        let passos = ((arco * 24.0).ceil() as usize).clamp(2, 4096);
        let (dx, dy) = self.deslocamento(indice, (inicio + fim) / 2.0);
        let c = Ponto::novo(self.centro.x + dx, self.centro.y + dy);
        let em = |r: f32, a: f32| Ponto::novo(c.x + r * a.cos(), c.y + r * a.sin());
        let mut pontos = Vec::with_capacity(2 * passos + 3);
        if self.furo <= 0.0 {
            pontos.push(c);
        }
        for j in 0..=passos {
            pontos.push(em(self.raio, inicio + arco * j as f32 / passos as f32));
        }
        if self.furo > 0.0 {
            for j in (0..=passos).rev() {
                pontos.push(em(self.furo, inicio + arco * j as f32 / passos as f32));
            }
        }
        pontos
    }

    /// O ponto cai dentro da fatia (com o anel, o giro e a explosão).
    pub fn contem(&self, indice: usize, inicio: f32, fim: f32, p: Ponto, tolerancia: f32) -> bool {
        let (dx, dy) = self.deslocamento(indice, (inicio + fim) / 2.0);
        let (vx, vy) = (p.x - self.centro.x - dx, p.y - self.centro.y - dy);
        let r = (vx * vx + vy * vy).sqrt();
        if r > self.raio + tolerancia || r + tolerancia < self.furo {
            return false;
        }
        let tau = std::f32::consts::TAU;
        let a = vy.atan2(vx);
        let rel = (a - inicio).rem_euclid(tau);
        rel <= (fim - inicio) + 1e-6
    }
}
