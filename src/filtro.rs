//! Filtro interativo pela legenda: estado de itens escondidos (séries, ou
//! fatias na pizza) e recálculo do desenho sem eles. O dado nunca é alterado.
use crate::{AparenciaGrafico, ErroGrafico, Grafico, Tipo};
use crate::eixos::Eixos;
use crate::pizza::FatiaPizza;
use feather_draw::{Cena, Cor, Retangulo};
use std::collections::BTreeSet;

/// Índices escondidos. Série na maioria dos tipos; categoria (fatia) na pizza.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FiltroLegenda {
    ocultos: BTreeSet<usize>,
}

impl FiltroLegenda {
    pub fn novo() -> Self {
        Self::default()
    }
    pub fn esta_oculto(&self, indice: usize) -> bool {
        self.ocultos.contains(&indice)
    }
    pub fn ocultos(&self) -> &BTreeSet<usize> {
        &self.ocultos
    }
    /// Há algum item escondido.
    pub fn filtrando(&self) -> bool {
        !self.ocultos.is_empty()
    }

    /// Clique simples: esconde ou mostra o item. Não deixa esconder o último
    /// visível (o gráfico ficaria sem dados); nesse caso nada muda e devolve `false`.
    /// `total` é a quantidade de itens da legenda.
    pub fn alternar(&mut self, indice: usize, total: usize) -> bool {
        if indice >= total {
            return false;
        }
        if self.ocultos.remove(&indice) {
            return true;
        }
        let visiveis = (0..total).filter(|i| !self.ocultos.contains(i)).count();
        if visiveis <= 1 {
            return false;
        }
        self.ocultos.insert(indice);
        true
    }

    /// Ctrl+clique: mostra só este item. Se ele já era o único visível,
    /// volta a mostrar todos.
    pub fn mostrar_so(&mut self, indice: usize, total: usize) -> bool {
        if indice >= total {
            return false;
        }
        let ja_unico = !self.ocultos.contains(&indice) && self.ocultos.len() + 1 == total;
        self.ocultos.clear();
        if !ja_unico {
            self.ocultos.extend((0..total).filter(|i| *i != indice));
        }
        true
    }

    /// Duplo clique: volta tudo.
    pub fn mostrar_todos(&mut self) {
        self.ocultos.clear();
    }
}

/// Um item da legenda desenhada pelo motor, com a área clicável.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemLegenda {
    /// Índice da série (ou da categoria, na pizza) no gráfico original.
    pub indice: usize,
    pub nome: String,
    pub cor: Cor,
    pub escondido: bool,
    /// Quadradinho de cor.
    pub marca: Retangulo,
    /// Marca mais texto: região que responde ao clique.
    pub caixa: Retangulo,
    /// Texto já abreviado e a baseline onde é escrito.
    pub texto: String,
    pub x_texto: f32,
    pub y_texto: f32,
}

impl Grafico {
    /// Quantos itens a legenda tem: categorias na pizza, séries nos demais.
    pub fn total_itens_legenda(&self) -> usize {
        if self.tipo == Tipo::Pizza { self.categorias.len() } else { self.series.len() }
    }

    /// Cópia sem os itens escondidos e o mapa índice-filtrado -> índice-original.
    /// Erro `SemDados` se não sobrar nada visível. Os dados originais não mudam.
    pub fn filtrado(&self, filtro: &FiltroLegenda) -> Result<(Grafico, Vec<usize>), ErroGrafico> {
        let total = self.total_itens_legenda();
        let mapa: Vec<usize> = (0..total).filter(|i| !filtro.esta_oculto(*i)).collect();
        if mapa.is_empty() {
            return Err(ErroGrafico::SemDados);
        }
        if mapa.len() == total {
            return Ok((self.clone(), mapa));
        }
        let mut g = self.clone();
        if self.tipo == Tipo::Pizza {
            g.categorias = mapa.iter().map(|i| self.categorias[*i].clone()).collect();
            g.series[0].valores = mapa.iter().map(|i| self.series[0].valores[*i]).collect();
        } else {
            g.series = mapa.iter().map(|i| self.series[*i].clone()).collect();
        }
        Ok((g, mapa))
    }

