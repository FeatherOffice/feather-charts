//! Personalização por elemento: séries, pontos, barras, rótulos de dados e
//! eixos. Tudo é opcional e o padrão reproduz o desenho anterior. Todos os tipos
//! são serializáveis (JSON) para formar o `ModeloGrafico`.
use crate::tipografia::LocalidadeGrafico;
use crate::{ErroGrafico, Marcador};
use feather_draw::Cor;
use serde::{Deserialize, Serialize};

fn finito_positivo(v: f32) -> bool {
    v.is_finite() && v > 0.0
}
fn opcional_positivo(v: Option<f32>) -> bool {
    v.map_or(true, finito_positivo)
}

// ------------------------------------------------------------------ traço

/// Padrão do traço de uma linha (série ou grade).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TracoLinha {
    #[default]
    Continuo,
    Tracejado,
    Pontilhado,
    TracoPonto,
}
impl TracoLinha {
    /// Valores alternados tinta/espaço (vazio = contínuo), proporcionais à
    /// espessura para o padrão continuar legível em linhas grossas.
    pub fn padrao(self, espessura: f32) -> Vec<f32> {
        let l = espessura.max(1.0);
        match self {
            Self::Continuo => Vec::new(),
            Self::Tracejado => vec![4.0 * l, 3.0 * l],
            Self::Pontilhado => vec![l, 2.0 * l],
            Self::TracoPonto => vec![4.0 * l, 2.0 * l, l, 2.0 * l],
        }
    }
}

/// Linha de grade: cor, espessura e traço.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LinhaGrade {
    #[serde(with = "crate::serde_cor::cor")]
    pub cor: Cor,
    pub espessura: f32,
    pub traco: TracoLinha,
}
impl Default for LinhaGrade {
    fn default() -> Self {
        Self { cor: Cor::rgb(225, 228, 232), espessura: 1.0, traco: TracoLinha::Continuo }
    }
}
impl LinhaGrade {
    pub fn validar(&self) -> Result<(), ErroGrafico> {
        if finito_positivo(self.espessura) { Ok(()) } else { Err(ErroGrafico::ValorInvalido) }
    }
}

// ----------------------------------------------------------- séries/pontos

/// Marcador de uma série: forma, tamanho e cores. O que ficar `None` herda
/// da aparência global ou da cor da série.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EstiloMarcador {
    pub forma: Marcador,
    pub tamanho: Option<f32>,
    #[serde(with = "crate::serde_cor::opt")]
    pub preenchimento: Option<Cor>,
    #[serde(with = "crate::serde_cor::opt")]
    pub contorno: Option<Cor>,
    pub espessura_contorno: Option<f32>,
}
impl Default for EstiloMarcador {
    fn default() -> Self {
        Self {
            forma: Marcador::Circulo,
            tamanho: None,
            preenchimento: None,
            contorno: None,
            espessura_contorno: None,
        }
    }
}

/// Cor própria de um ponto (coluna, fatia ou marcador); vale mais que a série.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CorPonto {
    pub indice: usize,
    #[serde(with = "crate::serde_cor::cor")]
    pub cor: Cor,
}

/// Cor pelo valor do ponto.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RegraCor {
    /// `acima` quando o valor é maior ou igual à meta; senão `abaixo`.
    Meta {
        meta: f64,
        #[serde(with = "crate::serde_cor::cor")]
        acima: Cor,
        #[serde(with = "crate::serde_cor::cor")]
        abaixo: Cor,
    },
    /// Escala contínua entre 2 ou 3 cores igualmente espaçadas. Sem
    /// `minimo`/`maximo`, usa o menor e o maior valor da série.
    Escala {
        minimo: Option<f64>,
        maximo: Option<f64>,
        #[serde(with = "crate::serde_cor::lista")]
        cores: Vec<Cor>,
    },
}
impl RegraCor {
    pub fn validar(&self) -> Result<(), ErroGrafico> {
        match self {
            Self::Meta { meta, .. } if !meta.is_finite() => Err(ErroGrafico::ValorInvalido),
            Self::Escala { cores, minimo, maximo }
                if !(2..=3).contains(&cores.len())
                    || minimo.map_or(false, |v| !v.is_finite())
                    || maximo.map_or(false, |v| !v.is_finite()) =>
            {
                Err(ErroGrafico::ValorInvalido)
            }
            _ => Ok(()),
        }
    }

