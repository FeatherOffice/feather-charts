//! Contrato de dados prontos para renderização, independente de planilhas e Iced.
//! A origem decide como agregar e filtrar; Charts somente valida e desenha.
use crate::{ErroGrafico, Grafico};
use feather_draw::{Cena, Cor};

/// Índice original de uma série na fonte, preservado mesmo com filtros.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadadosSerie {
    pub indice_origem: u32,
    pub nome: String,
    pub cor: Cor,
    /// Nome da operação já executada pela origem, quando pertinente.
    pub agregacao: Option<String>,
}

/// Redução de dados aplicada pela fonte ANTES do desenho (não é LTTB).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AmostraDados {
    pub disponiveis: usize,
    pub usados: usize,
}

/// Entrada compartilhada de renderização. Não contém Sheet, células ou callbacks
/// de agregação e não participa da serialização de arquivos dos aplicativos.
#[derive(Debug, Clone, PartialEq)]
pub struct ContratoGrafico {
    pub grafico: Grafico,
    pub metadados_series: Vec<MetadadosSerie>,
    pub amostra: Option<AmostraDados>,
    pub avisos: Vec<String>,
}

impl ContratoGrafico {
    /// Rejeita metadados desalinhados, sem alterar séries ou categorias.
    pub fn validar(&self) -> Result<(), ErroGrafico> {
        self.grafico.validar()?;
        if self.metadados_series.len() != self.grafico.series.len()
            || self.metadados_series.iter().zip(&self.grafico.series).any(|(m, s)| {
                m.nome != s.nome || m.cor != s.cor
            })
        {
            return Err(ErroGrafico::TamanhosDiferentes);
        }
        if let Some(amostra) = self.amostra {
            if amostra.usados > amostra.disponiveis {
                return Err(ErroGrafico::ValorInvalido);
            }
        }
        Ok(())
    }

    /// Somente renderiza dados previamente calculados pela origem.
    pub fn cena(&self, largura: f32, altura: f32) -> Result<Cena, ErroGrafico> {
        self.validar()?;
        self.grafico.cena(largura, altura)
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::{Serie, Tipo};

    #[test]
    fn exige_metadados_coerentes_e_nao_agrega_valores() {
        let grafico = Grafico {
            titulo: "Teste".into(), tipo: Tipo::Colunas,
            categorias: vec!["A".into()],
            series: vec![Serie { nome: "S".into(), valores: vec![42.0], cor: Cor::rgb(1, 2, 3) }],
        };
        let mut contrato = ContratoGrafico {
            grafico: grafico.clone(),
            metadados_series: vec![MetadadosSerie {
                indice_origem: 7, nome: "S".into(),
                cor: Cor::rgb(1, 2, 3), agregacao: Some("Soma".into()),
            }],
            amostra: None, avisos: Vec::new(),
        };
        assert!(contrato.cena(400.0, 300.0).is_ok());
        assert_eq!(contrato.grafico, grafico);
        contrato.metadados_series[0].nome = "Outra".into();
        assert_eq!(contrato.validar(), Err(ErroGrafico::TamanhosDiferentes));
    }
}
