//! Tipografia e formatação regional compartilhadas para gráficos.
//!
//! A geometria continua em Grafico::cena e o texto é um overlay declarativo,
//! independente da GUI. Exportadores SVG/PDF usam o mesmo layout e conteúdo.

use crate::{ErroGrafico, Grafico, Tipo};
use feather_draw::{
    exportar_pdf_com_texto, exportar_svg, Cena, Cor, ErroPdf, ErroSvg,
    Estilo, Figura, FontePdf, Retangulo, TextoPdf,
};
use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalidadeGrafico {
    PtBr,
    EnUs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

/// Anotações separadas das formas, para serem consumidas por Iced ou PDF/SVG.
#[derive(Debug, Clone, PartialEq)]
pub struct CenaAnotada {
    pub cena: Cena,
    pub rotulos: Vec<RotuloGrafico>,
}

#[derive(Debug, Clone, PartialEq)]
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
    RotuloGrafico { texto, x, y, tamanho, cor, alinhamento }
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
    /// Composição tipográfica sem mudar Grafico/Serie ou persistência nativa.
    /// Os eixos conservam as posições retornadas pelo motor CHT-006.
    pub fn cena_anotada(
        &self,
        largura: f32,
        altura: f32,
        config: &TipografiaGrafico,
    ) -> Result<CenaAnotada, ErroGrafico> {
        config.validar()?;
        let mut cena = self.cena(largura, altura)?;
        let mut rotulos = Vec::new();
        let fonte = config.tamanho;
        let preto = Cor::rgb(32, 32, 32);
        if largura < 90.0 || altura < 70.0 {
            return Ok(CenaAnotada { cena, rotulos });
        }

        if config.mostrar_titulo && !self.titulo.is_empty() {
            let texto = abreviar(&self.titulo, largura * 0.88, fonte * 1.4);
            if !texto.is_empty() {
                rotulos.push(rotulo(texto, largura * 0.5, altura * 0.048,
                    fonte * 1.4, preto, AlinhamentoRotulo::Centro));
            }
        }
        if !config.subtitulo.is_empty() {
            let texto = abreviar(&config.subtitulo, largura * 0.88, fonte);
            if !texto.is_empty() {
                rotulos.push(rotulo(texto, largura * 0.5, altura * 0.087,
                    fonte, preto, AlinhamentoRotulo::Centro));
            }
        }

        if let Some(eixos) = self.eixos(largura, altura)? {
            if config.mostrar_eixos {
                let direito = eixos.area.x + eixos.area.largura;
                let inferior = eixos.area.y + eixos.area.altura;
                for tick in &eixos.ticks_x {
                    let texto = match tick.numero {
                        Some(v) => config.formato.formatar(v, config.localidade)?,
                        None => tick.rotulo.clone(),
                    };
                    let texto = abreviar(&texto, (eixos.area.largura / eixos.ticks_x.len().max(1) as f32).max(20.0), fonte);
                    if !texto.is_empty() {
                        rotulos.push(rotulo(texto, tick.posicao, inferior + fonte * 1.35,
                            fonte, preto, AlinhamentoRotulo::Centro));
                    }
                }
                for tick in &eixos.ticks_y {
                    let texto = match tick.numero {
                        Some(v) => config.formato.formatar(v, config.localidade)?,
                        None => tick.rotulo.clone(),
                    };
                    let texto = abreviar(&texto, (eixos.area.x - 7.0).max(0.0), fonte);
                    if !texto.is_empty() {
                        rotulos.push(rotulo(texto, eixos.area.x - 5.0,
                            tick.posicao + fonte * 0.33, fonte, preto,
                            AlinhamentoRotulo::Fim));
                    }
                }
                if !config.unidade.is_empty() {
                    let unit = abreviar(&config.unidade, eixos.area.largura * 0.5, fonte);
                    rotulos.push(rotulo(unit, direito, (eixos.area.y - 4.0).max(fonte),
                        fonte, preto, AlinhamentoRotulo::Fim));
                }
            }
            if config.mostrar_valores && matches!(self.tipo, Tipo::Colunas | Tipo::Barras) {
                let barras: Vec<_> = cena.figuras.iter().filter_map(|f| {
                    if let Figura::Retangulo { area, .. } = f { Some(*area) } else { None }
                }).collect();
                // Labels densos precisam de layout collision-aware; até 30
                // barras, usa a posição geométrica existente, sem recalcular
                // agregações nem tocar nas séries originais.
                if barras.len() <= 30 {
                    for (i, area) in barras.iter().enumerate() {
                        let serie_idx = i / self.categorias.len();
                        let categoria_idx = i % self.categorias.len();
                        if let Some(serie) = self.series.get(serie_idx) {
                            if let Some(&valor) = serie.valores.get(categoria_idx) {
                                let texto = config.formato.formatar(valor, config.localidade)?;
                                let (x, y, align) = if self.tipo == Tipo::Colunas {
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
                }
            }
        } else {
            // Pizza: usar as mesmas caixas calculadas por CHT-010 e usadas
            // pelos backends SVG/PDF legados. A ausência de espaço oculta texto.
            for fatia in self.fatias_pizza(largura, altura)? {
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
            let mut x = largura * 0.12;
            let mut y = altura * 0.925;
            for serie in &self.series {
                let texto = abreviar(&serie.nome, largura * 0.45, fonte);
                if texto.is_empty() { continue; }
                let espaco = fonte * 0.60 * texto.chars().count() as f32 + fonte * 2.3;
                if x + espaco > largura * 0.96 {
                    x = largura * 0.12;
                    y += fonte * 1.4;
                }
                if y + fonte * 0.4 > altura { break; }
                cena.adicionar(Figura::Retangulo {
                    area: Retangulo::novo(x, y - fonte * 0.75, fonte * 0.75, fonte * 0.75),
                    estilo: Estilo {
                        preenchimento: Some(serie.cor),
                        contorno: None,
                        espessura: 0.0,
                    },
                });
                rotulos.push(rotulo(texto, x + fonte * 1.0, y, fonte, preto,
                    AlinhamentoRotulo::Inicio));
                x += espaco;
            }
        }
        Ok(CenaAnotada { cena, rotulos })
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
        let mut svg = exportar_svg(&layout.cena)
            .map_err(ErroExportacaoTipografica::Svg)?;
        let mut overlays = String::new();
        for t in &layout.rotulos {
            let ancora = match t.alinhamento {
                AlinhamentoRotulo::Inicio => "start",
                AlinhamentoRotulo::Centro => "middle",
                AlinhamentoRotulo::Fim => "end",
            };
            writeln!(&mut overlays,
                "<text x=\"{:.3}\" y=\"{:.3}\" font-family=\"sans-serif\" font-size=\"{:.3}\" text-anchor=\"{}\" fill=\"#{:02x}{:02x}{:02x}\">{}</text>",
                t.x, t.y, t.tamanho, ancora, t.cor.r, t.cor.g, t.cor.b,
                escapar_svg(&t.texto)).unwrap();
        }
        if let Some(pos) = svg.rfind("</svg>") {
            svg.insert_str(pos, &overlays);
        }
        Ok(svg)
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
                fonte: FontePdf::Helvetica,
            }
        }).collect::<Vec<_>>();
        exportar_pdf_com_texto(&layout.cena, &textos)
            .map_err(ErroExportacaoTipografica::Pdf)
    }
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
