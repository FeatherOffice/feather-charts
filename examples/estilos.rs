//! Galeria dos estilos visuais: colunas arredondadas, linha suavizada, área com
//! gradiente, as 14 temáticas, neon e filtro pela legenda.
//!
//! Uso: `cargo run --example estilos -- <pasta>`; grava um SVG por gráfico e um
//! `index.html` com os SVGs embutidos (sem `<img>`, para abrir em qualquer lugar).
use feather_charts::motor::{renderizar_com, Entrada, TipoGrafico};
use feather_charts::{
    temas, AparenciaGrafico, Arredondamento, CantosArredondados, FiltroLegenda, GradienteArea,
    Grafico, Neon, RaioBarra, Serie, Suavizacao, Tema, Tipo, TipografiaGrafico,
};
use feather_draw::{exportar_svg, Cor};
use std::fs;
use std::path::PathBuf;

const MESES: [&str; 8] = ["Jan", "Fev", "Mar", "Abr", "Mai", "Jun", "Jul", "Ago"];

fn grafico(tipo: Tipo, titulo: &str, series: &[(&str, [f64; 8], Cor)]) -> Grafico {
    Grafico {
        titulo: titulo.into(),
        tipo,
        categorias: MESES.iter().map(|m| (*m).into()).collect(),
        series: series
            .iter()
            .map(|(nome, v, cor)| Serie { nome: (*nome).into(), valores: v.to_vec(), cor: *cor })
            .collect(),
    }
}

fn vendas(tipo: Tipo, titulo: &str) -> Grafico {
    grafico(tipo, titulo, &[
        ("Norte", [12.0, 18.0, 15.0, 26.0, 22.0, 31.0, 28.0, 36.0], Cor::rgb(31, 119, 180)),
        ("Sul", [8.0, 11.0, 19.0, 14.0, 24.0, 20.0, 27.0, 25.0], Cor::rgb(255, 127, 14)),
        ("Leste", [5.0, 9.0, 7.0, 15.0, 13.0, 18.0, 17.0, 22.0], Cor::rgb(44, 160, 44)),
    ])
}

/// Numera os ids do gradiente/recorte de cada SVG para que vários SVGs inline
/// na mesma página não disputem o mesmo `id`.
fn isolar_ids(svg: &str, n: usize) -> String {
    let mut s = svg.to_string();
    for prefixo in ["grad", "clip"] {
        s = s.replace(&format!("id=\"{prefixo}"), &format!("id=\"g{n}{prefixo}"));
        s = s.replace(&format!("url(#{prefixo}"), &format!("url(#g{n}{prefixo}"));
    }
    s
}