    /// `serie_min`/`serie_max` entram quando a escala não fixa os limites.
    pub fn cor(&self, valor: f64, serie_min: f64, serie_max: f64) -> Cor {
        match self {
            Self::Meta { meta, acima, abaixo } => {
                if valor >= *meta { *acima } else { *abaixo }
            }
            Self::Escala { minimo, maximo, cores } => {
                let lo = minimo.unwrap_or(serie_min);
                let hi = maximo.unwrap_or(serie_max);
                let t = if hi > lo { ((valor - lo) / (hi - lo)).clamp(0.0, 1.0) } else { 0.0 };
                let n = cores.len();
                if n == 0 {
                    return Cor::rgb(0, 0, 0);
                }
                if n == 1 {
                    return cores[0];
                }
                let pos = t * (n - 1) as f64;
                let k = (pos.floor() as usize).min(n - 2);
                misturar(cores[k], cores[k + 1], (pos - k as f64) as f32)
            }
        }
    }
}

/// Interpolação linear entre duas cores (`t` em 0..=1).
pub fn misturar(a: Cor, b: Cor, t: f32) -> Cor {
    let t = t.clamp(0.0, 1.0);
    let m = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    Cor::rgba(m(a.r, b.r), m(a.g, b.g), m(a.b, b.b), m(a.a, b.a))
}

/// Estilo de uma série (índice na lista `AparenciaGrafico::series`, que é o
/// índice da série no gráfico original).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EstiloSerie {
    /// Cor da série; sobrepõe a paleta e a cor original.
    #[serde(with = "crate::serde_cor::opt")]
    pub cor: Option<Cor>,
    /// Espessura da linha (linhas, áreas, degraus).
    pub espessura: Option<f32>,
    pub traco: TracoLinha,
    /// Multiplica o alfa de tudo que a série desenha (0 a 1).
    pub opacidade: Option<f32>,
    pub marcador: Option<EstiloMarcador>,
    /// Contorno das colunas/barras da série.
    #[serde(with = "crate::serde_cor::opt")]
    pub contorno: Option<Cor>,
    pub espessura_contorno: Option<f32>,
    /// Cor por ponto: vale mais que a série e que a regra.
    pub pontos: Vec<CorPonto>,
    /// Cor pelo valor: acima/abaixo de uma meta, ou escala contínua.
    pub regra: Option<RegraCor>,
    /// Liga ou desliga os rótulos de dados desta série; `None` herda.
    pub rotulos: Option<bool>,
}
impl EstiloSerie {
    pub fn validar(&self) -> Result<(), ErroGrafico> {
        let ok_op = self.opacidade.map_or(true, |o| o.is_finite() && (0.0..=1.0).contains(&o));
        let ok_esp = self.espessura.map_or(true, |e| e.is_finite() && e >= 0.0)
            && self.espessura_contorno.map_or(true, |e| e.is_finite() && e >= 0.0);
        let ok_marc = self.marcador.map_or(true, |m| {
            opcional_positivo(m.tamanho)
                && m.espessura_contorno.map_or(true, |e| e.is_finite() && e >= 0.0)
        });
        if !(ok_op && ok_esp && ok_marc) {
            return Err(ErroGrafico::ValorInvalido);
        }
        if let Some(r) = &self.regra {
            r.validar()?;
        }
        Ok(())
    }
    pub fn cor_do_ponto(&self, indice: usize) -> Option<Cor> {
        self.pontos.iter().rev().find(|p| p.indice == indice).map(|p| p.cor)
    }
}

// ------------------------------------------------------------------ barras

/// Gradiente ao longo da barra, da base à ponta.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GradienteBarra {
    /// Cor da ponta; `None` usa a cor da série clareada.
    #[serde(with = "crate::serde_cor::opt")]
    pub cor_final: Option<Cor>,
    /// Começa clara na base e termina na cor da série (em vez do contrário).
    pub invertido: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EstiloBarras {
    pub gradiente: Option<GradienteBarra>,
    /// Como o `gapWidth` do Excel: espaço entre grupos, em % da largura da
    /// barra (0 a 500). `None` mantém a largura anterior.
    pub espaco_grupos: Option<f32>,
    /// Como o `overlap` do Excel: -100 (afastadas) a 100 (totalmente
    /// sobrepostas), em % da largura da barra. `None` mantém o anterior.
    pub sobreposicao: Option<f32>,
}
impl EstiloBarras {
    pub fn validar(&self) -> Result<(), ErroGrafico> {
        let g = self.espaco_grupos.map_or(true, |v| v.is_finite() && (0.0..=500.0).contains(&v));
        let o = self.sobreposicao.map_or(true, |v| v.is_finite() && (-100.0..=100.0).contains(&v));
        if g && o { Ok(()) } else { Err(ErroGrafico::ValorInvalido) }
    }
    pub fn mexe_na_geometria(&self) -> bool {
        self.espaco_grupos.is_some() || self.sobreposicao.is_some()
    }
}

