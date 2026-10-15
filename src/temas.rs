//! Temáticas de cores: conjuntos nomeados que trocam fundo, texto, grade,
//! eixos e paleta de séries de uma vez. Só cores; combine com os efeitos de
//! `AparenciaGrafico` (arredondamento, suavização, gradiente, neon).
use crate::estilos::Neon;
use crate::{AparenciaGrafico, Grafico};
use feather_draw::Cor;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tema {
    /// Identificador estável (para arquivo e interface), ex. `"oceano"`.
    pub id: &'static str,
    /// Nome em português para a lista da interface.
    pub nome: &'static str,
    pub fundo: Cor,
    pub texto: Cor,
    pub grade: Cor,
    pub eixos: Cor,
    pub paleta: &'static [Cor],
    /// Brilho que acompanha a temática (só o Neon); `None` não mexe no efeito.
    pub brilho: Option<Neon>,
}

const fn c(hex: u32) -> Cor {
    Cor::rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

const NEON_PADRAO: Neon = Neon { intensidade: 0.8, espessura: 8.0, camadas: 3 };

static TEMAS: [Tema; 14] = [
    Tema { id: "oceano", nome: "Oceano", fundo: c(0xF2F8FB), texto: c(0x12303F), grade: c(0xD3E3EA),
        eixos: c(0x5E7C8A), brilho: None,
        paleta: &[c(0x0B6E99), c(0x2AA9C8), c(0x0F4C75), c(0x6CC5D6), c(0x1B9AAA), c(0x3A7CA5), c(0x8FD3E0)] },
    Tema { id: "floresta", nome: "Floresta", fundo: c(0xF4F8F2), texto: c(0x1B3320), grade: c(0xD8E5D3),
        eixos: c(0x607A5E), brilho: None,
        paleta: &[c(0x2D6A4F), c(0x52B788), c(0x1B4332), c(0x95D5B2), c(0x74A057), c(0x40916C), c(0xB7C68B)] },
    Tema { id: "por-do-sol", nome: "Pôr do sol", fundo: c(0xFFF7F0), texto: c(0x3B1F2B), grade: c(0xF2DACB),
        eixos: c(0x8C6A6A), brilho: None,
        paleta: &[c(0xE63946), c(0xF77F00), c(0xFCBF49), c(0xD62828), c(0xFF9E7A), c(0x9D4EDD), c(0xF4A261)] },
    Tema { id: "terra", nome: "Terra", fundo: c(0xF7F2EA), texto: c(0x3A2E22), grade: c(0xE2D6C4),
        eixos: c(0x7D6B58), brilho: None,
        paleta: &[c(0x8B5E3C), c(0xA68A64), c(0x6B705C), c(0xCB997E), c(0x7F5539), c(0xB7B7A4), c(0x9C6644)] },
    Tema { id: "neon", nome: "Neon", fundo: c(0x0B0F1A), texto: c(0xE8F1FF), grade: c(0x1E2A44),
        eixos: c(0x5C6F99), brilho: Some(NEON_PADRAO),
        paleta: &[c(0x00F5FF), c(0xFF2BD6), c(0x39FF14), c(0xFFF01F), c(0xFF6B00), c(0x9D4DFF), c(0x00FF9C)] },
    Tema { id: "pastel", nome: "Pastel", fundo: c(0xFFFFFF), texto: c(0x44444F), grade: c(0xECECF2),
        eixos: c(0x9A9AA8), brilho: None,
        paleta: &[c(0xA8D8EA), c(0xFFB5C2), c(0xB5EAD7), c(0xFFDAC1), c(0xC7CEEA), c(0xE2F0CB), c(0xF3C6F1)] },
    Tema { id: "outono", nome: "Outono", fundo: c(0xFBF4EA), texto: c(0x3D2314), grade: c(0xEBD9C0),
        eixos: c(0x8A6B4F), brilho: None,
        paleta: &[c(0xC1440E), c(0xE09F3E), c(0x9E2A2B), c(0x7F5539), c(0xB08968), c(0x6B8E23), c(0xD4A373)] },
    Tema { id: "inverno", nome: "Inverno", fundo: c(0xF3F6FB), texto: c(0x1D2B43), grade: c(0xD9E1EF),
        eixos: c(0x6B7B99), brilho: None,
        paleta: &[c(0x3F5E8E), c(0x8FB3E0), c(0x1F3A5F), c(0xB8CCE8), c(0x5A7BA8), c(0x7E8FA6), c(0xDCE6F5)] },
    Tema { id: "cafe", nome: "Café", fundo: c(0xF6EFE8), texto: c(0x2E1F16), grade: c(0xE3D3C3),
        eixos: c(0x7A5C45), brilho: None,
        paleta: &[c(0x4B2E1E), c(0x8B5A3C), c(0xC69C72), c(0x6F4E37), c(0xA67B5B), c(0xD9BFA1), c(0x3B2416)] },
    Tema { id: "mono-azul", nome: "Monocromático azul", fundo: c(0xF5F9FF), texto: c(0x0D2A4D), grade: c(0xD8E5F7),
        eixos: c(0x5C7BA5), brilho: None,
        paleta: &[c(0x0B3C78), c(0x1F5FA8), c(0x3F83D1), c(0x6FA8E8), c(0x9CC4F2), c(0xC4DCF8), c(0x072A55)] },
    Tema { id: "mono-verde", nome: "Monocromático verde", fundo: c(0xF5FBF6), texto: c(0x0F3A1E), grade: c(0xD9EBDD),
        eixos: c(0x5E8C6A), brilho: None,
        paleta: &[c(0x0F5A2B), c(0x1E8A47), c(0x3FB56A), c(0x74D199), c(0xA3E3BD), c(0xCDF1DB), c(0x093D1D)] },
    Tema { id: "mono-cinza", nome: "Monocromático cinza", fundo: c(0xFAFAFA), texto: c(0x1F1F1F), grade: c(0xE3E3E3),
        eixos: c(0x757575), brilho: None,
        paleta: &[c(0x2B2B2B), c(0x555555), c(0x808080), c(0xA6A6A6), c(0xC4C4C4), c(0xDADADA), c(0x111111)] },
    Tema { id: "corporativo", nome: "Corporativo", fundo: c(0xFFFFFF), texto: c(0x1F2937), grade: c(0xE5E7EB),
        eixos: c(0x6B7280), brilho: None,
        paleta: &[c(0x1D4ED8), c(0x0F766E), c(0xB45309), c(0x7C3AED), c(0xBE123C), c(0x475569), c(0x0891B2)] },
    Tema { id: "alto-contraste", nome: "Alto contraste", fundo: c(0x000000), texto: c(0xFFFFFF), grade: c(0x595959),
        eixos: c(0xFFFFFF), brilho: None,
        paleta: &[c(0xFFFF00), c(0x00FFFF), c(0xFF00FF), c(0x00FF00), c(0xFF8000), c(0xFFFFFF), c(0xFF4040)] },
];

/// Todas as temáticas, na ordem de exibição para a interface.
pub fn temas() -> &'static [Tema] {
    &TEMAS
}