fn main() {
    let pasta: PathBuf = std::env::args().nth(1).expect("uso: estilos <pasta de saida>").into();
    fs::create_dir_all(&pasta).unwrap();
    let cfg = TipografiaGrafico::default();
    let sem_filtro = FiltroLegenda::default();
    let (l, a) = (560.0_f32, 340.0_f32);
    let mut galeria: Vec<(String, String, String)> = Vec::new(); // (secao, titulo, svg)
    let mut guardar = |secao: &str, id: &str, titulo: &str, svg: String| {
        fs::write(pasta.join(format!("{id}.svg")), &svg).unwrap();
        galeria.push((secao.into(), titulo.into(), svg));
    };

    // 1. Colunas arredondadas
    let ap = AparenciaGrafico {
        arredondamento: Some(Arredondamento { raio: RaioBarra::Fracao(0.5), onde: CantosArredondados::Topo }),
        cor_grade: Some(Cor::rgb(225, 228, 232)),
        ..Default::default()
    };
    let g = vendas(Tipo::Colunas, "Colunas arredondadas (só o topo)");
    guardar("Barras e colunas arredondadas", "colunas-arredondadas", "Raio de 50% da largura, só o topo",
        g.exportar_svg_estilizado(l, a, &cfg, &ap, &sem_filtro).unwrap());
    let mut g2 = vendas(Tipo::Colunas, "Colunas com negativos, 8 px, topo e base");
    g2.series[1].valores = [-8.0, 11.0, -19.0, 14.0, -24.0, 20.0, -27.0, 25.0].to_vec();
    let ap2 = AparenciaGrafico {
        arredondamento: Some(Arredondamento { raio: RaioBarra::Pixels(8.0), onde: CantosArredondados::TopoEBase }),
        cor_grade: Some(Cor::rgb(225, 228, 232)),
        ..Default::default()
    };
    guardar("Barras e colunas arredondadas", "colunas-negativas", "Valor negativo: a ponta é a de baixo",
        g2.exportar_svg_estilizado(l, a, &cfg, &ap2, &sem_filtro).unwrap());
    let mut gb = vendas(Tipo::Barras, "Barras horizontais, todos os cantos");
    gb.series.truncate(2);
    let ap3 = AparenciaGrafico {
        arredondamento: Some(Arredondamento { raio: RaioBarra::Fracao(0.4), onde: CantosArredondados::Todos }),
        ..Default::default()
    };
    guardar("Barras e colunas arredondadas", "barras-arredondadas", "Raio de 40% da altura, todos os cantos",
        gb.exportar_svg_estilizado(l, a, &cfg, &ap3, &sem_filtro).unwrap());
    let mut e = Entrada {
        titulo: "Empilhadas".into(),
        categorias: MESES.iter().map(|m| (*m).into()).collect(),
        series: vendas(Tipo::Colunas, "").series,
        x_values: None,
        z_values: None,
    };
    e.series.iter_mut().for_each(|s| s.valores.truncate(8));
    let emp = renderizar_com(TipoGrafico::ColunasEmpilhadas, &e, l, a, &AparenciaGrafico {
        arredondamento: Some(Arredondamento { raio: RaioBarra::Pixels(9.0), onde: CantosArredondados::Topo }),
        ..Default::default()
    }).unwrap();
    guardar("Barras e colunas arredondadas", "empilhadas-arredondadas",
        "Empilhadas: só o segmento da ponta arredonda (cena do motor, sem textos)",
        exportar_svg(&emp).unwrap());

    // 2. Linha suavizada
    let neutro = AparenciaGrafico { cor_grade: Some(Cor::rgb(225, 228, 232)), ..Default::default() };
    let mut lin = vendas(Tipo::Linhas, "Linha reta");
    lin.series.truncate(2);
    guardar("Linha suavizada", "linha-reta", "Sem suavização",
        lin.exportar_svg_estilizado(l, a, &cfg, &neutro, &sem_filtro).unwrap());
    lin.titulo = "Linha suavizada (monótona)".into();
    let suave = AparenciaGrafico { suavizacao: Suavizacao::Todas, ..neutro.clone() };
    guardar("Linha suavizada", "linha-suave", "Curva que não ultrapassa os valores reais",
        lin.exportar_svg_estilizado(l, a, &cfg, &suave, &sem_filtro).unwrap());
    lin.titulo = "Só a série Norte suavizada".into();
    let uma = AparenciaGrafico { suavizacao: Suavizacao::Series(vec![0]), ..neutro.clone() };
    guardar("Linha suavizada", "linha-uma-serie", "Suavização por série",
        lin.exportar_svg_estilizado(l, a, &cfg, &uma, &sem_filtro).unwrap());

    // 3. Área com gradiente
    let mut ar = vendas(Tipo::Area, "Área com gradiente");
    ar.series.truncate(2);
    let grad = AparenciaGrafico {
        gradiente_area: Some(GradienteArea { opacidade_inicial: 0.75, opacidade_final: 0.0, faixas: 0 }),
        suavizacao: Suavizacao::Todas,
        ..neutro.clone()
    };
    guardar("Área com gradiente", "area-gradiente", "Cor da série até transparente na base (suavizada)",
        ar.exportar_svg_estilizado(l, a, &cfg, &grad, &sem_filtro).unwrap());
    ar.titulo = "Área com gradiente em faixas".into();
    let faixas = AparenciaGrafico {
        gradiente_area: Some(GradienteArea { faixas: 16, ..grad.gradiente_area.unwrap() }),
        ..neutro.clone()
    };
    guardar("Área com gradiente", "area-faixas", "16 faixas recortadas (a forma que o PDF aceita)",
        ar.exportar_svg_estilizado(l, a, &cfg, &faixas, &sem_filtro).unwrap());

    // 4. Um gráfico com cada tema
    for t in temas() {
        let mut g = vendas(Tipo::Colunas, t.nome);
        g.series.truncate(3);
        let mut ap = t.aparencia();
        ap.arredondamento = Some(Arredondamento { raio: RaioBarra::Pixels(5.0), onde: CantosArredondados::Topo });
        guardar("Temáticas de cores", &format!("tema-{}", t.id), t.nome,
            g.exportar_svg_estilizado(l, a, &cfg, &ap, &sem_filtro).unwrap());
    }

    // 5. Neon
    let neon_t = Tema::pelo_id("neon").unwrap();
    let mut nl = vendas(Tipo::Linhas, "Neon: linhas com brilho");
    nl.series.truncate(3);
    let mut ap = neon_t.aparencia();
    ap.espessura_linha = 2.5;
    ap.suavizacao = Suavizacao::Todas;
    guardar("Neon", "neon-linhas", "Brilho em 3 camadas atrás do traço",
        nl.exportar_svg_estilizado(l, a, &cfg, &ap, &sem_filtro).unwrap());
    let mut nc = vendas(Tipo::Colunas, "Neon: colunas");
    nc.series.truncate(2);
    let mut apc = neon_t.aparencia();
    apc.arredondamento = Some(Arredondamento { raio: RaioBarra::Fracao(0.3), onde: CantosArredondados::Topo });
    apc.neon = Some(Neon { intensidade: 0.9, espessura: 10.0, camadas: 4 });
    guardar("Neon", "neon-colunas", "Brilho no contorno das colunas",
        nc.exportar_svg_estilizado(l, a, &cfg, &apc, &sem_filtro).unwrap());
    let mut na = vendas(Tipo::Area, "Neon: área");
    na.series.truncate(2);
    let mut apa = neon_t.aparencia();
    apa.gradiente_area = Some(GradienteArea { opacidade_inicial: 0.5, opacidade_final: 0.0, faixas: 0 });
    apa.suavizacao = Suavizacao::Todas;
    guardar("Neon", "neon-area", "Área com gradiente e brilho na borda",
        na.exportar_svg_estilizado(l, a, &cfg, &apa, &sem_filtro).unwrap());

    // 6. Filtro pela legenda
    let g = vendas(Tipo::Colunas, "Série Sul escondida pela legenda");
    let mut filtro = FiltroLegenda::default();
    filtro.alternar(1, g.total_itens_legenda());
    let ap = Tema::pelo_id("corporativo").unwrap().aparencia();
    guardar("Filtro pela legenda", "filtro-legenda", "Eixos refeitos sem a série; o item fica esmaecido na legenda",
        g.exportar_svg_estilizado(l, a, &cfg, &ap, &filtro).unwrap());
    let mut pz = grafico(Tipo::Pizza, "Fatia Leste escondida", &[
        ("Receita", [40.0, 30.0, 20.0, 10.0, 0.0, 0.0, 0.0, 0.0], Cor::rgb(0, 0, 0))]);
    pz.categorias = ["Norte", "Sul", "Leste", "Oeste", "-", "-", "-", "-"].map(String::from).to_vec();
    pz.series[0].valores.truncate(4);
    pz.categorias.truncate(4);
    let mut f2 = FiltroLegenda::default();
    f2.alternar(2, pz.total_itens_legenda());
    guardar("Filtro pela legenda", "filtro-pizza", "Pizza: proporções refeitas com o total visível",
        pz.exportar_svg_estilizado(l, a, &cfg, &AparenciaGrafico::default(), &f2).unwrap());

    // index.html com SVG embutido
    let mut html = String::from("<!doctype html><html lang=\"pt-BR\"><head><meta charset=\"utf-8\">\
<title>feather-charts: estilos visuais</title><style>\
body{font-family:sans-serif;margin:24px;background:#f4f5f7;color:#1f2937}\
h1{font-size:22px}h2{font-size:17px;margin:28px 0 8px;border-bottom:1px solid #cbd0d8;padding-bottom:4px}\
.grade{display:flex;flex-wrap:wrap;gap:14px}\
figure{margin:0;background:#fff;border:1px solid #d6dae1;border-radius:6px;padding:8px}\
figure svg{display:block;width:420px;height:auto}\
figcaption{font-size:12px;color:#4b5563;margin-top:4px;max-width:420px}\
</style></head><body><h1>feather-charts: estilos visuais</h1>\n");
    let mut secao_atual = String::new();
    for (n, (secao, titulo, svg)) in galeria.iter().enumerate() {
        if *secao != secao_atual {
            if !secao_atual.is_empty() {
                html.push_str("</div>\n");
            }
            html.push_str(&format!("<h2>{secao}</h2><div class=\"grade\">\n"));
            secao_atual = secao.clone();
        }
        let limpo = svg.trim_start_matches("<?xml version=\"1.0\" encoding=\"UTF-8\"?>").to_string();
        html.push_str(&format!("<figure>{}<figcaption>{titulo}</figcaption></figure>\n", isolar_ids(&limpo, n)));
    }
    html.push_str("</div></body></html>\n");
    fs::write(pasta.join("index.html"), html).unwrap();
    println!("{} gráficos gravados em {}", galeria.len(), pasta.display());
}
