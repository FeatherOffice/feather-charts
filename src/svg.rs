//! Exportação de gráficos com tipografia SVG confinada às caixas de rótulo.
use crate::{ErroGrafico, Grafico, Tipo};
use feather_draw::{exportar_svg, ErroSvg};
use std::fmt::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErroSvgGrafico {
    Grafico(ErroGrafico),
    Svg(ErroSvg),
}

fn escapar(texto: &str) -> String {
    texto
        .chars()
        .filter(|c| !c.is_control() || matches!(c, '\t' | '\n' | '\r'))
        .collect::<String>()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

impl Grafico {
    /// Exporta as formas e, para pizza, os rótulos efetivamente renderizados.
    /// Texto usa 11 px e `textLength` limitado à caixa; clipping impede que
    /// métricas de fontes diferentes invadam outras caixas ou a pizza.
    /// A API `cena` permanece geométrica, sem alterar consumidores existentes.
    pub fn exportar_svg(&self, largura: f32, altura: f32) -> Result<String, ErroSvgGrafico> {
        let cena = self
            .cena(largura, altura)
            .map_err(ErroSvgGrafico::Grafico)?;
        let mut svg = exportar_svg(&cena).map_err(ErroSvgGrafico::Svg)?;
        if self.tipo != Tipo::Pizza {
            return Ok(svg);
        }
        let fatias = self
            .fatias_pizza(largura, altura)
            .map_err(ErroSvgGrafico::Grafico)?;
        let mut rotulos = String::new();
        for fatia in fatias {
            let Some(r) = fatia.rotulo else { continue };
            if r.texto.is_empty() || r.area.largura <= 0.0 {
                continue;
            }
            let a = r.area;
            writeln!(rotulos,
                "<defs><clipPath id=\"rotulo{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" /></clipPath></defs>",
                fatia.indice, a.x, a.y, a.largura, a.altura).unwrap();
            writeln!(rotulos,
                "<text x=\"{}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"11\" fill=\"#202020\" textLength=\"{}\" lengthAdjust=\"spacingAndGlyphs\" clip-path=\"url(#rotulo{})\"><title>{}</title>{}</text>",
                a.x, a.y + 11.0, a.largura, fatia.indice, escapar(&fatia.categoria), escapar(&r.texto)).unwrap();
        }
        let fim = svg
            .rfind("</svg>")
            .expect("exportador Drawing produz SVG completo");
        svg.insert_str(fim, &rotulos);
        Ok(svg)
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::Serie;
    use feather_draw::Cor;

    fn pizza(categorias: &[&str]) -> Grafico {
        Grafico {
            titulo: String::new(),
            tipo: Tipo::Pizza,
            categorias: categorias.iter().map(|x| (*x).into()).collect(),
            series: vec![Serie {
                nome: String::new(),
                valores: vec![1.; categorias.len()],
                cor: Cor::rgb(0, 0, 255),
            }],
        }
    }

    #[test]
    fn renderiza_textos_deterministicos_escapados_e_confinados() {
        let g = pizza(&["A<&", "WIDE"]);
        let svg = g.exportar_svg(640., 320.).unwrap();
        assert_eq!(svg.matches("<text ").count(), 2);
        assert!(svg.contains("A&lt;&amp;"));
        assert!(!svg.contains("A<&"));
        assert_eq!(svg.matches("lengthAdjust=\"spacingAndGlyphs\"").count(), 2);
        assert_eq!(svg.matches("clip-path=\"url(#rotulo").count(), 2);
        assert_eq!(svg, g.exportar_svg(640., 320.).unwrap());
    }

    #[test]
    fn nao_perde_fatias_quando_texto_nao_cabe() {
        let svg = pizza(&["A", "B"]).exportar_svg(30., 20.).unwrap();
        assert_eq!(svg.matches("<text ").count(), 0);
        assert_eq!(svg.matches("<path ").count(), 2);
    }

    #[test]
    fn preserva_svg_dos_outros_tipos_e_rejeita_invalidos() {
        let mut g = pizza(&["A", "B"]);
        g.tipo = Tipo::Colunas;
        assert_eq!(
            g.exportar_svg(640., 320.).unwrap(),
            exportar_svg(&g.cena(640., 320.).unwrap()).unwrap()
        );
        g.tipo = Tipo::Pizza;
        g.series[0].valores[0] = -1.;
        assert_eq!(
            g.exportar_svg(640., 320.),
            Err(ErroSvgGrafico::Grafico(ErroGrafico::ValorInvalido))
        );
    }
}
