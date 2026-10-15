//! Exportação de gráficos pelo mesmo modelo vetorial usado no SVG.
//!
//! `Grafico::cena` permanece a fonte de verdade das formas; o backend
//! PDF de Drawing faz a serialização sem rasterização ou dependência de Iced.

use crate::{ErroGrafico, Grafico, Tipo};
use feather_draw::{
    exportar_pdf_com_texto, Cor, ErroPdf, FontePdf, TextoPdf,
};

/// Falhas preservadas do gerador de Cena e do backend PDF.
/// Caracteres fora de WinAnsi não são substituídos silenciosamente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErroPdfGrafico {
    Grafico(ErroGrafico),
    Pdf(ErroPdf),
}

impl Grafico {
    /// Exporta as mesmas formas de `cena()` em PDF 1.4 vetorial.
    ///
    /// Os rótulos geométricos de pizza são convertidos em texto posicionável.
    /// Outras legendas, unidades e títulos pertencem a CHT-007 e não são
    /// inseridos artificialmente aqui. O layout usa coordenadas de página
    /// em pontos, de acordo com o backend do Drawing.
    pub fn exportar_pdf(&self, largura: f32, altura: f32) -> Result<Vec<u8>, ErroPdfGrafico> {
        let cena = self
            .cena(largura, altura)
            .map_err(ErroPdfGrafico::Grafico)?;

        if self.tipo != Tipo::Pizza {
            return exportar_pdf_com_texto(&cena, &[]).map_err(ErroPdfGrafico::Pdf);
        }

        let fatias = self
            .fatias_pizza(largura, altura)
            .map_err(ErroPdfGrafico::Grafico)?;
        let mut textos = Vec::with_capacity(fatias.len());
        for fatia in fatias {
            let Some(rotulo) = fatia.rotulo else {
                continue;
            };
            if rotulo.texto.is_empty()
                || rotulo.area.largura <= 0.0
                || rotulo.area.altura <= 0.0
            {
                continue;
            }

            // Helvetica tem largura de glifo variável. Um limite conservador
            // de um em por caractere impede que o rótulo invada outra caixa
            // mesmo sem suporte a textLength/clipping tipográfico no PDF.
            let caracteres = rotulo.texto.chars().count() as f32;
            let tamanho = 11.0_f32
                .min(rotulo.area.altura)
                .min(rotulo.area.largura / caracteres);
            if !tamanho.is_finite() || tamanho <= 0.0 {
                continue;
            }
            textos.push(TextoPdf {
                texto: rotulo.texto,
                x: rotulo.area.x,
                y: rotulo.area.y + (rotulo.area.altura + tamanho) * 0.5,
                tamanho,
                cor: Cor::rgb(32, 32, 32),
                fonte: FontePdf::Helvetica,
            });
        }
        exportar_pdf_com_texto(&cena, &textos).map_err(ErroPdfGrafico::Pdf)
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::{Serie, Tipo};
    use feather_draw::{exportar_pdf, exportar_svg, Cor, Figura};

    fn grafico(tipo: Tipo) -> Grafico {
        Grafico {
            titulo: "Receita".into(),
            tipo,
            categorias: vec!["10".into(), "20".into(), "30".into()],
            series: vec![Serie {
                nome: "Série".into(),
                valores: vec![10.0, 20.0, 30.0],
                cor: Cor::rgb(11, 87, 208),
            }],
        }
    }

    fn pdf_comandos(pdf: &[u8]) -> String {
        String::from_utf8_lossy(pdf).into_owned()
    }

    #[test]
    fn cinco_tipos_usam_mesma_cena_para_svg_e_pdf() {
        for tipo in [
            Tipo::Colunas,
            Tipo::Barras,
            Tipo::Linhas,
            Tipo::Pizza,
            Tipo::Dispersao,
        ] {
            let grafico = grafico(tipo);
            let cena = grafico.cena(600.0, 400.0).unwrap();
            let svg = grafico.exportar_svg(600.0, 400.0).unwrap();
            let pdf = grafico.exportar_pdf(600.0, 400.0).unwrap();
            let texto_pdf = pdf_comandos(&pdf);

            assert_eq!(svg.matches("<path d=").count(), cena.figuras.len());
            assert!(texto_pdf.starts_with("%PDF-1.4\n"));
            assert!(texto_pdf.contains("/MediaBox [0 0 600 400]"));
            assert!(texto_pdf.contains("1 0 0 -1 0 400 cm"));
            assert!(texto_pdf.contains("/Type /Page "));
            assert!(texto_pdf.ends_with("%%EOF\n"));
            assert_eq!(pdf, grafico.exportar_pdf(600.0, 400.0).unwrap());
            assert_eq!(svg, grafico.exportar_svg(600.0, 400.0).unwrap());
            assert_eq!(cena, grafico.cena(600.0, 400.0).unwrap());
        }
    }

    #[test]
    fn snapshot_geometrico_drawing_equivalente_para_pdf_e_svg() {
        // Mesmo contrato geométrico sem serialização duplicada no Charts.
        let grafico = grafico(Tipo::Colunas);
        let cena = grafico.cena(600.0, 400.0).unwrap();
        let svg = grafico.exportar_svg(600.0, 400.0).unwrap();
        let pdf = grafico.exportar_pdf(600.0, 400.0).unwrap();
        assert_eq!(svg, exportar_svg(&cena).unwrap());
        assert_eq!(pdf, exportar_pdf(&cena).unwrap());
        let num_barras = cena.figuras.iter().filter(|figura| {
            matches!(figura, Figura::Retangulo { .. })
        }).count();
        assert_eq!(num_barras, 3);

        // Snapshot parcial de dimensões e posicionamento, independente da
        // numeração interna dos objetos PDF.
        let texto = pdf_comandos(&pdf);
        assert!(svg.contains("viewBox=\"0 0 600 400\""));
        assert!(texto.contains("/MediaBox [0 0 600 400]"));
        assert!(texto.contains("1 0 0 -1 0 400 cm"));
        let numero_pdf = |v: f32| {
            let mut texto = format!("{v:.4}");
            while texto.ends_with('0') && texto.contains('.') {
                texto.pop();
            }
            if texto.ends_with('.') {
                texto.pop();
            }
            texto
        };
        for figura in &cena.figuras {
            if let Figura::Retangulo { area, .. } = figura {
                let origem = format!("{} {} m", numero_pdf(area.x), numero_pdf(area.y));
                assert!(texto.contains(&origem), "posição ausente: {origem}");
            }
        }
    }

    #[test]
    fn pizza_rotulos_exportados_sem_duplicar_fatias() {
        let mut grafico = grafico(Tipo::Pizza);
        grafico.categorias = vec!["Brasil".into(), "Chile".into(), "Peru".into()];
        let fatias = grafico.fatias_pizza(800.0, 400.0).unwrap();
        let n_rotulos = fatias.iter().filter(|f| f.rotulo.is_some()).count();
        assert!(n_rotulos > 0);
        let pdf = grafico.exportar_pdf(800.0, 400.0).unwrap();
        let texto = pdf_comandos(&pdf);
        let svg = grafico.exportar_svg(800.0, 400.0).unwrap();
        assert_eq!(texto.matches(" Tj ET").count(), n_rotulos);
        assert_eq!(svg.matches("<text ").count(), n_rotulos);
        assert_eq!(svg.matches("<path d=").count(), grafico.cena(800.0, 400.0).unwrap().figuras.len());
        assert_eq!(grafico.series[0].valores, vec![10.0, 20.0, 30.0]);
    }

    #[test]
    fn pizza_sem_espaco_nao_exporta_textos_mas_mantem_fatias() {
        let pdf = grafico(Tipo::Pizza).exportar_pdf(30.0, 20.0).unwrap();
        assert_eq!(pdf_comandos(&pdf).matches(" Tj ET").count(), 0);
        let svg = grafico(Tipo::Pizza).exportar_svg(30.0, 20.0).unwrap();
        assert_eq!(svg.matches("<text ").count(), 0);
        assert_eq!(svg.matches("<path d=").count(), 3);
    }

    #[test]
    fn preserva_erros_e_recusa_unicode_incompativel() {
        let mut grafico = grafico(Tipo::Pizza);
        grafico.series[0].valores[1] = -1.0;
        assert_eq!(
            grafico.exportar_pdf(640.0, 480.0),
            Err(ErroPdfGrafico::Grafico(ErroGrafico::ValorInvalido))
        );
        grafico.series[0].valores[1] = 20.0;
        assert_eq!(
            grafico.exportar_pdf(f32::NAN, 480.0),
            Err(ErroPdfGrafico::Grafico(ErroGrafico::EixoInvalido))
        );
        grafico.categorias[0] = "東京".into();
        // Sem fonte CJK incorporada não se pode substituir caracteres.
        assert_eq!(
            grafico.exportar_pdf(800.0, 400.0),
            Err(ErroPdfGrafico::Pdf(ErroPdf::CaractereNaoSuportado))
        );
    }
}
