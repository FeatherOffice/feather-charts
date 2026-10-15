//! Estilos visuais: padrão idêntico ao desenho anterior e cada efeito opt-in.
use feather_charts::motor::{renderizar, renderizar_com, Entrada, TipoGrafico};
use feather_charts::{
    temas, AparenciaGrafico, Arredondamento, CantosArredondados, FiltroLegenda, GradienteArea,
    Grafico, InteracaoGrafico, Neon, RaioBarra, Serie, Suavizacao, Tema, Tipo, TipografiaGrafico,
};
use feather_draw::{exportar_svg, Cena, Comando, Cor, Figura, Ponto, Preenchimento};

fn grafico(tipo: Tipo, series: Vec<Vec<f64>>) -> Grafico {
    let n = series[0].len();
    Grafico {
        titulo: "Estilos".into(),
        tipo,
        categorias: (0..n).map(|i| format!("{}", i + 1)).collect(),
        series: series
            .into_iter()
            .enumerate()
            .map(|(i, valores)| Serie {
                nome: format!("S{}", i + 1),
                valores,
                cor: [Cor::rgb(200, 40, 40), Cor::rgb(40, 40, 200), Cor::rgb(40, 160, 40)][i % 3],
            })
            .collect(),
    }
}

fn entrada(series: Vec<Vec<f64>>) -> Entrada {
    let n = series[0].len();
    Entrada {
        titulo: "Motor".into(),
        categorias: (0..n).map(|i| format!("{}", i + 1)).collect(),
        series: series
            .into_iter()
            .enumerate()
            .map(|(i, valores)| Serie {
                nome: format!("S{}", i + 1),
                valores,
                cor: [Cor::rgb(200, 40, 40), Cor::rgb(40, 40, 200), Cor::rgb(40, 160, 40)][i % 3],
            })
            .collect(),
        x_values: Some((0..n).map(|i| i as f64).collect()),
        z_values: Some((0..n).map(|i| (i % 3) as f64 + 1.0).collect()),
    }
}

fn caminhos(cena: &Cena) -> Vec<&feather_draw::Caminho> {
    cena.figuras.iter().filter_map(|f| match f {
        Figura::Caminho { caminho, .. } => Some(caminho),
        _ => None,
    }).collect()
}
fn n_cubicas(c: &feather_draw::Caminho) -> usize {
    c.comandos.iter().filter(|c| matches!(c, Comando::Cubica { .. })).count()
}

#[test]
fn aparencia_padrao_nao_muda_nenhuma_cena() {
    for tipo in [Tipo::Colunas, Tipo::Barras, Tipo::Linhas, Tipo::Area, Tipo::Degraus, Tipo::Pizza, Tipo::Dispersao] {
        let g = grafico(tipo, vec![vec![1.0, 4.0, 2.0, 5.0]]);
        let a = g.cena(640.0, 400.0).unwrap();
        let b = g.cena_estilizada(640.0, 400.0, &AparenciaGrafico::default()).unwrap();
        assert_eq!(a, b, "{tipo:?}");
        assert_eq!(exportar_svg(&a).unwrap(), exportar_svg(&b).unwrap());
        let c = g.cena_filtrada(640.0, 400.0, &AparenciaGrafico::default(), &FiltroLegenda::default()).unwrap();
        assert_eq!(a, c);
    }
    let e = entrada(vec![vec![1.0, 4.0, 2.0, 5.0], vec![2.0, 1.0, 3.0, 1.0], vec![1.0, 1.0, 1.0, 2.0]]);
    for tipo in TipoGrafico::TODOS {
        let mut d = e.clone();
        if tipo == TipoGrafico::Candlestick {
            d.series.push(Serie { nome: "x".into(), valores: vec![0.5; 4], cor: Cor::rgb(1, 2, 3) });
            d.series = vec![
                Serie { nome: "A".into(), valores: vec![2., 3., 4., 5.], cor: Cor::rgb(1, 1, 1) },
                Serie { nome: "B".into(), valores: vec![5., 6., 7., 8.], cor: Cor::rgb(1, 1, 1) },
                Serie { nome: "C".into(), valores: vec![0., 1., 2., 3.], cor: Cor::rgb(1, 1, 1) },
                Serie { nome: "D".into(), valores: vec![2., 3., 4., 5.], cor: Cor::rgb(1, 1, 1) },
            ];
        }
        let a = renderizar(tipo, &d, 480.0, 300.0);
        let b = renderizar_com(tipo, &d, 480.0, 300.0, &AparenciaGrafico::default());
        assert_eq!(a, b, "{}", tipo.id());
    }
}

