//! Modelos de gráfico: toda a aparência (e a tipografia, sem os textos do
//! gráfico) num JSON que se salva, carrega e aplica a outro gráfico. É a base
//! do futuro arquivo `.fchart`.
use crate::{AparenciaGrafico, ErroGrafico, TipografiaGrafico};
use serde::{Deserialize, Serialize};

/// Identificador gravado no JSON, para recusar arquivos de outra coisa.
pub const FORMATO_MODELO: &str = "feather-charts/modelo";
/// Versão do formato que este código grava e entende.
pub const VERSAO_MODELO: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModeloGrafico {
    pub formato: String,
    pub versao: u32,
    /// Nome para a lista de modelos da interface.
    pub nome: String,
    #[serde(default)]
    pub aparencia: AparenciaGrafico,
    /// Formato dos números, tamanho do texto e o que mostrar. O subtítulo e a
    /// unidade são do gráfico, não do modelo, e não são aplicados.
    #[serde(default)]
    pub tipografia: TipografiaGrafico,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErroModelo {
    /// JSON mal formado ou campo com tipo errado (mensagem do leitor).
    Json(String),
    /// Não é um modelo do feather-charts.
    FormatoDesconhecido,
    /// Modelo gravado por uma versão mais nova.
    VersaoNaoSuportada(u32),
    /// Os valores lidos não passam na validação da aparência.
    Invalido(ErroGrafico),
}

impl ModeloGrafico {
    pub fn novo(nome: &str, aparencia: AparenciaGrafico, tipografia: TipografiaGrafico) -> Self {
        let tipografia = TipografiaGrafico { subtitulo: String::new(), unidade: String::new(), ..tipografia };
        Self {
            formato: FORMATO_MODELO.into(),
            versao: VERSAO_MODELO,
            nome: nome.into(),
            aparencia,
            tipografia,
        }
    }

    pub fn validar(&self) -> Result<(), ErroModelo> {
        if self.formato != FORMATO_MODELO {
            return Err(ErroModelo::FormatoDesconhecido);
        }
        if self.versao > VERSAO_MODELO {
            return Err(ErroModelo::VersaoNaoSuportada(self.versao));
        }
        self.aparencia.validar().map_err(ErroModelo::Invalido)?;
        self.tipografia.validar().map_err(ErroModelo::Invalido)
    }

    /// JSON legível (indentado), com as cores em `#rrggbb`/`#rrggbbaa`.
    pub fn para_json(&self) -> Result<String, ErroModelo> {
        self.validar()?;
        serde_json::to_string_pretty(self).map_err(|e| ErroModelo::Json(e.to_string()))
    }

    /// Lê e valida. Campos ausentes ficam com o valor padrão, então um
    /// modelo pequeno (só o que muda) também é aceito.
    pub fn de_json(texto: &str) -> Result<Self, ErroModelo> {
        let m: Self = serde_json::from_str(texto).map_err(|e| ErroModelo::Json(e.to_string()))?;
        m.validar()?;
        Ok(m)
    }

    /// Aparência e tipografia para outro gráfico: tudo vem do modelo, menos
    /// o subtítulo e a unidade, que continuam os do gráfico de destino.
    pub fn aplicar(&self, tipografia_atual: &TipografiaGrafico) -> (AparenciaGrafico, TipografiaGrafico) {
        let tipografia = TipografiaGrafico {
            subtitulo: tipografia_atual.subtitulo.clone(),
            unidade: tipografia_atual.unidade.clone(),
            ..self.tipografia.clone()
        };
        (self.aparencia.clone(), tipografia)
    }
}