/// Clareia uma cor na direção do branco (`f` de 0 a 1).
pub fn clarear(c: Cor, f: f32) -> Cor {
    misturar(c, Cor::rgba(255, 255, 255, c.a), f)
}

// ------------------------------------------------------- rótulos de dados

/// Fonte de um texto do gráfico; `None` herda (tamanho da tipografia, cor do texto).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FonteTexto {
    pub tamanho: Option<f32>,
    #[serde(with = "crate::serde_cor::opt")]
    pub cor: Option<Cor>,
    pub negrito: bool,
}
impl FonteTexto {
    pub fn validar(&self) -> Result<(), ErroGrafico> {
        if opcional_positivo(self.tamanho) { Ok(()) } else { Err(ErroGrafico::ValorInvalido) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PosicaoRotulo {
    /// Fora, depois da ponta (acima da coluna, à direita da barra, em cima do
    /// ponto; na pizza, ao redor com linha guia).
    #[default]
    Fora,
    /// Dentro, junto da ponta.
    PontaDentro,
    Centro,
    /// Dentro, junto da base (em linhas e pontos, logo abaixo do ponto).
    Base,
}

/// O que o rótulo diz. Sai na ordem série, categoria, valor, porcentagem.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConteudoRotulo {
    pub valor: bool,
    pub categoria: bool,
    pub serie: bool,
    /// Parte do total da categoria (na pizza, do total das fatias).
    pub porcentagem: bool,
    pub separador: String,
}
impl Default for ConteudoRotulo {
    fn default() -> Self {
        Self { valor: true, categoria: false, serie: false, porcentagem: false, separador: "; ".into() }
    }
}

/// Número formatado como no Brasil: `R$ 1.234,50`, `12,5 %`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FormatoRotuloNumerico {
    pub casas: u8,
    pub separador_milhar: bool,
    pub prefixo: String,
    pub sufixo: String,
    pub localidade: LocalidadeGrafico,
}
impl Default for FormatoRotuloNumerico {
    fn default() -> Self {
        Self {
            casas: 1,
            separador_milhar: true,
            prefixo: String::new(),
            sufixo: String::new(),
            localidade: LocalidadeGrafico::PtBr,
        }
    }
}
impl FormatoRotuloNumerico {
    pub fn novo(casas: u8, prefixo: &str, sufixo: &str) -> Self {
        Self { casas, prefixo: prefixo.into(), sufixo: sufixo.into(), ..Self::default() }
    }
    pub fn validar(&self) -> Result<(), ErroGrafico> {
        if self.casas > 6 || self.prefixo.len() > 32 || self.sufixo.len() > 32 {
            return Err(ErroGrafico::ValorInvalido);
        }
        Ok(())
    }
    pub fn formatar(&self, valor: f64) -> String {
        if !valor.is_finite() {
            return String::new();
        }
        let (milhar, decimal) = match self.localidade {
            LocalidadeGrafico::PtBr => ('.', ','),
            LocalidadeGrafico::EnUs => (',', '.'),
        };
        let casas = usize::from(self.casas.min(6));
        let numero = valor.abs();
        let corpo = if numero >= 1e15 {
            let e = format!("{:.*e}", casas, numero);
            if decimal == ',' { e.replacen('.', ",", 1) } else { e }
        } else {
            let bruto = format!("{:.*}", casas, numero);
            let (inteiro, fracao) = bruto.split_once('.').unwrap_or((&bruto, ""));
            let mut s = String::with_capacity(bruto.len() + inteiro.len() / 3 + 1);
            for (i, c) in inteiro.chars().enumerate() {
                if self.separador_milhar && i > 0 && (inteiro.len() - i) % 3 == 0 {
                    s.push(milhar);
                }
                s.push(c);
            }
            if casas > 0 {
                s.push(decimal);
                s.push_str(fracao);
            }
            s
        };
        let zero = !corpo.chars().any(|c| c.is_ascii_digit() && c != '0');
        let sinal = if valor < 0.0 && !zero { "-" } else { "" };
        format!("{sinal}{}{corpo}{}", self.prefixo, self.sufixo)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RotulosDados {
    /// Rótulos em todas as séries (cada série pode contrariar em `EstiloSerie::rotulos`).
    pub ativo: bool,
    pub conteudo: ConteudoRotulo,
    pub posicao: PosicaoRotulo,
    pub formato: FormatoRotuloNumerico,
    /// Casas da porcentagem (padrão 1).
    pub casas_porcentagem: u8,
    pub fonte: FonteTexto,
    /// Pizza com rótulo fora: desenha a linha guia até a fatia.
    pub linha_guia: bool,
}
impl Default for RotulosDados {
    fn default() -> Self {
        Self {
            ativo: true,
            conteudo: ConteudoRotulo::default(),
            posicao: PosicaoRotulo::Fora,
            formato: FormatoRotuloNumerico::default(),
            casas_porcentagem: 1,
            fonte: FonteTexto::default(),
            linha_guia: true,
        }
    }
}
impl RotulosDados {
    pub fn validar(&self) -> Result<(), ErroGrafico> {
        self.formato.validar()?;
        self.fonte.validar()?;
        if self.casas_porcentagem > 6 || self.conteudo.separador.len() > 16 {
            return Err(ErroGrafico::ValorInvalido);
        }
        Ok(())
    }
    pub fn porcentagem_texto(&self, parte: f64) -> String {
        FormatoRotuloNumerico {
            casas: self.casas_porcentagem,
            separador_milhar: false,
            prefixo: String::new(),
            sufixo: "%".into(),
            localidade: self.formato.localidade,
        }
        .formatar(parte * 100.0)
    }
}

// ------------------------------------------------------------------- eixos

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RotacaoRotulos {
    #[default]
    Graus0,
    Graus45,
    Graus90,
}
impl RotacaoRotulos {
    pub fn graus(self) -> f32 {
        match self {
            Self::Graus0 => 0.0,
            Self::Graus45 => 45.0,
            Self::Graus90 => 90.0,
        }
    }
}

/// Configuração de um eixo. `valores` é o eixo numérico (Y nas colunas, X nas
/// barras); `categorias` é o outro (em dispersão, o X numérico).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConfigEixo {
    /// Desenhar o eixo (linha, marcas e rótulos).
    pub mostrar: bool,
    /// Marcas junto à linha do eixo.
    pub mostrar_marcas: bool,
    /// Só no eixo de valores: limites e intervalo fixos (`None` = automático).
    pub minimo: Option<f64>,
    pub maximo: Option<f64>,
    pub intervalo: Option<f64>,
    /// Só no eixo de valores: escala logarítmica (base 10).
    pub logaritmica: bool,
    /// Inverte a ordem (valores de cima para baixo; categorias de trás para frente).
    pub invertido: bool,
    /// Formato dos rótulos numéricos; `None` usa o da tipografia.
    pub formato: Option<FormatoRotuloNumerico>,
    pub rotacao: RotacaoRotulos,
    pub titulo: Option<String>,
    pub fonte_titulo: FonteTexto,
    pub grade_principal: Option<LinhaGrade>,
    pub grade_secundaria: Option<LinhaGrade>,
    /// Linhas secundárias entre duas principais (eixo linear; 2 a 10).
    pub subdivisoes: u8,
}
impl Default for ConfigEixo {
    fn default() -> Self {
        Self {
            mostrar: true,
            mostrar_marcas: true,
            minimo: None,
            maximo: None,
            intervalo: None,
            logaritmica: false,
            invertido: false,
            formato: None,
            rotacao: RotacaoRotulos::Graus0,
            titulo: None,
            fonte_titulo: FonteTexto::default(),
            grade_principal: None,
            grade_secundaria: None,
            subdivisoes: 4,
        }
    }
}
impl ConfigEixo {
    pub fn validar(&self) -> Result<(), ErroGrafico> {
        let lim = |v: Option<f64>| v.map_or(true, f64::is_finite);
        if !lim(self.minimo) || !lim(self.maximo) || !lim(self.intervalo) {
            return Err(ErroGrafico::EixoInvalido);
        }
        if let (Some(a), Some(b)) = (self.minimo, self.maximo) {
            if a >= b {
                return Err(ErroGrafico::EixoInvalido);
            }
        }
        if self.intervalo.map_or(false, |i| i <= 0.0) || !(2..=10).contains(&self.subdivisoes) {
            return Err(ErroGrafico::EixoInvalido);
        }
        if self.logaritmica && (self.minimo.map_or(false, |m| m <= 0.0) || self.maximo.map_or(false, |m| m <= 0.0)) {
            return Err(ErroGrafico::EixoInvalido);
        }
        if let Some(f) = &self.formato {
            f.validar()?;
        }
        self.fonte_titulo.validar()?;
        if self.titulo.as_ref().map_or(false, |t| t.len() > 512) {
            return Err(ErroGrafico::ValorInvalido);
        }
        for g in [&self.grade_principal, &self.grade_secundaria].into_iter().flatten() {
            g.validar()?;
        }
        Ok(())
    }
    /// Algum ajuste muda a escala (e não só o aspecto).
    pub fn muda_escala(&self) -> bool {
        self.minimo.is_some() || self.maximo.is_some() || self.logaritmica || self.invertido
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ConfigEixos {
    pub valores: ConfigEixo,
    pub categorias: ConfigEixo,
}
impl ConfigEixos {
    pub fn validar(&self) -> Result<(), ErroGrafico> {
        self.valores.validar()?;
        self.categorias.validar()
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn formato_pt_br_com_prefixo_sufixo_e_sinal() {
        let f = FormatoRotuloNumerico::novo(2, "R$ ", "");
        assert_eq!(f.formatar(1234567.891), "R$ 1.234.567,89");
        assert_eq!(f.formatar(-1234.5), "-R$ 1.234,50");
        assert_eq!(f.formatar(-0.001), "R$ 0,00", "zero arredondado não leva sinal");
        let p = FormatoRotuloNumerico::novo(1, "", " %");
        assert_eq!(p.formatar(12.5), "12,5 %");
        let s = FormatoRotuloNumerico { separador_milhar: false, casas: 0, ..Default::default() };
        assert_eq!(s.formatar(1234567.0), "1234567");
        let en = FormatoRotuloNumerico { localidade: LocalidadeGrafico::EnUs, casas: 2, ..Default::default() };
        assert_eq!(en.formatar(9876.5), "9,876.50");
        assert_eq!(f.formatar(f64::NAN), "");
        assert!(FormatoRotuloNumerico { casas: 9, ..Default::default() }.validar().is_err());
    }

    #[test]
    fn regra_de_meta_e_escala_continua() {
        let verde = Cor::rgb(0, 160, 0);
        let verm = Cor::rgb(200, 0, 0);
        let meta = RegraCor::Meta { meta: 20.0, acima: verde, abaixo: verm };
        assert_eq!(meta.cor(20.0, 0.0, 0.0), verde);
        assert_eq!(meta.cor(19.9, 0.0, 0.0), verm);
        let esc = RegraCor::Escala {
            minimo: None, maximo: None,
            cores: vec![Cor::rgb(0, 0, 255), Cor::rgb(255, 255, 255), Cor::rgb(255, 0, 0)],
        };
        assert_eq!(esc.cor(0.0, 0.0, 10.0), Cor::rgb(0, 0, 255));
        assert_eq!(esc.cor(5.0, 0.0, 10.0), Cor::rgb(255, 255, 255));
        assert_eq!(esc.cor(10.0, 0.0, 10.0), Cor::rgb(255, 0, 0));
        assert_eq!(esc.cor(2.5, 0.0, 10.0), Cor::rgb(128, 128, 255));
        assert!(RegraCor::Escala { minimo: None, maximo: None, cores: vec![verde] }.validar().is_err());
    }

    #[test]
    fn traco_proporcional_e_validacoes() {
        assert!(TracoLinha::Continuo.padrao(2.0).is_empty());
        assert_eq!(TracoLinha::Tracejado.padrao(2.0), vec![8.0, 6.0]);
        assert_eq!(TracoLinha::TracoPonto.padrao(1.0).len(), 4);
        assert!(EstiloBarras { sobreposicao: Some(150.0), ..Default::default() }.validar().is_err());
        assert!(ConfigEixo { minimo: Some(5.0), maximo: Some(1.0), ..Default::default() }.validar().is_err());
        assert!(ConfigEixo { logaritmica: true, minimo: Some(0.0), ..Default::default() }.validar().is_err());
        assert!(EstiloSerie { opacidade: Some(2.0), ..Default::default() }.validar().is_err());
    }
}