#[test]
fn colunas_arredondadas_so_na_ponta_inclusive_negativa() {
    let g = grafico(Tipo::Colunas, vec![vec![5.0, -3.0, 4.0]]);
    let base = g.cena(600.0, 400.0).unwrap();
    let areas: Vec<_> = base.figuras.iter().filter_map(|f| match f {
        Figura::Retangulo { area, .. } => Some(*area),
        _ => None,
    }).collect();
    assert_eq!(areas.len(), 3);
    let ap = AparenciaGrafico {
        arredondamento: Some(Arredondamento { raio: RaioBarra::Pixels(8.0), onde: CantosArredondados::Topo }),
        ..Default::default()
    };
    let cena = g.cena_estilizada(600.0, 400.0, &ap).unwrap();
    let cs = caminhos(&cena);
    assert_eq!(cs.len(), 3);
    for c in &cs {
        assert_eq!(n_cubicas(c), 4); // sempre 4 cantos no caminho (os retos têm raio zero)
    }
    // positiva: começa 8 px depois da quina esquerda, no topo
    match cs[0].comandos[0] {
        Comando::Mover(p) => {
            assert!((p.x - (areas[0].x + 8.0)).abs() < 1e-3);
            assert!((p.y - areas[0].y).abs() < 1e-3);
        }
        _ => panic!(),
    }
    // negativa: a ponta é embaixo; o topo (junto do zero) fica reto
    match cs[1].comandos[0] {
        Comando::Mover(p) => assert!((p.x - areas[1].x).abs() < 1e-3),
        _ => panic!(),
    }
    // fim da 2a cúbica (canto inferior direito) começa 8 px antes da quina
    let fins: Vec<Ponto> = cs[1].comandos.iter().filter_map(|c| match c {
        Comando::Cubica { fim, .. } => Some(*fim),
        _ => None,
    }).collect();
    assert!((fins[1].x - (areas[1].x + areas[1].largura - 8.0)).abs() < 1e-3);
    assert!((fins[1].y - (areas[1].y + areas[1].altura)).abs() < 1e-3);
    // dados e determinismo
    assert_eq!(g.series[0].valores, [5.0, -3.0, 4.0]);
    assert_eq!(cena, g.cena_estilizada(600.0, 400.0, &ap).unwrap());
    assert!(exportar_svg(&cena).unwrap().contains(" C"));
}

#[test]
fn hit_test_e_rotulos_seguem_a_barra_arredondada_e_o_fundo() {
    let g = grafico(Tipo::Colunas, vec![vec![5.0, 3.0], vec![2.0, 6.0]]);
    let ap = AparenciaGrafico {
        fundo: Some(Cor::rgb(10, 10, 10)),
        arredondamento: Some(Arredondamento { raio: RaioBarra::Fracao(0.5), onde: CantosArredondados::Todos }),
        neon: Some(Neon::default()),
        ..Default::default()
    };
    let sem = AparenciaGrafico::default();
    // o centro de cada barra da cena simples acerta a mesma amostra com os efeitos ligados
    let base = g.cena(600.0, 400.0).unwrap();
    let centros: Vec<_> = base.figuras.iter().filter_map(|f| match f {
        Figura::Retangulo { area, .. } => Some(Ponto::novo(area.x + area.largura / 2.0, area.y + area.altura / 2.0)),
        _ => None,
    }).collect();
    for (k, c) in centros.iter().enumerate() {
        let a = g.hit_test_dados(600.0, 400.0, *c, 0.5, &sem).unwrap().unwrap();
        let b = g.hit_test_dados(600.0, 400.0, *c, 0.5, &ap).unwrap().unwrap();
        assert_eq!(a.alvo, b.alvo, "barra {k}");
    }
    let mut p = g.clone();
    p.tipo = Tipo::Pizza;
    p.series.truncate(1);
    let pontos = (0..1).map(|_| Ponto::novo(300.0 + 40.0, 200.0)).collect::<Vec<_>>();
    assert_eq!(
        p.hit_test_dados(600.0, 400.0, pontos[0], 0.0, &sem).unwrap().map(|h| h.alvo),
        p.hit_test_dados(600.0, 400.0, pontos[0], 0.0, &ap).unwrap().map(|h| h.alvo));
}