    /// Cena sem os itens escondidos; eixos e escala se ajustam ao que sobra e as
    /// cores continuam as da paleta original.
    pub fn cena_filtrada(
        &self,
        largura: f32,
        altura: f32,
        aparencia: &AparenciaGrafico,
        filtro: &FiltroLegenda,
    ) -> Result<Cena, ErroGrafico> {
        self.validar()?;
        let (g, mapa) = self.filtrado(filtro)?;
        g.cena_impl(largura, altura, None, aparencia, Some(&mapa))
    }

    /// Eixos recalculados só com o que está visível.
    pub fn eixos_filtrados(
        &self,
        largura: f32,
        altura: f32,
        aparencia: &AparenciaGrafico,
        filtro: &FiltroLegenda,
    ) -> Result<Option<Eixos>, ErroGrafico> {
        self.validar()?;
        self.filtrado(filtro)?.0.eixos_com_layout(largura, altura, &aparencia.layout)
    }

    /// Fatias da pizza sem as escondidas: proporções refeitas com o total visível,
    /// `indice` continua o da categoria original.
    pub fn fatias_pizza_filtradas(
        &self,
        largura: f32,
        altura: f32,
        filtro: &FiltroLegenda,
    ) -> Result<Vec<FatiaPizza>, ErroGrafico> {
        self.validar()?;
        let (g, mapa) = self.filtrado(filtro)?;
        let mut fatias = g.fatias_pizza(largura, altura)?;
        for f in &mut fatias {
            f.indice = mapa[f.indice];
        }
        Ok(fatias)
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::{Serie, TipografiaGrafico};
    use feather_draw::Ponto;

    fn grafico(tipo: Tipo) -> Grafico {
        Grafico {
            titulo: "Filtro".into(),
            tipo,
            categorias: vec!["Jan".into(), "Fev".into(), "Mar".into()],
            series: vec![
                Serie { nome: "Pequena".into(), valores: vec![1.0, 2.0, 3.0], cor: Cor::rgb(200, 30, 30) },
                Serie { nome: "Grande".into(), valores: vec![100.0, 80.0, 120.0], cor: Cor::rgb(30, 30, 200) },
                Serie { nome: "Media".into(), valores: vec![10.0, 20.0, 15.0], cor: Cor::rgb(30, 160, 30) },
            ],
        }
    }

    #[test]
    fn alternar_so_e_todos() {
        let mut f = FiltroLegenda::novo();
        assert!(f.alternar(1, 3));
        assert!(f.esta_oculto(1));
        assert!(f.alternar(1, 3));
        assert!(!f.filtrando());
        // mostrar só um, e repetir devolve tudo
        assert!(f.mostrar_so(2, 3));
        assert_eq!(f.ocultos().iter().copied().collect::<Vec<_>>(), vec![0, 1]);
        assert!(f.mostrar_so(2, 3));
        assert!(!f.filtrando());
        // não esconde o último visível
        f.alternar(0, 3);
        f.alternar(1, 3);
        assert!(!f.alternar(2, 3));
        assert!(!f.esta_oculto(2));
        f.mostrar_todos();
        assert!(!f.filtrando());
        assert!(!f.alternar(7, 3));
    }

    #[test]
    fn eixos_recalculam_sem_a_serie_escondida_e_dados_ficam() {
        let g = grafico(Tipo::Colunas);
        let ap = AparenciaGrafico::default();
        let tudo = g.eixos_filtrados(600.0, 400.0, &ap, &FiltroLegenda::novo()).unwrap().unwrap();
        assert_eq!(tudo.escala_y.unwrap().maximo, 120.0);
        let mut f = FiltroLegenda::novo();
        f.alternar(1, 3);
        let sem = g.eixos_filtrados(600.0, 400.0, &ap, &f).unwrap().unwrap();
        assert_eq!(sem.escala_y.unwrap().maximo, 20.0);
        assert_eq!(g.series.len(), 3);
        assert_eq!(g.series[1].valores[0], 100.0);
        // a cena tem 2 séries x 3 categorias de colunas
        let cena = g.cena_filtrada(600.0, 400.0, &ap, &f).unwrap();
        let n = cena.figuras.iter()
            .filter(|f| matches!(f, feather_draw::Figura::Retangulo { .. })).count();
        assert_eq!(n, 6);
    }

    #[test]
    fn cores_da_paleta_continuam_as_originais() {
        let g = grafico(Tipo::Colunas);
        let ap = AparenciaGrafico {
            paleta: vec![Cor::rgb(1, 1, 1), Cor::rgb(2, 2, 2), Cor::rgb(3, 3, 3)],
            ..Default::default()
        };
        let mut f = FiltroLegenda::novo();
        f.alternar(0, 3);
        let cena = g.cena_filtrada(600.0, 400.0, &ap, &f).unwrap();
        let cores: Vec<_> = cena.figuras.iter().filter_map(|fig| match fig {
            feather_draw::Figura::Retangulo { estilo, .. } => estilo.preenchimento,
            _ => None,
        }).collect();
        assert_eq!(cores[0], Cor::rgb(2, 2, 2));
        assert_eq!(cores[3], Cor::rgb(3, 3, 3));
    }

    #[test]
    fn pizza_filtra_fatias_e_refaz_proporcoes() {
        let mut g = grafico(Tipo::Pizza);
        g.series.truncate(1);
        g.series[0].valores = vec![10.0, 10.0, 20.0];
        let mut f = FiltroLegenda::novo();
        let cheia = g.fatias_pizza_filtradas(600.0, 400.0, &f).unwrap();
        assert!((cheia[2].proporcao - 0.5).abs() < 1e-9);
        f.alternar(2, 3);
        let sem = g.fatias_pizza_filtradas(600.0, 400.0, &f).unwrap();
        assert_eq!(sem.len(), 2);
        assert_eq!(sem.iter().map(|s| s.indice).collect::<Vec<_>>(), vec![0, 1]);
        assert!((sem[0].proporcao - 0.5).abs() < 1e-9);
        assert_eq!(g.series[0].valores.len(), 3);
        // tudo escondido por fora da API segura devolve erro
        let mut tudo = FiltroLegenda::novo();
        tudo.ocultos.extend([0, 1, 2]);
        assert_eq!(g.filtrado(&tudo).unwrap_err(), ErroGrafico::SemDados);
    }

    #[test]
    fn legenda_marca_escondido_esmaecido_e_acerta_o_item() {
        let g = grafico(Tipo::Linhas);
        let cfg = TipografiaGrafico::default();
        let ap = AparenciaGrafico::default();
        let mut f = FiltroLegenda::novo();
        f.alternar(1, 3);
        let itens = g.itens_legenda(600.0, 400.0, &cfg, &ap, &f);
        assert_eq!(itens.len(), 3);
        assert_eq!(itens.iter().map(|i| i.escondido).collect::<Vec<_>>(), vec![false, true, false]);
        for item in &itens {
            let centro = Ponto::novo(
                item.caixa.x + item.caixa.largura / 2.0,
                item.caixa.y + item.caixa.altura / 2.0);
            assert_eq!(g.hit_test_legenda(600.0, 400.0, &cfg, &ap, &f, centro), Some(item.indice));
        }
        assert_eq!(g.hit_test_legenda(600.0, 400.0, &cfg, &ap, &f, Ponto::novo(300.0, 100.0)), None);
        // o texto do item escondido é desenhado com cor diferente
        let anotada = g.cena_anotada_estilizada(600.0, 400.0, &cfg, &ap, &f).unwrap();
        let cor_de = |nome: &str| anotada.rotulos.iter().find(|r| r.texto == nome).unwrap().cor;
        assert_ne!(cor_de("Grande"), cor_de("Pequena"));
        assert_eq!(cor_de("Pequena"), cor_de("Media"));
        // e a linha escondida não é desenhada
        let sem = g.cena_filtrada(600.0, 400.0, &ap, &f).unwrap();
        let com = g.cena_filtrada(600.0, 400.0, &ap, &FiltroLegenda::novo()).unwrap();
        assert_eq!(com.figuras.len() - sem.figuras.len(), 1);
    }

    #[test]
    fn sem_filtro_a_anotada_estilizada_so_muda_a_legenda_da_pizza() {
        let g = grafico(Tipo::Colunas);
        let cfg = TipografiaGrafico::default();
        let velha = g.cena_anotada(640.0, 420.0, &cfg).unwrap();
        let nova = g.cena_anotada_estilizada(
            640.0, 420.0, &cfg, &AparenciaGrafico::default(), &FiltroLegenda::novo()).unwrap();
        assert_eq!(velha, nova);
    }
}
