# Custo por resposta (D229)

Medição **estática e fora do runtime** do custo de bytes de uma resposta do `kd ask`. A bancada
roda o binário real sobre um projeto temporário com o corpus sintético determinístico de
`bench/src/quality.rs` (192 notas, 60 consultas, 5 famílias), e o confronto abaixo repete a métrica
sobre uma cópia isolada de corpus real. Mesmo corpus + consulta + flags produzem as mesmas linhas —
três execuções em 2026-11-17 devolveram TSV idêntico.
Nada é escrito em config, nenhum verbo ou byte de envelope muda; nenhum arquivo extra em `.knudge/`.

## Baseline: D161, não um `ask` ideal

O contra-factual **não** é comparado a uma busca hipotética que devolvesse só corpos limpos.
`bytes_out` são os bytes **exatos do stdout** do `kd ask` default, que já aplica D161:

- 1º hit com corpo completo;
- hits 2–5 com corpo truncado a `recall.preview_chars` (default 280);
- hits 6+ no padrão `id|statement|score|why`.

`bytes_counterfactual` é a soma de `len(statement) + len(body)` (em bytes UTF-8) das notas
devolvidas — o que um leitor faria com `cat`, sem frontmatter. `razao = counterfactual / bytes_out`
(`f64`, 3 casas decimais). Família = soma das 12 consultas daquele rótulo.

## Tese medida

> "recall em pipe e corpo sob demanda economizam contexto" — D229 §4.

Se `razao ≫ 1`, o `ask` custa menos bytes que abrir as notas. Se a razão fica perto de 1 (ou
abaixo), a vantagem econômica não aparece neste desenho e a redação da tese precisa ser corrigida.

## Reprodução

```sh
cargo build --release -p knudge-cli
cargo run --release --manifest-path bench/Cargo.toml -- custo --kd target/release/kd
```

A medição abaixo foi reproduzida com `target/release/kd` (**kd 0.6.6**) em 2026-11-17 e é
idêntica à obtida com o `kd` 0.6.4 instalado — nenhuma das mudanças de 0.6.6 toca o `ask`.
Para reproduzir, use os comandos acima (ou defina `KD_BIN`).

## Resultados (TSV)

```tsv
familia	bytes_out	bytes_counterfactual	razao
com-acento	10471	8666	0.828
morfologia	10517	8717	0.829
sem-acento	10523	8740	0.831
sinonimo	10330	8426	0.816
working-set	2156	1772	0.822
```

## Leitura

As razões ficam entre **0.816 e 0.831**: neste corpus sintético o contra-factual é *menor* que a
saída real do `ask`. O motivo é mecânico: os corpos gerados têm ~100 bytes, bem abaixo dos 280 de
`preview_chars`, então D161 não chega a truncar nada — ele paga o corpo integral **mais** as
colunas `id|statement|score|why` de cada hit. O `working-set` tem volumes menores porque o filtro
por âncora devolve menos notas; a razão permanece na mesma faixa.

Ou seja: a tese descrita no §4 do rascunho **não se sustenta como ganho de bytes naquele corpus
sintético**. Ela pode continuar valendo por outros motivos (revelação progressiva, legibilidade),
mas não por economia de bytes medida contra D161 naquele regime de corpos curtos.

## Confronto com corpus real (2026-11-17)

Cópia isolada de `maxima/v7/public-v7/.knudge` (1.535 notas, 3.750 eventos, 45 MB de `.idx`) com
`kd` 0.6.6. Mesma métrica de D229: `bytes_out` = stdout do `kd ask` **sem** `--json`;
`contra-factual` = `statement + body` completos das notas devolvidas (obtidos via `ask --id`).

| consulta | bytes_out | contra-factual | razão |
|---|---:|---:|---:|
| daisyUI 5 is MIT-licensed | 1956 | 2891 | 1,478 |
| up_prod.sh validado fim-a-fim | 2141 | 3177 | 1,484 |
| Core\CoreServiceProvider nao estava registrado | 2088 | 3636 | 1,741 |
| RustFS UID 10001 bind mount | 1985 | 3007 | 1,515 |
| P28 Legislativo completo proposicoes comissoes | 967 | 1003 | 1,037 |

A razão **passa de 1 em todas as consultas**: com corpos longos, D161 trunca a 280 chars e o pipe
passa a custar menos que `cat`. O resultado sintético não generaliza — ele descreve o regime de
corpos curtos, não o contrato do `ask`. A tese econômica vale em corpus real com corpos longos; o
número sintético continua registrado acima, sem maquiagem.

Duas armadilhas do confronto: medir `--json` infla o envelope e inverte a razão (0,31–0,52), mas
D229 mede o pipe; e `ask --id` devolve `statement`/`body`, permitindo o contra-factual exato sem
abrir `notas/` à mão.

## Regra de reversão (D229)

> Se a razão vier perto de 1 em corpus real, a tese econômica está errada e deve ser reescrita na
> documentação — não no tool.

A razão está abaixo de 1 no corpus sintético e acima de 1 no corpus real (5 consultas, tabela
acima): a regra não dispara, mas a redação da tese deve citar o regime — "economia de contexto"
é quantificável em corpos longos, não em corpos de ~100 bytes. Amostra real é pequena e de um
único projeto; não autoriza generalização além do mecanismo de truncamento de D161.

## Não medido

- **[PENDENTE]** A/B secundário `--brief` vs default em `bytes_out`: o subcomando executa
  `--brief` apenas para extrair os ids devolvidos; não há linha TSV que compare os bytes das duas
  saídas.
- **[PENDENTE]** `--full-content`: fora do contrato desta fatia.