#[test]
fn empilhadas_arredondam_so_o_segmento_da_ponta() {
    let e = entrada(vec![vec![3.0, 2.0], vec![4.0, 1.0], vec![2.0, 3.0]]);
    let ap = AparenciaGrafico {
        arredondamento: Some(Arredondamento { raio: RaioBarra::Pixels(6.0), onde: CantosArredondados::Topo }),
        ..Default::default()
    };
    let cena = renderizar_com(TipoGrafico::ColunasEmpilhadas, &e, 600.0, 400.0, &ap).unwrap();
    let rets = cena.figuras.iter().filter(|f| matches!(f, Figura::Retangulo { .. })).count();
    let cams = caminhos(&cena).len();
    assert_eq!((rets, cams), (4, 2), "2 categorias: 2 segmentos retos + 1 ponta arredondada cada");
    // a ponta é a última figura de cada categoria (série 3 sobre as demais)
    assert!(matches!(cena.figuras[2], Figura::Caminho { .. }));
    assert!(matches!(cena.figuras[5], Figura::Caminho { .. }));
    // todos os cantos
    let todos = AparenciaGrafico {
        arredondamento: Some(Arredondamento { raio: RaioBarra::Pixels(6.0), onde: CantosArredondados::Todos }),
        ..Default::default()
    };
    let c2 = renderizar_com(TipoGrafico::BarrasEmpilhadas, &e, 600.0, 400.0, &todos).unwrap();
    assert_eq!(caminhos(&c2).len(), 6);
    // pilha com negativo: a ponta negativa é a de baixo
    let neg = entrada(vec![vec![3.0], vec![-2.0], vec![-1.0]]);
    let c3 = renderizar_com(TipoGrafico::ColunasEmpilhadas, &neg, 600.0, 400.0, &ap).unwrap();
    assert_eq!(caminhos(&c3).len(), 2); // ponta positiva (única positiva) + ponta negativa (3a série)
    assert!(matches!(c3.figuras[1], Figura::Retangulo { .. })); // 2a série: meio da pilha negativa
}

#[test]
fn linha_suavizada_por_serie_nao_passa_dos_valores() {
    let g = grafico(Tipo::Linhas, vec![vec![0.0, 10.0, 10.0, 0.0, 6.0, 6.0, 30.0], vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]]);
    let ap = AparenciaGrafico {
        suavizacao: Suavizacao::Series(vec![0]),
        ..Default::default()
    };
    let cena = g.cena_estilizada(700.0, 400.0, &ap).unwrap();
    let cs = caminhos(&cena);
    assert_eq!(cs.len(), 1, "só a série 0 virou curva");
    assert_eq!(n_cubicas(cs[0]), 6);
    assert!(cena.figuras.iter().any(|f| matches!(f, Figura::Polilinha { fechada: false, .. })));
    // amostra cada cúbica: y nunca sai do intervalo dos extremos do segmento
    let mut atual = Ponto::default();
    for cmd in &cs[0].comandos {
        match *cmd {
            Comando::Mover(p) => atual = p,
            Comando::Cubica { controle1, controle2, fim } => {
                let (lo, hi) = (atual.y.min(fim.y), atual.y.max(fim.y));
                for k in 0..=40 {
                    let t = k as f32 / 40.0;
                    let u = 1.0 - t;
                    let y = u * u * u * atual.y + 3.0 * u * u * t * controle1.y
                        + 3.0 * u * t * t * controle2.y + t * t * t * fim.y;
                    assert!(y >= lo - 1e-2 && y <= hi + 1e-2);
                }
                atual = fim;
            }
            _ => {}
        }
    }
    // todas as séries
    let todas = AparenciaGrafico { suavizacao: Suavizacao::Todas, ..Default::default() };
    assert_eq!(caminhos(&g.cena_estilizada(700.0, 400.0, &todas).unwrap()).len(), 2);
    // área também
    let mut a = g.clone();
    a.tipo = Tipo::Area;
    let ca = a.cena_estilizada(700.0, 400.0, &todas).unwrap();
    assert!(caminhos(&ca).iter().any(|c| n_cubicas(c) == 6));
    // determinismo
    assert_eq!(ca, a.cena_estilizada(700.0, 400.0, &todas).unwrap());
}