impl Tema {
    pub fn pelo_id(id: &str) -> Option<&'static Tema> {
        TEMAS.iter().find(|t| t.id == id)
    }

    /// Cor da série `indice` (cíclica).
    pub fn cor_serie(&self, indice: usize) -> Cor {
        self.paleta[indice % self.paleta.len()]
    }

    /// Escreve fundo, texto, grade, eixos e paleta na aparência. Os demais
    /// campos (arredondamento, suavização…) ficam como estão; o brilho só
    /// muda se a temática traz um (Neon).
    pub fn aplicar(&self, aparencia: &mut AparenciaGrafico) {
        aparencia.paleta = self.paleta.to_vec();
        aparencia.fundo = Some(self.fundo);
        aparencia.cor_texto = Some(self.texto);
        aparencia.cor_grade = Some(self.grade);
        aparencia.cor_eixos = Some(self.eixos);
        if self.brilho.is_some() {
            aparencia.neon = self.brilho;
        }
    }

    /// Aparência nova só com a temática.
    pub fn aparencia(&self) -> AparenciaGrafico {
        let mut a = AparenciaGrafico::default();
        self.aplicar(&mut a);
        a
    }

    /// Pinta também as séries do gráfico (para quem desenha sem `AparenciaGrafico`,
    /// como o `motor`). Dados e nomes não mudam.
    pub fn aplicar_ao_grafico(&self, grafico: &mut Grafico) {
        for (i, s) in grafico.series.iter_mut().enumerate() {
            s.cor = self.cor_serie(i);
        }
    }
}

