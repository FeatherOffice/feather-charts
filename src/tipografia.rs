//! Tipografia e formatação regional compartilhadas para gráficos.
//!
//! A geometria continua em Grafico::cena e o texto é um overlay declarativo,
//! independente da GUI. Exportadores SVG/PDF usam o mesmo layout e conteúdo.

use crate::filtro::{FiltroLegenda, ItemLegenda};
use crate::moldura::{largura_item_legenda, quebrar_legenda, texto_tick, Composicao, Moldura};
use crate::personalizacao::RotacaoRotulos;
use crate::quadro::{AlinhamentoTitulo, PosicaoLegenda};
use crate::{AparenciaGrafico, ErroGrafico, Grafico, LayoutGrafico, Tipo};
use feather_draw::{
    exportar_pdf_com_texto, exportar_svg, Cena, Cor, ErroPdf, ErroSvg,
    Estilo, Figura, FontePdf, Ponto, Retangulo, TextoPdf,
};
use serde::{Deserialize, Serialize};
use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalidadeGrafico {
    PtBr,
    EnUs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormatoNumero {
    Decimal(u8),
    Moeda(u8),
    Percentual(u8),
}

impl FormatoNumero {
    /// Não inclui unidade textual: ela pertence ao layout, não ao dado.
    /// Números excessivamente grandes usam notação exponencial para limitar
    /// alocações. NaN/infinito e mais de seis decimais são recusados.
    pub fn formatar(self, valor: f64, localidade: LocalidadeGrafico) -> Result<String, ErroGrafico> {
        let (casas, moeda, porcentagem) = match self {
            Self::Decimal(n) => (n, false, false),
            Self::Moeda(n) => (n, true, false),
            Self::Percentual(n) => (n, false, true),
        };
        if !valor.is_finite() || casas > 6 {
            return Err(ErroGrafico::ValorInvalido);
        }
        let numero = if porcentagem { valor * 100.0 } else { valor };
        if !numero.is_finite() {
            return Err(ErroGrafico::ValorInvalido);
        }
        let sinal = if numero < 0.0 { "-" } else { "" };
        let numero = numero.abs();
        let sufixo = if porcentagem { "%" } else { "" };
        let prefixo = if moeda {
            match localidade { LocalidadeGrafico::PtBr => "R$ ", LocalidadeGrafico::EnUs => "$" }
        } else { "" };

        if numero >= 1e15 {
            let exp = format!("{:.*e}", casas as usize, numero);
            let exp = if localidade == LocalidadeGrafico::PtBr {
                exp.replacen('.', ",", 1)
            } else { exp };
            return Ok(format!("{sinal}{prefixo}{exp}{sufixo}"));
        }

        let bruto = format!("{:.*}", casas as usize, numero);
        let (inteiro, fracao) = bruto.split_once('.').unwrap_or((&bruto, ""));
        let mut separado = String::with_capacity(inteiro.len() + inteiro.len() / 3);
        let milhar = match localidade {
            LocalidadeGrafico::PtBr => '.',
            LocalidadeGrafico::EnUs => ',',
        };
        for (i, c) in inteiro.chars().enumerate() {
            if i > 0 && (inteiro.len() - i) % 3 == 0 {
                separado.push(milhar);
            }
            separado.push(c);
        }
        if casas > 0 {
            separado.push(match localidade {
                LocalidadeGrafico::PtBr => ',',
                LocalidadeGrafico::EnUs => '.',
            });
            separado.push_str(fracao);
        }
        Ok(format!("{sinal}{prefixo}{separado}{sufixo}"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlinhamentoRotulo {
    Inicio,
    Centro,
    Fim,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RotuloGrafico {
    pub texto: String,
    /// Posição da baseline no sistema de coordenadas da Cena.
    pub x: f32,
    pub y: f32,
    pub tamanho: f32,
    pub cor: Cor,
    pub alinhamento: AlinhamentoRotulo,
    pub negrito: bool,
    /// Giro em graus, sentido anti-horário, em torno de (x, y): 90 escreve de
    /// baixo para cima. O PDF do `feather-draw` não gira texto: lá sai reto.
    pub rotacao: f32,
}

/// Anotações separadas das formas, para serem consumidas por Iced ou PDF/SVG.
#[derive(Debug, Clone, PartialEq)]
pub struct CenaAnotada {
    pub cena: Cena,
    pub rotulos: Vec<RotuloGrafico>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TipografiaGrafico {
    pub subtitulo: String,
    pub unidade: String,
    pub localidade: LocalidadeGrafico,
    pub formato: FormatoNumero,
    pub mostrar_titulo: bool,
    pub mostrar_legenda: bool,
    pub mostrar_eixos: bool,
    pub mostrar_valores: bool,
    /// Tamanho do texto em unidades da cena. 10 por padrão.
    pub tamanho: f32,
}

impl Default for TipografiaGrafico {
    fn default() -> Self {
        Self {
            subtitulo: String::new(),
            unidade: String::new(),
            localidade: LocalidadeGrafico::PtBr,
            formato: FormatoNumero::Decimal(1),
            mostrar_titulo: true,
            mostrar_legenda: true,
            mostrar_eixos: true,
            mostrar_valores: false,
            tamanho: 10.0,
        }
    }
}

impl TipografiaGrafico {
    pub fn validar(&self) -> Result<(), ErroGrafico> {
        if !self.tamanho.is_finite()
            || !(4.0..=32.0).contains(&self.tamanho)
            || self.subtitulo.len() > 4096
            || self.unidade.len() > 128
        {
            return Err(ErroGrafico::ValorInvalido);
        }
        self.formato.formatar(0.0, self.localidade)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErroExportacaoTipografica {
    Grafico(ErroGrafico),
    Svg(ErroSvg),
    Pdf(ErroPdf),
}

fn rotulo(texto: String, x: f32, y: f32, tamanho: f32, cor: Cor, alinhamento: AlinhamentoRotulo) -> RotuloGrafico {
    RotuloGrafico { texto, x, y, tamanho, cor, alinhamento, negrito: false, rotacao: 0.0 }
}
fn abreviar(texto: &str, largura: f32, tamanho: f32) -> String {
    let maximo = if largura <= 0.0 || tamanho <= 0.0 { 0 }
        else { (largura / (tamanho * 0.60)).floor() as usize };
    let total = texto.chars().count();
    if total <= maximo { return texto.to_owned(); }
    if maximo < 2 { return String::new(); }
    let mut saida: String = texto.chars().take(maximo - 1).collect();
    saida.push('…');
    saida
}
/// Mistura a cor com o fundo para indicar item desativado (resultado opaco,
/// para SVG e PDF mostrarem igual).
fn esmaecer(cor: Cor, fundo: Cor) -> Cor {
    let mix = |a: u8, b: u8| (f32::from(a) * 0.35 + f32::from(b) * 0.65).round() as u8;
    Cor::rgb(mix(cor.r, fundo.r), mix(cor.g, fundo.g), mix(cor.b, fundo.b))
}
fn escapar_svg(t: &str) -> String {
    let mut out = String::with_capacity(t.len());
    for c in t.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c if c.is_control() => out.push(' '),
            _ => out.push(c),
        }
    }
    out
}

impl Grafico {
    /// Linhas de base do título e do subtítulo no layout antigo (nunca cortados em cima).
    fn bases_do_titulo(altura: f32, fonte: f32) -> (f32, f32) {
        let titulo = (altura * 0.048).max(fonte * 1.4 * 0.95);
        (titulo, (altura * 0.087).max(titulo + fonte * 1.2))
    }

    /// A área do gráfico da cena anotada no layout antigo: título e subtítulo em
    /// cima, rótulos do eixo X e legenda embaixo ficam fora dela, para não pisarem
    /// no desenho nem um no outro (gráfico baixo ou letra grande). Com folga, é o
    /// layout padrão.
    pub fn layout_anotado(&self, _largura: f32, altura: f32, config: &TipografiaGrafico) -> LayoutGrafico {
        let padrao = LayoutGrafico::default();
        if self.tipo == Tipo::Pizza || altura <= 0.0 {
            return padrao;
        }
        let fonte = config.tamanho;
        let (base_titulo, base_subtitulo) = Self::bases_do_titulo(altura, fonte);
        let topo = if !config.subtitulo.is_empty() {
            base_subtitulo + fonte * 0.6
        } else if config.mostrar_titulo && !self.titulo.is_empty() {
            base_titulo + fonte * 0.6
        } else {
            0.0
        };
        let eixo = if config.mostrar_eixos { fonte * 1.75 } else { 0.0 };
        let legenda = if config.mostrar_legenda { fonte * 1.7 } else { 0.0 };
        let l = LayoutGrafico {
            superior: padrao.superior.max(topo / altura),
            inferior: padrao.inferior.max((eixo + legenda + fonte * 0.3) / altura),
            ..padrao
        };
        if l.superior + l.inferior < 0.8 { l } else { padrao }
    }

    /// Aparência efetiva da cena anotada: no layout antigo com as margens padrão,
    /// as margens de `layout_anotado`; nos demais casos, a própria.
    fn aparencia_anotada<'a>(&self, largura: f32, altura: f32, config: &TipografiaGrafico, ap: &'a AparenciaGrafico) -> std::borrow::Cow<'a, AparenciaGrafico> {
        if !ap.usa_moldura() && ap.layout == LayoutGrafico::default() {
            std::borrow::Cow::Owned(AparenciaGrafico { layout: self.layout_anotado(largura, altura, config), ..ap.clone() })
        } else {
            std::borrow::Cow::Borrowed(ap)
        }
    }

    /// Composição tipográfica sem mudar Grafico/Serie ou persistência nativa.
    /// Os eixos conservam as posições retornadas pelo motor CHT-006.
    pub fn cena_anotada(
        &self,
        largura: f32,
        altura: f32,
        config: &TipografiaGrafico,
    ) -> Result<CenaAnotada, ErroGrafico> {
        self.cena_anotada_impl(
            largura, altura, config, &AparenciaGrafico::default(), &FiltroLegenda::default(), false)
    }

    /// Como `cena_anotada`, mas com aparência (fundo, cores de texto, efeitos)
    /// e filtro da legenda. Séries (ou fatias, na pizza) escondidas somem do
    /// desenho, dos eixos e dos totais, e continuam na legenda, esmaecidas. Na
    /// pizza a legenda lista as categorias, para que cada fatia possa ser filtrada.
    pub fn cena_anotada_estilizada(
        &self,
        largura: f32,
        altura: f32,
        config: &TipografiaGrafico,
        aparencia: &AparenciaGrafico,
        filtro: &FiltroLegenda,
    ) -> Result<CenaAnotada, ErroGrafico> {
        self.cena_anotada_impl(largura, altura, config, aparencia, filtro, true)
    }

    fn cena_anotada_impl(
        &self,
        largura: f32,
        altura: f32,
        config: &TipografiaGrafico,
        aparencia: &AparenciaGrafico,
        filtro: &FiltroLegenda,
        pizza_por_categoria: bool,
    ) -> Result<CenaAnotada, ErroGrafico> {
        config.validar()?;
        let aparencia = &*self.aparencia_anotada(largura, altura, config, aparencia);
        let (vis, mapa) = self.filtrado(filtro)?;
        let comp = Composicao::da_tipografia(self, config, pizza_por_categoria);
        let desenho = vis.desenhar(largura, altura, None, aparencia, Some(&mapa), &comp)?;
        let m = desenho.moldura.clone();
        let mut cena = desenho.cena.clone();
        let mut rotulos = Vec::new();
        let fonte = config.tamanho;
        let preto = aparencia.cor_texto.unwrap_or(Cor::rgb(32, 32, 32));
        if largura < 90.0 || altura < 70.0 {
            return Ok(CenaAnotada { cena, rotulos });
        }

        if m.legado {
            let (base_titulo, base_subtitulo) = Self::bases_do_titulo(altura, fonte);
            if config.mostrar_titulo && !self.titulo.is_empty() {
                let texto = abreviar(&self.titulo, largura * 0.88, fonte * 1.4);
                if !texto.is_empty() {
                    rotulos.push(rotulo(texto, largura * 0.5, base_titulo,
                        fonte * 1.4, preto, AlinhamentoRotulo::Centro));
                }
            }
            if !config.subtitulo.is_empty() {
                let texto = abreviar(&config.subtitulo, largura * 0.88, fonte);
                if !texto.is_empty() {
                    rotulos.push(rotulo(texto, largura * 0.5, base_subtitulo,
                        fonte, preto, AlinhamentoRotulo::Centro));
                }
            }
        } else {
            let c = m.conteudo;
            for (texto, pos, estilo) in [
                (&self.titulo, m.titulo, aparencia.titulo),
                (&config.subtitulo, m.subtitulo, aparencia.subtitulo),
            ] {
                let Some((y, tamanho)) = pos else { continue };
                let estilo = estilo.unwrap_or_default();
                let t = abreviar(texto, c.largura, tamanho);
                if t.is_empty() {
                    continue;
                }
                let (x, al) = match estilo.alinhamento {
                    AlinhamentoTitulo::Esquerda => (c.x, AlinhamentoRotulo::Inicio),
                    AlinhamentoTitulo::Centro => (c.x + c.largura / 2.0, AlinhamentoRotulo::Centro),
                    AlinhamentoTitulo::Direita => (c.x + c.largura, AlinhamentoRotulo::Fim),
                };
                let mut r = rotulo(t, x, y, tamanho, estilo.fonte.cor.unwrap_or(preto), al);
                r.negrito = estilo.fonte.negrito;
                rotulos.push(r);
            }
        }

        if let Some(eixos) = &desenho.eixos {
            let (cv, cc) = (&aparencia.eixos.valores, &aparencia.eixos.categorias);
            let barras = vis.tipo == Tipo::Barras;
            let (cfg_x, cfg_y) = if barras { (cv, cc) } else { (cc, cv) };
            if config.mostrar_eixos {
                let direito = eixos.area.x + eixos.area.largura;
                let inferior = eixos.area.y + eixos.area.altura;
                if cfg_x.mostrar {
                    let rot = cfg_x.rotacao;
                    let limite = if rot == RotacaoRotulos::Graus0 {
                        (eixos.area.largura / eixos.ticks_x.len().max(1) as f32).max(20.0)
                    } else {
                        (altura * 0.3).max(20.0)
                    };
                    for tick in &eixos.ticks_x {
                        let texto = match tick.numero {
                            Some(v) if cfg_x.formato.is_some() => texto_tick(cfg_x, &comp, v),
                            Some(v) => config.formato.formatar(v, config.localidade)?,
                            None => tick.rotulo.clone(),
                        };
                        let texto = abreviar(&texto, limite, fonte);
                        if texto.is_empty() {
                            continue;
                        }
                        let mut r = match rot {
                            RotacaoRotulos::Graus0 => rotulo(texto, tick.posicao, inferior + fonte * 1.35,
                                fonte, preto, AlinhamentoRotulo::Centro),
                            RotacaoRotulos::Graus45 => rotulo(texto, tick.posicao + fonte * 0.3,
                                inferior + fonte * 0.9, fonte, preto, AlinhamentoRotulo::Fim),
                            RotacaoRotulos::Graus90 => rotulo(texto, tick.posicao + fonte * 0.35,
                                inferior + 6.0, fonte, preto, AlinhamentoRotulo::Fim),
                        };
                        r.rotacao = rot.graus();
                        rotulos.push(r);
                    }
                }
                if cfg_y.mostrar {
                    for tick in &eixos.ticks_y {
                        let texto = match tick.numero {
                            Some(v) if cfg_y.formato.is_some() => texto_tick(cfg_y, &comp, v),
                            Some(v) => config.formato.formatar(v, config.localidade)?,
                            None => tick.rotulo.clone(),
                        };
                        let espaco = if m.legado { eixos.area.x - 7.0 } else { eixos.area.x - m.conteudo.x - 2.0 };
                        let texto = abreviar(&texto, espaco.max(0.0), fonte);
                        if !texto.is_empty() {
                            rotulos.push(rotulo(texto, eixos.area.x - 5.0,
                                tick.posicao + fonte * 0.33, fonte, preto,
                                AlinhamentoRotulo::Fim));
                        }
                    }
                }
                if !config.unidade.is_empty() {
                    let unit = abreviar(&config.unidade, eixos.area.largura * 0.5, fonte);
                    let y = if m.legado { (eixos.area.y - 4.0).max(fonte) } else { m.unidade_y };
                    rotulos.push(rotulo(unit, direito, y, fonte, preto, AlinhamentoRotulo::Fim));
                }
            }
            // Títulos dos eixos (só no layout novo, que reserva o espaço).
            let titulo_eixo = |cfg: &crate::ConfigEixo| -> Option<(String, f32, Cor, bool)> {
                let t = cfg.titulo.as_ref().filter(|t| !t.is_empty())?;
                Some((t.clone(), cfg.fonte_titulo.tamanho.unwrap_or(fonte),
                    cfg.fonte_titulo.cor.unwrap_or(preto), cfg.fonte_titulo.negrito))
            };
            if let (Some(y), Some((t, tam, cor, negrito))) = (m.titulo_eixo_h, titulo_eixo(cfg_x)) {
                let t = abreviar(&t, eixos.area.largura, tam);
                let mut r = rotulo(t, eixos.area.x + eixos.area.largura / 2.0, y, tam, cor, AlinhamentoRotulo::Centro);
                r.negrito = negrito;
                rotulos.push(r);
            }
            if let (Some(x), Some((t, tam, cor, negrito))) = (m.titulo_eixo_v, titulo_eixo(cfg_y)) {
                let t = abreviar(&t, eixos.area.altura, tam);
                let mut r = rotulo(t, x, eixos.area.y + eixos.area.altura / 2.0, tam, cor, AlinhamentoRotulo::Centro);
                r.negrito = negrito;
                r.rotacao = 90.0;
                rotulos.push(r);
            }
            if let Some(rd) = &aparencia.rotulos {
                crate::rotulos::rotulos_dados(&vis, &desenho, aparencia, &mapa, rd, fonte, preto, &mut cena, &mut rotulos)?;
            } else if config.mostrar_valores && matches!(vis.tipo, Tipo::Colunas | Tipo::Barras) {
                let barras = vis.retangulos_barras_em(eixos, aparencia)?;
                // Labels densos precisam de layout collision-aware; até 30
                // barras, usa a posição geométrica existente, sem recalcular
                // agregações nem tocar nas séries originais.
                if barras.len() <= 30 {
                    for (serie_idx, categoria_idx, area) in &barras {
                        let valor = vis.series[*serie_idx].valores[*categoria_idx];
                        let texto = config.formato.formatar(valor, config.localidade)?;
                        let (x, y, align) = if vis.tipo == Tipo::Colunas {
                            (area.x + area.largura * 0.5,
                                (area.y - 3.0).max(fonte),
                                AlinhamentoRotulo::Centro)
                        } else {
                            (area.x + area.largura + 3.0,
                                area.y + area.altura * 0.75,
                                AlinhamentoRotulo::Inicio)
                        };
                        rotulos.push(rotulo(texto, x, y, fonte, preto, align));
                    }
                }
            }
        } else if let Some(rd) = aparencia.rotulos.clone().or_else(|| {
            (!m.legado || aparencia.pizza.is_some()).then(crate::rotulos::padrao_pizza)
        }) {
            crate::rotulos::rotulos_dados(&vis, &desenho, aparencia, &mapa, &rd, fonte, preto, &mut cena, &mut rotulos)?;
        } else {
            // Pizza: usar as mesmas caixas calculadas por CHT-010 e usadas
            // pelos backends SVG/PDF legados. A ausência de espaço oculta texto.
            for fatia in vis.fatias_pizza(largura, altura)? {
                let Some(r) = fatia.rotulo else { continue };
                let tamanho = fonte.min(r.area.altura)
                    .min(r.area.largura / r.texto.chars().count().max(1) as f32);
                if tamanho > 0.0 && !r.texto.is_empty() {
                    rotulos.push(rotulo(r.texto, r.area.x,
                        r.area.y + (r.area.altura + tamanho) * 0.5,
                        tamanho, preto, AlinhamentoRotulo::Inicio));
                }
            }
        }
        if config.mostrar_legenda {
            let fundo = aparencia.fundo.unwrap_or(Cor::rgb(255, 255, 255));
            let fonte_legenda = aparencia.legenda.map(|l| l.fonte).unwrap_or_default();
            let cor_legenda = fonte_legenda.cor.unwrap_or(preto);
            for item in self.itens_legenda_impl(largura, altura, config, aparencia, filtro, pizza_por_categoria, &m) {
                let (cor_marca, cor_texto) = if item.escondido {
                    (esmaecer(item.cor, fundo), esmaecer(cor_legenda, fundo))
                } else {
                    (item.cor, cor_legenda)
                };
                cena.adicionar(Figura::Retangulo {
                    area: item.marca,
                    estilo: Estilo {
                        preenchimento: Some(cor_marca),
                        contorno: None,
                        espessura: 0.0,
                    },
                });
                let tamanho = if m.legado { fonte } else { fonte_legenda.tamanho.unwrap_or(fonte) };
                let mut r = rotulo(item.texto, item.x_texto, item.y_texto, tamanho, cor_texto,
                    AlinhamentoRotulo::Inicio);
                r.negrito = fonte_legenda.negrito;
                rotulos.push(r);
            }
        }
        Ok(CenaAnotada { cena, rotulos })
    }

    /// Itens da legenda com posição e área clicável. Itens escondidos pelo
    /// filtro continuam presentes (`escondido = true`). Pizza lista as categorias.
    pub fn itens_legenda(
        &self,
        largura: f32,
        altura: f32,
        config: &TipografiaGrafico,
        aparencia: &AparenciaGrafico,
        filtro: &FiltroLegenda,
    ) -> Vec<ItemLegenda> {
        if !config.mostrar_legenda || largura < 90.0 || altura < 70.0 {
            return Vec::new();
        }
        let comp = Composicao::da_tipografia(self, config, true);
        let Ok(m) = self
            .filtrado(filtro)
            .and_then(|(vis, _)| vis.moldura(largura, altura, &self.aparencia_anotada(largura, altura, config, aparencia), &comp))
        else {
            return Vec::new();
        };
        self.itens_legenda_impl(largura, altura, config, aparencia, filtro, true, &m)
    }

    /// Qual item da legenda está sob o ponto (índice da série, ou da categoria
    /// na pizza), se algum.
    pub fn hit_test_legenda(
        &self,
        largura: f32,
        altura: f32,
        config: &TipografiaGrafico,
        aparencia: &AparenciaGrafico,
        filtro: &FiltroLegenda,
        ponto: Ponto,
    ) -> Option<usize> {
        self.itens_legenda(largura, altura, config, aparencia, filtro)
            .into_iter()
            .find(|i| i.caixa.contem(ponto))
            .map(|i| i.indice)
    }

    #[allow(clippy::too_many_arguments)]
    fn itens_legenda_impl(
        &self,
        largura: f32,
        altura: f32,
        config: &TipografiaGrafico,
        aparencia: &AparenciaGrafico,
        filtro: &FiltroLegenda,
        pizza_por_categoria: bool,
        m: &Moldura,
    ) -> Vec<ItemLegenda> {
        let entradas: Vec<(String, Cor)> = if self.tipo == Tipo::Pizza && pizza_por_categoria {
            let v = &self.series[0].valores;
            let (lo, hi) = v.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), x| (a.min(*x), b.max(*x)));
            self.categorias.iter().enumerate()
                .map(|(i, c)| (c.clone(), aparencia.cor_fatia(i, v[i], lo, hi)))
                .collect()
        } else {
            self.series.iter().enumerate()
                .map(|(i, s)| (s.nome.clone(), aparencia.cor_serie(i, s.cor)))
                .collect()
        };
        let item = |indice: usize, nome: String, cor: Cor, texto: String, x: f32, y: f32, fonte: f32, espaco: f32| ItemLegenda {
            indice,
            nome,
            cor,
            escondido: filtro.esta_oculto(indice),
            marca: Retangulo::novo(x, y - fonte * 0.75, fonte * 0.75, fonte * 0.75),
            caixa: Retangulo::novo(x, y - fonte * 0.9, espaco, fonte * 1.3),
            texto,
            x_texto: x + fonte * 1.0,
            y_texto: y,
        };
        let mut itens = Vec::new();
        if !m.legado {
            let Some((posicao, area, fonte)) = m.legenda else { return itens };
            let textos: Vec<String> = entradas.iter().map(|(n, _)| {
                let limite = if matches!(posicao, PosicaoLegenda::Esquerda | PosicaoLegenda::Direita) {
                    area.largura - fonte * 2.3
                } else {
                    area.largura * 0.45
                };
                abreviar(n, limite, fonte)
            }).collect();
            if matches!(posicao, PosicaoLegenda::Esquerda | PosicaoLegenda::Direita) {
                let passo = fonte * 1.5;
                let total = passo * textos.iter().filter(|t| !t.is_empty()).count() as f32;
                let mut y = (m.plot.y + m.plot.altura / 2.0 - total / 2.0).max(area.y) + fonte;
                for (indice, ((nome, cor), texto)) in entradas.into_iter().zip(textos).enumerate() {
                    if texto.is_empty() { continue; }
                    if y > area.y + area.altura { break; }
                    let espaco = largura_item_legenda(&texto, fonte);
                    itens.push(item(indice, nome, cor, texto, area.x, y, fonte, espaco));
                    y += passo;
                }
            } else {
                let linhas = quebrar_legenda(&textos, fonte, area.largura);
                let mut entradas: Vec<Option<(String, Cor)>> = entradas.into_iter().map(Some).collect();
                for (k, linha) in linhas.iter().enumerate() {
                    let total: f32 = linha.iter().map(|i| largura_item_legenda(&textos[*i], fonte)).sum();
                    let mut x = area.x + ((area.largura - total) / 2.0).max(0.0);
                    let y = area.y + fonte * 1.05 + k as f32 * fonte * 1.4;
                    for &i in linha {
                        let espaco = largura_item_legenda(&textos[i], fonte);
                        if textos[i].is_empty() { continue; }
                        let (nome, cor) = entradas[i].take().expect("item único");
                        itens.push(item(i, nome, cor, textos[i].clone(), x, y, fonte, espaco));
                        x += espaco;
                    }
                }
            }
            return itens;
        }
        let fonte = config.tamanho;
        let mut x = largura * 0.12;
        // Abaixo dos rótulos do eixo X (`layout_anotado` deixou o espaço).
        let base_eixo = if config.mostrar_eixos && self.tipo != Tipo::Pizza { m.plot.y + m.plot.altura + fonte * 1.35 } else { 0.0 };
        let mut y = (altura * 0.925).max(base_eixo + fonte * 1.3).min(altura - fonte * 0.3);
        for (indice, (nome, cor)) in entradas.into_iter().enumerate() {
            let texto = abreviar(&nome, largura * 0.45, fonte);
            if texto.is_empty() { continue; }
            let espaco = fonte * 0.60 * texto.chars().count() as f32 + fonte * 2.3;
            if x + espaco > largura * 0.96 {
                x = largura * 0.12;
                y += fonte * 1.4;
            }
            if y + fonte * 0.4 > altura { break; }
            itens.push(item(indice, nome, cor, texto, x, y, fonte, espaco));
            x += espaco;
        }
        itens
    }

    /// Exportação opt-in: exportar_svg() legado permanece idêntico.
    pub fn exportar_svg_anotado(
        &self,
        largura: f32,
        altura: f32,
        config: &TipografiaGrafico,
    ) -> Result<String, ErroExportacaoTipografica> {
        let layout = self.cena_anotada(largura, altura, config)
            .map_err(ErroExportacaoTipografica::Grafico)?;
        svg_da_cena_anotada(&layout)
    }

    /// SVG com aparência (fundo, temática, efeitos) e filtro da legenda.
    pub fn exportar_svg_estilizado(
        &self,
        largura: f32,
        altura: f32,
        config: &TipografiaGrafico,
        aparencia: &AparenciaGrafico,
        filtro: &FiltroLegenda,
    ) -> Result<String, ErroExportacaoTipografica> {
        let layout = self.cena_anotada_estilizada(largura, altura, config, aparencia, filtro)
            .map_err(ErroExportacaoTipografica::Grafico)?;
        svg_da_cena_anotada(&layout)
    }

    /// Exportação opt-in PDF, reutilizando a mesma Cena e posições do SVG.
    /// Caractere não mapeável em Helvetica/WinAnsi é erro explícito.
    pub fn exportar_pdf_anotado(
        &self,
        largura: f32,
        altura: f32,
        config: &TipografiaGrafico,
    ) -> Result<Vec<u8>, ErroExportacaoTipografica> {
        let layout = self.cena_anotada(largura, altura, config)
            .map_err(ErroExportacaoTipografica::Grafico)?;
        pdf_da_cena_anotada(&layout)
    }

    /// PDF com aparência e filtro da legenda.
    pub fn exportar_pdf_estilizado(
        &self,
        largura: f32,
        altura: f32,
        config: &TipografiaGrafico,
        aparencia: &AparenciaGrafico,
        filtro: &FiltroLegenda,
    ) -> Result<Vec<u8>, ErroExportacaoTipografica> {
        // O PDF não aceita gradiente com opacidade variável: usa faixas.
        let mut ap = aparencia.clone();
        if let Some(g) = &mut ap.gradiente_area {
            if g.faixas == 0 {
                g.faixas = 32;
            }
        }
        let layout = self.cena_anotada_estilizada(largura, altura, config, &ap, filtro)
            .map_err(ErroExportacaoTipografica::Grafico)?;
        pdf_da_cena_anotada(&layout)
    }
}

fn svg_da_cena_anotada(layout: &CenaAnotada) -> Result<String, ErroExportacaoTipografica> {
    let mut svg = exportar_svg(&layout.cena)
        .map_err(ErroExportacaoTipografica::Svg)?;
    let mut overlays = String::new();
    for t in &layout.rotulos {
        let ancora = match t.alinhamento {
            AlinhamentoRotulo::Inicio => "start",
            AlinhamentoRotulo::Centro => "middle",
            AlinhamentoRotulo::Fim => "end",
        };
        let mut extra = String::new();
        if t.negrito {
            extra.push_str(" font-weight=\"bold\"");
        }
        if t.rotacao != 0.0 {
            write!(&mut extra, " transform=\"rotate({:.3} {:.3} {:.3})\"", -t.rotacao, t.x, t.y).unwrap();
        }
        if t.cor.a != 255 {
            write!(&mut extra, " fill-opacity=\"{:.3}\"", f32::from(t.cor.a) / 255.0).unwrap();
        }
        writeln!(&mut overlays,
            "<text x=\"{:.3}\" y=\"{:.3}\" font-family=\"sans-serif\" font-size=\"{:.3}\" text-anchor=\"{}\" fill=\"#{:02x}{:02x}{:02x}\"{}>{}</text>",
            t.x, t.y, t.tamanho, ancora, t.cor.r, t.cor.g, t.cor.b, extra,
            escapar_svg(&t.texto)).unwrap();
    }
    if let Some(pos) = svg.rfind("</svg>") {
        svg.insert_str(pos, &overlays);
    }
    Ok(svg)
}

fn pdf_da_cena_anotada(layout: &CenaAnotada) -> Result<Vec<u8>, ErroExportacaoTipografica> {
    let textos = layout.rotulos.iter().map(|t| {
        let extensao = t.texto.chars().count() as f32 * t.tamanho * 0.55;
        let x = match t.alinhamento {
            AlinhamentoRotulo::Inicio => t.x,
            AlinhamentoRotulo::Centro => t.x - extensao * 0.5,
            AlinhamentoRotulo::Fim => t.x - extensao,
        };
        TextoPdf {
            texto: t.texto.clone(),
            x: x.max(0.0),
            y: t.y,
            tamanho: t.tamanho,
            cor: t.cor,
            fonte: if t.negrito { FontePdf::Negrito } else { FontePdf::Helvetica },
        }
    }).collect::<Vec<_>>();
    exportar_pdf_com_texto(&layout.cena, &textos)
        .map_err(ErroExportacaoTipografica::Pdf)
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::Serie;

    fn exemplo(tipo: Tipo) -> Grafico {
        Grafico {
            titulo: "Produção & energia".into(),
            tipo,
            categorias: vec!["Jan".into(), "Fev".into()],
            series: vec![
                Serie { nome: "Vendas".into(), valores: vec![-1234.5, 2345.5], cor: Cor::rgb(230, 10, 20) },
                Serie { nome: "Custos".into(), valores: vec![200.0, 500.0], cor: Cor::rgb(15, 20, 210) },
            ],
        }
    }

    #[test]
    fn legenda_nao_pisa_nos_rotulos_do_eixo_nem_no_titulo() {
        let g = Grafico {
            titulo: "ΔV (%) por circuito × limite".into(),
            tipo: Tipo::Colunas,
            categorias: vec!["C1".into(), "C2".into()],
            series: vec![
                Serie { nome: "ΔV".into(), valores: vec![2.0, 3.0], cor: Cor::rgb(31, 58, 95) },
                Serie { nome: "Limite".into(), valores: vec![4.0, 4.0], cor: Cor::rgb(198, 40, 40) },
            ],
        };
        for (w, h, fonte) in [(800.0, 450.0, 12.0), (620.0, 180.0, 12.0), (480.0, 300.0, 14.0)] {
            let cfg = TipografiaGrafico { tamanho: fonte, ..Default::default() };
            let a = g.cena_anotada(w, h, &cfg).unwrap();
            let eixos = g.eixos_com_layout(w, h, &g.layout_anotado(w, h, &cfg)).unwrap().unwrap();
            let base_x = eixos.area.y + eixos.area.altura + fonte * 1.35;
            let titulo_y = a.rotulos.iter().find(|r| r.texto.starts_with("ΔV (%)")).unwrap().y;
            assert!(titulo_y - fonte * 1.4 * 0.8 >= 0.0, "{w}x{h}: título cortado em cima");
            assert!(titulo_y < eixos.area.y, "{w}x{h}: título dentro da área");
            for r in a.rotulos.iter().filter(|r| r.texto == "Limite" || r.texto == "ΔV") {
                assert!(r.y >= base_x + fonte, "{w}x{h}: legenda em {} pisa no eixo ({base_x})", r.y);
                assert!(r.y <= h, "{w}x{h}: legenda fora");
            }
        }
    }

    #[test]
    fn formata_pt_br_en_us_com_grupos_percentuais_e_negativos() {
        assert_eq!(FormatoNumero::Decimal(2).formatar(-1234567.5, LocalidadeGrafico::PtBr).unwrap(), "-1.234.567,50");
        assert_eq!(FormatoNumero::Decimal(2).formatar(-1234567.5, LocalidadeGrafico::EnUs).unwrap(), "-1,234,567.50");
        assert_eq!(FormatoNumero::Moeda(2).formatar(1234.5, LocalidadeGrafico::PtBr).unwrap(), "R$ 1.234,50");
        assert_eq!(FormatoNumero::Moeda(2).formatar(1234.5, LocalidadeGrafico::EnUs).unwrap(), "$1,234.50");
        assert_eq!(FormatoNumero::Percentual(1).formatar(0.125, LocalidadeGrafico::PtBr).unwrap(), "12,5%");
        assert_eq!(FormatoNumero::Decimal(0).formatar(1000000.0, LocalidadeGrafico::EnUs).unwrap(), "1,000,000");
        assert_eq!(FormatoNumero::Decimal(7).formatar(1.0, LocalidadeGrafico::PtBr), Err(ErroGrafico::ValorInvalido));
        assert_eq!(FormatoNumero::Decimal(2).formatar(f64::NAN, LocalidadeGrafico::PtBr), Err(ErroGrafico::ValorInvalido));
    }

    #[test]
    fn metadados_geram_titulo_subtitulo_legenda_e_eixos() {
        let g = exemplo(Tipo::Colunas);
        let cfg = TipografiaGrafico {
            subtitulo: "Comparativo 2026".into(),
            unidade: "MWh".into(),
            mostrar_valores: true,
            ..Default::default()
        };
        let layout = g.cena_anotada(800.0, 500.0, &cfg).unwrap();
        assert!(layout.rotulos.iter().any(|r| r.texto == "Produção & energia"));
        assert!(layout.rotulos.iter().any(|r| r.texto == "Comparativo 2026"));
        assert!(layout.rotulos.iter().any(|r| r.texto == "MWh"));
        assert!(layout.rotulos.iter().any(|r| r.texto == "Vendas"));
        assert!(layout.rotulos.iter().any(|r| r.texto == "Custos"));
        assert!(layout.rotulos.iter().any(|r| r.texto == "-1.234,5"));
        assert!(layout.cena.figuras.len() > g.cena(800.0, 500.0).unwrap().figuras.len());
        assert_eq!(g.series[0].valores[0], -1234.5);
    }

    #[test]
    fn SVG_e_PDF_geram_mesmos_textos_e_cores_sem_mudar_export_legado() {
        let g = exemplo(Tipo::Barras);
        let cfg = TipografiaGrafico {
            subtitulo: "Olá <mundo> & relatório".into(),
            ..Default::default()
        };
        let svg = g.exportar_svg_anotado(900.0, 520.0, &cfg).unwrap();
        let pdf = g.exportar_pdf_anotado(900.0, 520.0, &cfg).unwrap();
        let texto = String::from_utf8_lossy(&pdf);
        assert!(svg.contains("Produção &amp; energia"));
        assert!(svg.contains("Olá &lt;mundo&gt; &amp; relatório"));
        assert!(svg.contains("text-anchor=\"middle\""));
        assert!(svg.contains("fill=\"#e60a14\"")); // marcador de Vendas
        assert!(texto.contains("BT /F1"));
        assert!(texto.contains("/MediaBox [0 0 900 520]"));
        assert_eq!(pdf, g.exportar_pdf_anotado(900.0, 520.0, &cfg).unwrap());
        assert_eq!(svg, g.exportar_svg_anotado(900.0, 520.0, &cfg).unwrap());
        let antigo = g.exportar_svg(900.0, 520.0).unwrap();
        assert!(!antigo.contains("Produção &amp; energia"));
    }

    #[test]
    fn erro_unicode_pdf_sem_perda_silenciosa() {
        let mut g = exemplo(Tipo::Pizza);
        g.series.truncate(1);
        g.series[0].valores = vec![12.0, 17.0];
        g.titulo = "東京".into();
        let err = g.exportar_pdf_anotado(800.0, 500.0, &TipografiaGrafico::default()).unwrap_err();
        assert_eq!(err, ErroExportacaoTipografica::Pdf(ErroPdf::CaractereNaoSuportado));
        let svg = g.exportar_svg_anotado(800.0, 500.0, &TipografiaGrafico::default()).unwrap();
        assert!(svg.contains("東京"));
    }

    #[test]
    fn pizza_rende_rotulos_e_layout_pequeno_suprime_overlay() {
        let mut g = exemplo(Tipo::Pizza);
        g.series.truncate(1);
        g.series[0].valores = vec![12.0, 17.0];
        let cfg = TipografiaGrafico::default();
        let layout = g.cena_anotada(800.0, 500.0, &cfg).unwrap();
        assert!(layout.rotulos.iter().any(|r| r.texto == "Jan" || r.texto == "Fev"));
        let pequeno = g.cena_anotada(40.0, 30.0, &cfg).unwrap();
        assert!(pequeno.rotulos.is_empty());
        assert!(pequeno.cena.figuras.len() > 0);
    }

    #[test]
    fn valida_configuracao_e_dados_invalidos() {
        let mut g = exemplo(Tipo::Colunas);
        let c = TipografiaGrafico { tamanho: f32::NAN, ..Default::default() };
        assert_eq!(g.cena_anotada(800.0, 400.0, &c), Err(ErroGrafico::ValorInvalido));
        g.series[0].valores[0] = f64::INFINITY;
        assert_eq!(g.cena_anotada(800.0, 400.0, &TipografiaGrafico::default()), Err(ErroGrafico::ValorInvalido));
    }
}