#[test]
fn area_com_gradiente_vai_da_serie_ate_transparente() {
    let mut g = grafico(Tipo::Area, vec![vec![2.0, 8.0, 5.0, 9.0]]);
    let ap = AparenciaGrafico {
        gradiente_area: Some(GradienteArea { opacidade_inicial: 0.7, opacidade_final: 0.0, faixas: 0 }),
        ..Default::default()
    };
    let cena = g.cena_estilizada(600.0, 400.0, &ap).unwrap();
    let eixos = g.eixos(600.0, 400.0).unwrap().unwrap();
    let grad = cena.figuras.iter().find_map(|f| match f {
        Figura::Caminho { estilo, .. } => match &estilo.preenchimento {
            Some(Preenchimento::Linear { de, ate, paradas }) => Some((*de, *ate, paradas.clone())),
            _ => None,
        },
        _ => None,
    }).expect("preenchimento linear");
    assert!((grad.1.y - eixos.zero).abs() < 1e-3, "termina na linha de base");
    assert!(grad.0.y < grad.1.y, "começa no topo da série");
    assert_eq!((grad.2[0].cor.a, grad.2[1].cor.a), (179, 0));
    let svg = exportar_svg(&cena).unwrap();
    assert!(svg.contains("linearGradient"));
    let pdf = g.exportar_pdf_estilizado(600.0, 400.0, &TipografiaGrafico::default(), &ap, &FiltroLegenda::default());
    assert!(pdf.is_ok(), "{pdf:?}");
    // faixas: sem gradiente linear, recortadas, alfa decrescente de cima para baixo
    let fx = AparenciaGrafico {
        gradiente_area: Some(GradienteArea { faixas: 8, ..ap.gradiente_area.unwrap() }),
        ..Default::default()
    };
    let cf = g.cena_estilizada(600.0, 400.0, &fx).unwrap();
    let alfas: Vec<u8> = cf.figuras.iter().filter_map(|f| match f {
        Figura::Recorte { figura, .. } => match &**figura {
            Figura::Retangulo { estilo, .. } => estilo.preenchimento.map(|c| c.a),
            _ => None,
        },
        _ => None,
    }).collect();
    assert_eq!(alfas.len(), 8);
    assert!(alfas.windows(2).all(|w| w[0] > w[1]));
    assert!(exportar_svg(&cf).is_ok());
    assert!(feather_draw::exportar_pdf(&cf).is_ok());
    // valores negativos: a base continua o zero
    g.series[0].valores = vec![-4.0, 3.0, -1.0, 2.0];
    assert!(g.cena_estilizada(600.0, 400.0, &ap).is_ok());
}

#[test]
fn neon_desenha_brilho_antes_do_traco() {
    let g = grafico(Tipo::Linhas, vec![vec![1.0, 3.0, 2.0]]);
    let neon = Neon { intensidade: 0.9, espessura: 9.0, camadas: 3 };
    let ap = AparenciaGrafico { neon: Some(neon), ..Default::default() };
    let base = g.cena_estilizada(500.0, 300.0, &AparenciaGrafico::default()).unwrap();
    let cena = g.cena_estilizada(500.0, 300.0, &ap).unwrap();
    assert_eq!(cena.figuras.len(), base.figuras.len() + 3);
    let larguras: Vec<f32> = cena.figuras.iter().filter_map(|f| match f {
        Figura::Caminho { estilo, .. } => estilo.stroke.as_ref().map(|s| s.largura),
        _ => None,
    }).collect();
    assert_eq!(larguras.len(), 4);
    assert!(larguras[0] > larguras[1] && larguras[1] > larguras[2] && larguras[2] > larguras[3]);
    assert_eq!(larguras[3], 2.0, "o traço normal é o último");
    // colunas e áreas também brilham
    let mut c = g.clone();
    c.tipo = Tipo::Colunas;
    let cc = c.cena_estilizada(500.0, 300.0, &ap).unwrap();
    assert!(cc.figuras.len() >= c.cena(500.0, 300.0).unwrap().figuras.len() + 9);
    // neon inválido é recusado
    let ruim = AparenciaGrafico { neon: Some(Neon { camadas: 0, ..neon }), ..Default::default() };
    assert!(g.cena_estilizada(500.0, 300.0, &ruim).is_err());
}