/// Aplica a temática ao gráfico e à aparência de uma vez.
pub fn aplicar_tema(tema: &Tema, grafico: &mut Grafico, aparencia: &mut AparenciaGrafico) {
    tema.aplicar_ao_grafico(grafico);
    tema.aplicar(aparencia);
}

#[cfg(test)]
mod testes {
    use super::*;

    fn luminancia(c: Cor) -> f64 {
        let f = |v: u8| {
            let v = f64::from(v) / 255.0;
            if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
        };
        0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b)
    }
    fn contraste(a: Cor, b: Cor) -> f64 {
        let (x, y) = (luminancia(a), luminancia(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    #[test]
    fn catalogo_completo_ids_unicos_e_paletas() {
        assert_eq!(temas().len(), 14);
        let nomes: Vec<_> = temas().iter().map(|t| t.nome).collect();
        for esperado in ["Oceano", "Floresta", "Pôr do sol", "Terra", "Neon", "Pastel", "Outono",
            "Inverno", "Café", "Monocromático azul", "Monocromático verde",
            "Monocromático cinza", "Corporativo", "Alto contraste"] {
            assert!(nomes.contains(&esperado), "falta {esperado}");
        }
        for (i, t) in temas().iter().enumerate() {
            assert!(t.paleta.len() >= 6, "{}", t.id);
            assert!(temas()[i + 1..].iter().all(|o| o.id != t.id));
            assert_eq!(Tema::pelo_id(t.id), Some(t));
            assert!(contraste(t.texto, t.fundo) >= 4.5, "texto de {} sem contraste", t.id);
        }
        assert!(Tema::pelo_id("nao-existe").is_none());
    }

    #[test]
    fn neon_tem_fundo_escuro_e_cores_saturadas() {
        let n = Tema::pelo_id("neon").unwrap();
        assert!(luminancia(n.fundo) < 0.02);
        assert!(n.brilho.is_some());
        for cor in n.paleta {
            let (mx, mn) = (cor.r.max(cor.g).max(cor.b), cor.r.min(cor.g).min(cor.b));
            assert!(mx >= 200 && mn <= 100, "{cor:?} pouco saturada");
        }
        assert!(Tema::pelo_id("alto-contraste").map(|t| contraste(t.texto, t.fundo)).unwrap() >= 21.0 - 1e-6);
    }

    #[test]
    fn aplicar_troca_cores_sem_tocar_nos_efeitos_do_usuario() {
        let mut ap = AparenciaGrafico { espessura_linha: 4.0, ..Default::default() };
        let oceano = Tema::pelo_id("oceano").unwrap();
        oceano.aplicar(&mut ap);
        assert_eq!(ap.fundo, Some(oceano.fundo));
        assert_eq!(ap.paleta.len(), oceano.paleta.len());
        assert_eq!(ap.espessura_linha, 4.0);
        assert!(ap.neon.is_none());
        Tema::pelo_id("neon").unwrap().aplicar(&mut ap);
        assert!(ap.neon.is_some());
        Tema::pelo_id("pastel").unwrap().aplicar(&mut ap);
        assert!(ap.neon.is_some(), "tema sem brilho não apaga o efeito escolhido");
    }
}
