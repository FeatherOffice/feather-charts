//! Regressão do warning private_interfaces: os módulos de tipos são públicos
//! e precisam aceitar Area pela API pública, inclusive de outro crate.
//! cargo test --test visibilidade_area -j 2

use feather_charts::{
    motor::{self, tipos, Area, Entrada, TipoGrafico},
    Serie,
};
use feather_draw::{Cena, Cor};

fn entrada() -> Entrada {
    Entrada {
        titulo: "Valores".into(),
        categorias: vec!["A".into(), "B".into()],
        series: vec![Serie {
            nome: "Série 1".into(),
            valores: vec![12.0, 18.0],
            cor: Cor::rgb(70, 115, 205),
        }],
        x_values: None,
        z_values: None,
    }
}

#[test]
fn area_publica_pode_ser_passada_a_um_renderizador_externo() {
    let area = Area::nova(480.0, 300.0);
    assert!(area.w > 0.0 && area.h > 0.0);
    let mut cena = Cena::nova(480.0, 300.0);
    // A chamada é compilada como consumidor externo do crate: se Area
    // voltar a ser pub(super), o teste falha na compilação.
    tipos::colunas::desenhar(&mut cena, &entrada(), area);
    tipos::barras::desenhar(&mut cena, &entrada(), area);
}

#[test]
fn fachada_publica_continua_renderizando_apos_a_correcao() {
    let cena = motor::renderizar(TipoGrafico::Colunas, &entrada(), 480.0, 300.0);
    assert!(cena.is_ok());
    let svg = motor::renderizar_svg(TipoGrafico::Barras, &entrada(), 480.0, 300.0)
        .expect("o motor deve exportar sem alterar desenho ou agregação");
    assert!(svg.contains("<svg"));
}