#[test]
fn cada_tema_gera_svg_com_fundo_e_cores_da_paleta() {
    let g = grafico(Tipo::Colunas, vec![vec![1.0, 3.0, 2.0], vec![2.0, 1.0, 4.0]]);
    assert_eq!(temas().len(), 14);
    for t in temas() {
        let ap = t.aparencia();
        let svg = g.exportar_svg_estilizado(600.0, 360.0, &TipografiaGrafico::default(), &ap, &FiltroLegenda::default()).unwrap();
        assert!(svg.starts_with("<svg"));
        let hex = |c: Cor| format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b);
        assert!(svg.to_lowercase().contains(&hex(t.fundo)), "{} sem fundo", t.id);
        assert!(svg.to_lowercase().contains(&hex(t.paleta[0])), "{} sem a 1a cor", t.id);
        assert!(svg.to_lowercase().contains(&hex(t.texto)), "{} sem cor de texto", t.id);
    }
    let mut g2 = g.clone();
    let mut ap = AparenciaGrafico::default();
    feather_charts::aplicar_tema(Tema::pelo_id("floresta").unwrap(), &mut g2, &mut ap);
    assert_eq!(g2.series[0].cor, Tema::pelo_id("floresta").unwrap().paleta[0]);
    assert_eq!(g.series[0].valores, g2.series[0].valores);
}

#[test]
fn clique_na_legenda_esconde_so_e_limpa_selecao() {
    let g = grafico(Tipo::Colunas, vec![vec![1.0, 2.0], vec![10.0, 20.0], vec![5.0, 4.0]]);
    let cfg = TipografiaGrafico::default();
    let ap = AparenciaGrafico::default();
    let mut st = InteracaoGrafico::default();
    let itens = g.itens_legenda(600.0, 400.0, &cfg, &ap, &st.filtro);
    let centro = |i: usize| Ponto::novo(
        itens[i].caixa.x + itens[i].caixa.largura / 2.0, itens[i].caixa.y + itens[i].caixa.altura / 2.0);
    // seleciona algo da série 1 e esconde essa série
    st.selecionado = Some(feather_charts::AlvoGrafico {
        serie: 1, categoria: 0, nome_serie: "S2".into(), rotulo_categoria: "1".into(), valor: 10.0 });
    assert_eq!(st.clicar_legenda(&g, 600.0, 400.0, &cfg, &ap, centro(1), false).unwrap(), Some(1));
    assert!(st.filtro.esta_oculto(1));
    assert!(st.selecionado.is_none());
    // ponto fora da legenda não acerta nada
    assert_eq!(st.clicar_legenda(&g, 600.0, 400.0, &cfg, &ap, Ponto::novo(1.0, 1.0), false).unwrap(), None);
    // hit-test nas barras ignora a escondida e devolve índice original
    let sem = g.cena_filtrada(600.0, 400.0, &ap, &st.filtro).unwrap();
    assert!(!sem.figuras.is_empty());
    // Ctrl+clique: só a série 2
    assert_eq!(st.clicar_legenda(&g, 600.0, 400.0, &cfg, &ap, centro(2), true).unwrap(), Some(2));
    assert_eq!(st.filtro.ocultos().len(), 2);
    assert!(!st.filtro.esta_oculto(2));
    st.limpar_filtro();
    assert!(!st.filtro.filtrando());
}
