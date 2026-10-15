//! Demonstra a migração dos tipos Área e Linha em degraus sem dependência de GUI.
//! Execução:
//! cargo run \
//!   --example migracao_area_degraus -- area > /tmp/feather_charts_area.svg
//! cargo run \
//!   --example migracao_area_degraus -- degraus > /tmp/feather_charts_degraus.svg
use feather_charts::{Grafico, Serie, Tipo};
use feather_draw::Cor;

fn grafico(tipo: Tipo) -> Grafico {
    Grafico {
        titulo: match tipo {
            Tipo::Area => "Áreas — produção por trimestre".into(),
            Tipo::Degraus => "Degraus — meta por trimestre".into(),
            _ => unreachable!("exemplo aceita somente Área e Degraus"),
        },
        tipo,
        categorias: ["Jan", "Fev", "Mar", "Abr"].map(str::to_owned).to_vec(),
        series: vec![
            Serie {
                nome: "Produção".into(),
                valores: vec![20.0, -10.0, 35.0, 28.0],
                cor: Cor::rgb(43, 122, 120),
            },
            Serie {
                nome: "Planejado".into(),
                valores: vec![15.0, 25.0, 18.0, 32.0],
                cor: Cor::rgb(232, 113, 10),
            },
        ],
    }
}

fn main() {
    let tipo = match std::env::args().nth(1).as_deref() {
        Some("area") => Tipo::Area,
        Some("degraus") => Tipo::Degraus,
        _ => {
            eprintln!("Uso: migracao_area_degraus area|degraus > arquivo.svg");
            std::process::exit(2);
        }
    };
    let modelo = grafico(tipo);
    let svg = modelo.exportar_svg(800.0, 450.0)
        .expect("modelo validado e dimensões de SVG válidas");
    print!("{svg}");
    eprintln!(
        "tipo={tipo:?}, categorias={}, series={}, amostras_preservadas={}",
        modelo.categorias.len(),
        modelo.series.len(),
        modelo.series.iter().all(|s| s.valores.len() == modelo.categorias.len())
    );
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn ambos_exportam_e_mantem_dados_sem_pre_agregacao() {
        for tipo in [Tipo::Area, Tipo::Degraus] {
            let m = grafico(tipo);
            let original = m.clone();
            assert!(m.exportar_svg(800.0, 450.0).unwrap().starts_with("<svg"));
            assert_eq!(m, original);
            assert_eq!(m.series[0].valores[1], -10.0);
        }
    }
}
