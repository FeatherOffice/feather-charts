//! Normalização robusta de pizza e geometria dos rótulos sem dependência de GUI.
use crate::{ErroGrafico, Grafico, Tipo};
use feather_draw::{Ponto, Retangulo};

/// Geometria reservada ao texto; renderização tipográfica é do adaptador.
#[derive(Debug, Clone, PartialEq)]
pub struct RotuloPizza {
    pub texto: String,
    pub area: Retangulo,
    pub ancora: Ponto,
}
#[derive(Debug, Clone, PartialEq)]
pub struct FatiaPizza {
    pub indice: usize,
    pub categoria: String,
    pub valor: f64,
    pub proporcao: f64,
    pub inicio: f64,
    pub fim: f64,
    pub rotulo: Option<RotuloPizza>,
}
fn reduzir_texto(s: &str, maximo: usize) -> String {
    if s.chars().count() <= maximo {
        return s.into();
    }
    if maximo <= 1 {
        return "…".into();
    }
    let mut t: String = s.chars().take(maximo - 1).collect();
    t.push('…');
    t
}
impl Grafico {
    /// Calcula fatias sem somar diretamente f64 muito grandes.
    /// Zeros não geram área; categorias originais permanecem intocadas.
    /// Ao faltar espaço, oculta rótulos menores em vez de sobrepô-los.
    pub fn fatias_pizza(&self, largura: f32, altura: f32) -> Result<Vec<FatiaPizza>, ErroGrafico> {
        if self.tipo != Tipo::Pizza {
            return Err(ErroGrafico::EixoInvalido);
        }
        self.validar()?;
        if !largura.is_finite() || !altura.is_finite() || largura <= 0. || altura <= 0. {
            return Err(ErroGrafico::EixoInvalido);
        }
        let valores = &self.series[0].valores;
        let maior = valores.iter().copied().fold(0.0_f64, f64::max);
        let soma: f64 = valores.iter().map(|v| *v / maior).sum();
        let quantidade = valores.iter().filter(|v| **v > 0.).count();
        let mut angulo = -std::f64::consts::FRAC_PI_2;
        let mut fatias = Vec::with_capacity(quantidade);
        for (i, &valor) in valores.iter().enumerate() {
            if valor <= 0. {
                continue;
            }
            let proporcao = (valor / maior) / soma;
            let inicio = angulo;
            angulo = if fatias.len() + 1 == quantidade {
                -std::f64::consts::FRAC_PI_2 + std::f64::consts::TAU
            } else {
                angulo + std::f64::consts::TAU * proporcao
            };
            fatias.push(FatiaPizza {
                indice: i,
                categoria: self.categorias[i].clone(),
                valor,
                proporcao,
                inicio,
                fim: angulo,
                rotulo: None,
            });
        }
        const LINHA: f32 = 14.;
        const MARGEM: f32 = 4.;
        if altura < LINHA + 2. * MARGEM {
            return Ok(fatias);
        }
        let centro = Ponto::novo(largura / 2., altura / 2.);
        let raio = largura.min(altura) * 0.38;
        let limite = ((altura - 2. * MARGEM) / LINHA).floor() as usize;
        let left_width = (centro.x - raio - 12. - MARGEM).max(0.);
        let right_x = centro.x + raio + 12.;
        let right_width = (largura - right_x - MARGEM).max(0.);
        for (direita, disponivel) in [(false, left_width), (true, right_width)] {
            let max_chars = (disponivel / 7.).floor() as usize;
            if max_chars < 2 || limite == 0 {
                continue;
            }
            let mut candidatos: Vec<(usize, f32)> = fatias
                .iter()
                .enumerate()
                .filter_map(|(pos, f)| {
                    let a = (f.inicio + f.fim) / 2.;
                    if (a.cos() >= 0.) != direita {
                        return None;
                    }
                    let y = (centro.y + raio * a.sin() as f32 - LINHA / 2.)
                        .clamp(MARGEM, altura - MARGEM - LINHA);
                    Some((pos, y))
                })
                .collect();
            candidatos.sort_by(|a, b| {
                fatias[b.0]
                    .proporcao
                    .total_cmp(&fatias[a.0].proporcao)
                    .then_with(|| a.0.cmp(&b.0))
            });
            candidatos.truncate(limite);
            candidatos.sort_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
            let mut escolhidos = Vec::with_capacity(candidatos.len());
            let mut proximo = MARGEM;
            for (i, y) in candidatos {
                let y = y.max(proximo);
                escolhidos.push((i, y));
                proximo = y + LINHA;
            }
            let excedente = escolhidos
                .last()
                .map_or(0., |(_, y)| (y + LINHA + MARGEM - altura).max(0.));
            for (i, y) in escolhidos {
                let f = &mut fatias[i];
                let texto = reduzir_texto(&f.categoria, max_chars);
                let w = texto.chars().count() as f32 * 7.;
                let x = if direita {
                    right_x
                } else {
                    centro.x - raio - 12. - w
                };
                let a = (f.inicio + f.fim) / 2.;
                f.rotulo = Some(RotuloPizza {
                    texto,
                    area: Retangulo::novo(x, y - excedente, w, LINHA),
                    ancora: Ponto::novo(
                        centro.x + raio * a.cos() as f32,
                        centro.y + raio * a.sin() as f32,
                    ),
                });
            }
        }
        Ok(fatias)
    }
}
#[cfg(test)]
mod testes {
    use super::*;
    use crate::{Cor, Serie};
    fn grafico(v: Vec<f64>) -> Grafico {
        Grafico {
            titulo: "Pizza".into(),
            tipo: Tipo::Pizza,
            categorias: (0..v.len()).map(|i| format!("Categoria {i}")).collect(),
            series: vec![Serie {
                nome: "X".into(),
                valores: v,
                cor: Cor::rgb(10, 20, 30),
            }],
        }
    }
    #[test]
    fn valida_zero_negativo_e_nao_finito() {
        assert_eq!(grafico(vec![0., 0.]).validar(), Err(ErroGrafico::SemDados));
        for v in [vec![1., -1.], vec![1., f64::NAN], vec![1., f64::INFINITY]] {
            assert_eq!(grafico(v).validar(), Err(ErroGrafico::ValorInvalido));
        }
    }
    #[test]
    fn normaliza_maximo_sem_overflow_e_preserva_indices() {
        let f = grafico(vec![f64::MAX, 0., f64::MAX])
            .fatias_pizza(640., 480.)
            .unwrap();
        assert_eq!(f.len(), 2);
        assert_eq!(f[0].indice, 0);
        assert_eq!(f[1].indice, 2);
        assert_eq!(f[0].proporcao, 0.5);
        assert_eq!(f[1].proporcao, 0.5);
        assert!((f[1].fim - f[0].inicio - std::f64::consts::TAU).abs() < 1e-12);
        assert!(grafico(vec![f64::MAX, f64::MAX]).cena(640., 480.).is_ok());
    }
    #[test]
    fn caixas_nao_se_sobrepoem_com_muitas_categorias() {
        let f = grafico(vec![1.; 120]).fatias_pizza(960., 480.).unwrap();
        let caixas: Vec<_> = f
            .iter()
            .filter_map(|a| a.rotulo.as_ref().map(|r| r.area))
            .collect();
        assert!(!caixas.is_empty());
        assert!(caixas.len() < f.len());
        for (i, a) in caixas.iter().enumerate() {
            assert!(
                a.x >= 0. && a.y >= 0. && a.x + a.largura <= 960.001 && a.y + a.altura <= 480.001
            );
            for b in caixas.iter().skip(i + 1) {
                assert!(
                    a.x + a.largura <= b.x
                        || b.x + b.largura <= a.x
                        || a.y + a.altura <= b.y
                        || b.y + b.altura <= a.y
                );
            }
        }
    }
    #[test]
    fn sem_espaco_oculta_rotulos_sem_perder_fatias() {
        let f = grafico(vec![1., 2., 3.]).fatias_pizza(30., 20.).unwrap();
        assert_eq!(f.len(), 3);
        assert!(f.iter().all(|x| x.rotulo.is_none()));
    }
}
