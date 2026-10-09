# ADRs — decisões de arquitetura do knudge

Uma decisão por arquivo, gerada de [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
Cada ADR traz **status**, **categoria**, **contexto mínimo**, a **decisão** e o **impacto obtido**.

Total: **200** decisões.

| # | Título | Status | Categoria |
|---|---|---|---|
| [D01](D01-id-enderecado-por-conteudo.md) | ID endereçado por conteúdo | Aceita | A. Identidade e IDs |
| [D02](D02-prefixo-acompanha-o-type.md) | Prefixo acompanha o type | Aceita | A. Identidade e IDs |
| [D03](D03-formato-fixo.md) | Formato fixo | Aceita | A. Identidade e IDs |
| [D04](D04-ordem-canonica-de-declaracao-do-schema.md) | Ordem canônica de declaração do schema | Aceita | B. Schema e serialização (contrato de bytes) |
| [D05](D05-nao-existe-nota-parcial-opcional.md) | Não existe nota parcial/opcional | Aceita (com ponto de atenção) | B. Schema e serialização (contrato de bytes) |
| [D06](D06-statement-corpo.md) | statement + corpo | Aceita | B. Schema e serialização (contrato de bytes) |
| [D07](D07-utc-com-milissegundos-sufixo-z.md) | UTC, com milissegundos, sufixo Z | Aceita | B. Schema e serialização (contrato de bytes) |
| [D08](D08-escalares-unicode.md) | escalares Unicode | Aceita | B. Schema e serialização (contrato de bytes) |
| [D09](D09-inteiros.md) | Inteiros | Aceita | B. Schema e serialização (contrato de bytes) |
| [D10](D10-raw-utf-8.md) | Raw UTF-8 | Aceita | B. Schema e serialização (contrato de bytes) |
| [D11](D11-arquivo-zero-bytes.md) | arquivo zero bytes | Aceita | B. Schema e serialização (contrato de bytes) |
| [D12](D12-normalizar.md) | Normalizar | Aceita | B. Schema e serialização (contrato de bytes) |
| [D13](D13-ordem-canonica-sempre.md) | Ordem canônica sempre | Aceita | B. Schema e serialização (contrato de bytes) |
| [D14](D14-sem-retrocompatibilidade-sem-aliases.md) | Sem retrocompatibilidade, sem aliases | Aceita | C. Versionamento e migração |
| [D15](D15-on-read-com-defaults.md) | On-read com defaults | Aceita | C. Versionamento e migração |
| [D16](D16-tolerar-com-warning.md) | tolerar com warning | Aceita | C. Versionamento e migração |
| [D17](D17-rejeitar.md) | rejeitar | Aceita (com ponto de atenção) | C. Versionamento e migração |
| [D18](D18-skip-com-warning-orientacao-de-correcao.md) | skip com warning + orientação de correção | Aceita | C. Versionamento e migração |
| [D19](D19-corrige.md) | corrige | Aceita | C. Versionamento e migração |
| [D20](D20-tmp-rename-no-mesmo-diretorio.md) | tmp + rename no mesmo diretório | Aceita | D. Escrita, atomicidade e crash |
| [D21](D21-nota-primeiro-evento-depois.md) | nota primeiro, evento depois | Aceita | D. Escrita, atomicidade e crash |
| [D22](D22-batch.md) | batch | Aceita | D. Escrita, atomicidade e crash |
| [D23](D23-lock-advisory-por-arquivo-alvo.md) | Lock advisory por arquivo-alvo | Aceita | E. Concorrência |
| [D24](D24-rename-sidecar-inode-mtime.md) | rename sidecar + inode/mtime | Aceita | E. Concorrência |
| [D25](D25-ordem-de-aquisicao-de-locks-documentada.md) | Ordem de aquisição de locks documentada | Aceita | E. Concorrência |
| [D26](D26-on-write.md) | on-write | Aceita | E. Concorrência |
| [D27](D27-double-buffer.md) | double-buffer | Aceita | E. Concorrência |
| [D28](D28-deltas-eventos.md) | deltas/eventos | Aceita | E. Concorrência |
| [D29](D29-worktree-principal.md) | worktree principal | Aceita | F. Git, diretórios e persistência |
| [D30](D30-exclusao-absoluta-via.md) | Exclusão absoluta via | Aceita (com ponto de atenção) | F. Git, diretórios e persistência |
| [D31](D31-events.md) | events | Aceita | F. Git, diretórios e persistência |
| [D32](D32-sync-commita-notas-eventos-mensagem-gerada-do.md) | `sync` commita `notas/` + `eventos/`; mensagem gerada do evento; guard | Aceita | F. Git, diretórios e persistência |
| [D33](D33-eventos-anchors.md) | eventos + anchors | Aceita | F. Git, diretórios e persistência |
| [D34](D34-persistinproject.md) | `persist_in_project` | Aceita | F. Git, diretórios e persistência |
| [D35](D35-bm25.md) | BM25 | Aceita | G. Retrieval e ranking |
| [D36](D36-ascii-explicita.md) | ASCII explícita | Aceita | G. Retrieval e ranking |
| [D37](D37-idf-por-campo.md) | IDF por campo | Aceita | G. Retrieval e ranking |
| [D38](D38-boost-por-confirmacao.md) | Boost por confirmação | Aceita | G. Retrieval e ranking |
| [D39](D39-4a-coluna-why.md) | 4ª coluna why | Aceita | G. Retrieval e ranking |
| [D40](D40-orcamento-de-tokens.md) | Orçamento de tokens | Aceita | G. Retrieval e ranking |
| [D41](D41-auto-context-scope.md) | Auto-context-scope | Aceita | G. Retrieval e ranking |
| [D42](D42-configuraveis-desde-ja.md) | configuráveis desde já | Aceita | G. Retrieval e ranking |
| [D43](D43-decay-de-ancoras.md) | Decay de âncoras | Aceita | H. Ciclo de vida e saúde |
| [D44](D44-observational.md) | observational | Aceita | H. Ciclo de vida e saúde |
| [D45](D45-deteccao-de-ciclos.md) | detecção de ciclos | Aceita | H. Ciclo de vida e saúde |
| [D46](D46-integridade-do-grafo.md) | Integridade do grafo | Aceita | H. Ciclo de vida e saúde |
| [D47](D47-propoe.md) | propõe | Aceita | H. Ciclo de vida e saúde |
| [D48](D48-outcomes.md) | outcomes[] | Aceita | H. Ciclo de vida e saúde |
| [D49](D49-arestas-explicitas-primarias-regex-como-sugestao.md) | Arestas explícitas primárias + regex como sugestão | Aceita | I. Grafo e arestas |
| [D50](D50-no-write.md) | no write | Aceita | I. Grafo e arestas |
| [D51](D51-fechado-e-declarativo.md) | fechado e declarativo | Aceita | I. Grafo e arestas |
| [D52](D52-view-derivada.md) | view derivada | Aceita | J. Tarefas e planos |
| [D53](D53-ciclo-de-vida-completo-do-plan.md) | Ciclo de vida completo do plan | Aceita | J. Tarefas e planos |
| [D54](D54-catalogo-executavel.md) | catálogo executável | Aceita | J. Tarefas e planos |
| [D55](D55-evidencia.md) | evidência | Aceita | J. Tarefas e planos |
| [D56](D56-scheduling-separado-de-expiracao.md) | Scheduling separado de expiração | Aceita | J. Tarefas e planos |
| [D57](D57-prime-e-o-protocolo-estatico.md) | prime é o protocolo estático | Aceita | K. Prime, protocolo e hooks |
| [D58](D58-footer-curto-do-prime.md) | footer curto do prime | Aceita | K. Prime, protocolo e hooks |
| [D59](D59-hooks-opcionais.md) | Hooks opcionais | Aceita | K. Prime, protocolo e hooks |
| [D60](D60-marcadores-idempotentes.md) | marcadores idempotentes | Aceita | K. Prime, protocolo e hooks |
| [D61](D61-global.md) | Global | Aceita | L. Config |
| [D62](D62-copia-literal.md) | cópia literal | Aceita | L. Config |
| [D63](D63-ordem-canonica-e-quoting-estaveis.md) | ordem canônica e quoting estáveis | Aceita | L. Config |
| [D64](D64-valida-contra-o-schema.md) | valida contra o schema | Aceita | L. Config |
| [D65](D65-isolamento-por-escopo-tematico.md) | Isolamento por escopo temático | Aceita | M. Arquitetura e distribuição |
| [D66](D66-rust.md) | Rust | Aceita | M. Arquitetura e distribuição |
| [D67](D67-kd.md) | kd | Aceita | M. Arquitetura e distribuição |
| [D68](D68-mcp-cli.md) | MCP + CLI | Aceita | M. Arquitetura e distribuição |
| [D79](D79-provedor-de-embedding-plugavel-via-config.md) | Provedor de embedding plugável via config | Aceita | M. Arquitetura e distribuição |
| [D80](D80-embedding-assincrono-e-lazy-nunca-bloqueia.md) | Embedding assíncrono e lazy; nunca bloqueia | Aceita | M. Arquitetura e distribuição |
| [D69](D69-binario-estatico-kd-self-setup.md) | Binário estático + `kd self setup` | Aceita | M. Arquitetura e distribuição |
| [D70](D70-sem-migracao.md) | Sem migração | Aceita | M. Arquitetura e distribuição |
| [D71](D71-pipe-para-llm-json.md) | Pipe para LLM + --json | Aceita | N. Contrato de saída e erros |
| [D72](D72-catalogo-de-mensagens-congelado-por-teste.md) | Catálogo de mensagens congelado por teste | Aceita | N. Contrato de saída e erros |
| [D73](D73-epipe-exit-0.md) | EPIPE → exit 0 | Aceita | N. Contrato de saída e erros |
| [D74](D74-proprios.md) | próprios | Aceita | O. TOON |
| [D75](D75-fallback-e-deteccao-de-versao.md) | fallback e detecção de versão | Aceita | O. TOON |
| [D76](D76-golden-snapshot-property-tests-stress-de-concorrencia.md) | Golden/snapshot + property tests + stress de concorrência + | Aceita | P. Testes e qualidade |
| [D77](D77-divergences.md) | DIVERGENCES | Aceita | P. Testes e qualidade |
| [D78](D78-matriz-de-aceite-por-tool.md) | Matriz de aceite por tool | Aceita | P. Testes e qualidade |
| [D81](D81-recall-funde-canais-por-rrf.md) | recall funde canais por RRF | Aceita | Q. Extrações do arags (D81–D92) |
| [D82](D82-orcamento-do-prime-sem-tokenizer.md) | Orçamento do prime sem tokenizer | Aceita | Q. Extrações do arags (D81–D92) |
| [D83](D83-cache-de-embedding-por-bodyhash.md) | Cache de embedding por body_hash | Aceita | Q. Extrações do arags (D81–D92) |
| [D84](D84-purga-do-vetor-em-toda-remocao.md) | Purga do vetor em toda remoção | Aceita | Q. Extrações do arags (D81–D92) |
| [D85](D85-flush-coalescido.md) | Flush coalescido | Aceita | Q. Extrações do arags (D81–D92) |
| [D86](D86-anchors-com-path-contenthash-derivado.md) | anchors com path + content_hash derivado | Aceita | Q. Extrações do arags (D81–D92) |
| [D87](D87-confianca-derivada-em-tempo-de-consulta.md) | Confiança derivada em tempo de consulta | Aceita | Q. Extrações do arags (D81–D92) |
| [D88](D88-rewind-emite-contextid-enderecavel.md) | rewind emite context_id endereçável | Aceita | Q. Extrações do arags (D81–D92) |
| [D89](D89-provider-lightweight.md) | provider = "lightweight" | Aceita | Q. Extrações do arags (D81–D92) |
| [D90](D90-kd-maintenance-eval-ab.md) | kd maintenance eval --ab | Aceita | Q. Extrações do arags (D81–D92) |
| [D91](D91-projeto-por-nome-logico.md) | Projeto por nome lógico | Aceita | Q. Extrações do arags (D81–D92) |
| [D92](D92-disciplina-rust.md) | Disciplina Rust | Aceita | Q. Extrações do arags (D81–D92) |
| [D93](D93-task-com-hierarquia-fechada.md) | task com hierarquia fechada | Aceita | R. Superfície CLI v2 (D93–D94) |
| [D94](D94-strict-e-config-de-projeto.md) | strict é config de projeto | Aceita | R. Superfície CLI v2 (D93–D94) |
| [D95](D95-hash-curto-sha-256-truncado-aos-4.md) | Hash curto = SHA-256 truncado aos 4 primeiros bytes | Aceita | S. Contrato de bytes (D95) |
| [D96](D96-evento.md) | Evento | Aceita | T. Persistência e eventos (D96) |
| [D97](D97-toml-subset-proprio.md) | TOML subset próprio | Aceita | U. Config e worktree (D97) |
| [D98](D98-arestas-explicitas-sao-chaves-de-frontmatter-de.md) | Arestas explícitas são chaves de frontmatter de primeiro nível | Aceita | V. Arestas explícitas (D98) |
| [D99](D99-o-catalogo-de-validators-e.md) | O catálogo de validators é **` | Aceita | W. Catálogo de validators (D99) |
| [D100](D100-notbefore.md) | not_before | Aceita | X. Agendamento separado de expiração (D100) |
| [D101](D101-nao-roda-in-process.md) | não roda in-process | Aceita | Y. Embeddings via HTTP local (D101) |
| [D102](D102-filtrado.md) | filtrado | Aceita | Impactos no panorama (D102–D171) |
| [D103](D103-qualquer.md) | qualquer | Aceita | Impactos no panorama (D102–D171) |
| [D104](D104-modos.md) | modos | Aceita | Impactos no panorama (D102–D171) |
| [D105](D105-atomica.md) | atômica | Aceita | Impactos no panorama (D102–D171) |
| [D106](D106-rewind-emite-next.md) | `rewind` emite `next | Aceita | Impactos no panorama (D102–D171) |
| [D107](D107-kd-ask-tags-lista-o-vocabulario-de.md) | `kd ask --tags` lista o vocabulário de tags | Aceita | Impactos no panorama (D102–D171) |
| [D108](D108-tarefaconhecimento.md) | tarefa→conhecimento | Aceita | Impactos no panorama (D102–D171) |
| [D109](D109-derivado.md) | derivado | Aceita | Impactos no panorama (D102–D171) |
| [D110](D110-jsonl.md) | JSONL | Aceita | Impactos no panorama (D102–D171) |
| [D111](D111-tarefaconhecimento.md) | tarefa→conhecimento | Aceita | Impactos no panorama (D102–D171) |
| [D112](D112-propoe.md) | propõe | Aceita | Impactos no panorama (D102–D171) |
| [D113](D113-scope-nivel-type-especie-kd-task-new.md) | `scope` = nível, `type` = espécie; `kd task new --kind`; `scope` | Aceita | Impactos no panorama (D102–D171) |
| [D114](D114-derivado-de-eventos.md) | derivado de eventos | Aceita | Impactos no panorama (D102–D171) |
| [D115](D115-derivado.md) | derivado | Aceita | Impactos no panorama (D102–D171) |
| [D116](D116-derivado.md) | derivado | Aceita | Impactos no panorama (D102–D171) |
| [D119](D119-programa-arquivo-externo-plan.md) | Programa = arquivo externo plan/ | Aceita | Impactos no panorama (D102–D171) |
| [D120](D120-item-de-trabalho-especie-de-trabalho-com.md) | Item de trabalho = espécie de trabalho com scope | Aceita | Impactos no panorama (D102–D171) |
| [D121](D121-semantic.md) | semantic | Aceita | Impactos no panorama (D102–D171) |
| [D122](D122-stopwords-pt-en.md) | stopwords PT+EN | Aceita | Impactos no panorama (D102–D171) |
| [D123](D123-modelo-de-embedding-default-passa-a.md) | Modelo de embedding default passa a | Aceita | Impactos no panorama (D102–D171) |
| [D124](D124-fusao-rrf-passa-a-ter-peso-por.md) | Fusão RRF passa a ter peso por canal | Aceita | Impactos no panorama (D102–D171) |
| [D125](D125-kd-task-show-resolve-o-contexto-estrutural.md) | kd task show resolve o contexto estrutural | Aceita | Impactos no panorama (D102–D171) |
| [D126](D126-arestas-tem-via-unica.md) | Arestas têm via única | Aceita | Impactos no panorama (D102–D171) |
| [D127](D127-rollup-de-progresso-por-epico-e-derivado.md) | Rollup de progresso por épico é derivado | Aceita | Impactos no panorama (D102–D171) |
| [D128](D128-clusters-ganham-verbo-proprio-kd-knowledge-e.md) | Clusters ganham verbo próprio kd knowledge e o eixo container passa a | Aceita | Impactos no panorama (D102–D171) |
| [D129](D129-fase-2-usa-complete-link.md) | Fase 2 usa complete-link | Aceita | Impactos no panorama (D102–D171) |
| [D130](D130-verbos-falham-alto-nunca-em-silencio.md) | Verbos falham alto, nunca em silêncio | Aceita | Impactos no panorama (D102–D171) |
| [D131](D131-auto-drain-ocioso-no-lazy-sem-eager.md) | Auto-drain ocioso no lazy; sem eager | Aceita | Impactos no panorama (D102–D171) |
| [D132](D132-kd-maintenance-watch-service-gerencia-o-worker.md) | kd maintenance watch-service gerencia o worker ocioso — com | Aceita | Impactos no panorama (D102–D171) |
| [D133](D133-watch-service-e-multiplataforma-e-mantem-o.md) | watch-service é multiplataforma e mantém o servidor de embeddings | Aceita | Impactos no panorama (D102–D171) |
| [D134](D134-epic-e-a-raiz-issue-e-opcional.md) | epic é a raiz, issue é opcional e scope=plan sai | Aceita | Impactos no panorama (D102–D171) |
| [D135](D135-anchor-e-o-unico-link-externo-fim.md) | --anchor é o único link externo; fim de --source, --expires-at e | Aceita | Impactos no panorama (D102–D171) |
| [D136](D136-um-agente-principal-sempre.md) | Um agente principal, sempre | Aceita | Impactos no panorama (D102–D171) |
| [D137](D137-show-completo-e-list-full-content.md) | show completo e list --full-content | Aceita | Impactos no panorama (D102–D171) |
| [D138](D138-kd-task-plan-fica-so-com-prompt.md) | kd task plan fica só com --prompt/--submit | Aceita | Impactos no panorama (D102–D171) |
| [D139](D139-graph-enxuto-e-plan.md) | graph enxuto e plan | Aceita | Impactos no panorama (D102–D171) |
| [D140](D140-convencao-universal-do-posicional.md) | Convenção universal do posicional | Aceita | Impactos no panorama (D102–D171) |
| [D141](D141-criacao-de-tarefas-em-lote.md) | Criação de tarefas em lote | Aceita | Impactos no panorama (D102–D171) |
| [D142](D142-poda-de-kd-write.md) | Poda de kd write | Aceita | Impactos no panorama (D102–D171) |
| [D143](D143-escopo-de-conhecimento.md) | Escopo de conhecimento | Aceita | Impactos no panorama (D102–D171) |
| [D144](D144-escopo-obrigatorio-em-learn-compact-prune-e.md) | Escopo obrigatório em learn/compact/prune e task list | Aceita | Impactos no panorama (D102–D171) |
| [D145](D145-fim-do-maintenance-eval-index-vira-kd.md) | Fim do maintenance eval; index vira kd knowledge digest | Aceita | Impactos no panorama (D102–D171) |
| [D146](D146-kd-ask-so-conhecimento-e-superficie-enxuta.md) | kd ask só conhecimento e superfície enxuta | Aceita | Impactos no panorama (D102–D171) |
| [D147](D147-params.md) | --params '{ | Aceita | Impactos no panorama (D102–D171) |
| [D148](D148-o-cache-vetorial-e-versionado.md) | O cache vetorial é versionado | Aceita | Impactos no panorama (D102–D171) |
| [D149](D149-fim-do-type-container-o-grupo-e.md) | Fim do type=container; o grupo é derivado de scope=epic | Aceita | Impactos no panorama (D102–D171) |
| [D150](D150-o-mapa-de-conhecimento-e-material-e.md) | O mapa de conhecimento é material e versionado | Aceita | Impactos no panorama (D102–D171) |
| [D151](D151-ask-expoe-a-contribuicao-de-canal-no.md) | ask expõe a contribuição de canal no --json | Aceita | Impactos no panorama (D102–D171) |
| [D152](D152-busca-sem-resultado-devolve-noresults.md) | Busca sem resultado devolve [no_results] | Aceita | Impactos no panorama (D102–D171) |
| [D153](D153-sincronizacao-multi-dev.md) | Sincronização multi-dev | Aceita | Impactos no panorama (D102–D171) |
| [D154](D154-renovacao-de-shelf-life-por-uso.md) | Renovação de shelf-life por uso | Aceita | Impactos no panorama (D102–D171) |
| [D155](D155-consulta-temporal-kd-ask-as-of-ts.md) | Consulta temporal kd ask --as-of <TS> | Aceita | Impactos no panorama (D102–D171) |
| [D156](D156-portao-de-evidencia-em-propostas.md) | Portão de evidência em propostas | Aceita | Impactos no panorama (D102–D171) |
| [D157](D157-promocao-de-conhecimento-a-regras-governadas-no.md) | Promoção de conhecimento a regras governadas no AGENTS | Aceita | Impactos no panorama (D102–D171) |
| [D158](D158-sugestao-semantica-de-arestas-contradicoes.md) | Sugestão semântica de arestas/contradições | Aceita | Impactos no panorama (D102–D171) |
| [D159](D159-redacao-tipada-de-segredos-no-log.md) | Redação tipada de segredos no log | Aceita | Impactos no panorama (D102–D171) |
| [D160](D160-varredura-de-residuos-na-inicializacao.md) | Varredura de resíduos na inicialização | Aceita | Impactos no panorama (D102–D171) |
| [D161](D161-busca-com-revelacao-progressiva-e-corpo-visivel.md) | Busca com revelação progressiva e corpo visível | Aceita | Impactos no panorama (D102–D171) |
| [D162](D162-incentivo-ao-corpo.md) | Incentivo ao corpo | Aceita | Impactos no panorama (D102–D171) |
| [D163](D163-kd-doctor-de-topo.md) | kd doctor de topo | Aceita | Impactos no panorama (D102–D171) |
| [D164](D164-kd-help-de-topo.md) | kd help de topo | Aceita | Impactos no panorama (D102–D171) |
| [D165](D165-verbosidade-de-init-self-config-sync-maintenance.md) | Verbosidade de init/self/config/sync/maintenance | Aceita | Impactos no panorama (D102–D171) |
| [D166](D166-prime-compacto-por-padrao.md) | prime compacto por padrão | Aceita | Impactos no panorama (D102–D171) |
| [D167](D167-help-embutido-por-verbo.md) | Help embutido por verbo | Aceita | Impactos no panorama (D102–D171) |
| [D168](D168-redundancia-da-superficie.md) | Redundância da superfície | Aceita | Impactos no panorama (D102–D171) |
| [D169](D169-documentacao-viva-e-estatica-em-sincronia.md) | Documentação viva e estática em sincronia | Aceita | Impactos no panorama (D102–D171) |
| [D170](D170-kd-drain-de-topo.md) | kd drain de topo | Aceita | Impactos no panorama (D102–D171) |
| [D171](D171-kd-sozinho-kd-help.md) | kd sozinho = kd help | Aceita | Impactos no panorama (D102–D171) |
| [D172](D172-fold-de-diacriticos-na-tokenizacao.md) | Fold de diacríticos na tokenização | Aceita | 0.5.0 (E16–E19) |
| [D173](D173-termos-de-alta-frequencia.md) | Termos de alta frequência | Aceita | 0.5.0 (E16–E19) |
| [D176](D176-consistencia-de-status-na-leitura.md) | Consistência de status na leitura | Aceita | 0.5.0 (E16–E19) |
| [D180](D180-acao-explicita-aceite.md) | Ação explícita = aceite | Aceita | 0.5.0 (E16–E19) |
| [D181](D181-stream-do-worker.md) | Stream do worker | Aceita | 0.5.0 (E16–E19) |
| [D182](D182-reconciliacao-e-probe-do-worker.md) | Reconciliação e probe do worker | Aceita | 0.5.0 (E16–E19) |
| [D183](D183-supply-chain-do-worker.md) | Supply-chain do worker | Aceita | 0.5.0 (E16–E19) |
| [D187](D187-self-upgrade-real.md) | self upgrade real | Aceita | 0.5.0 (E16–E19) |
| [D188](D188-auditoria-de-verbos-acionaveis.md) | Auditoria de verbos acionáveis | Aceita | 0.5.0 (E16–E19) |
| [D184](D184-scripts-no-repositorio-comando-wrapper-fino.md) | Scripts no repositório; comando = wrapper fino | Aceita | 0.5.0 (E16–E19) |
| [D185](D185-cross-platform-do-wrapper.md) | Cross-platform do wrapper | Aceita | 0.5.0 (E16–E19) |
| [D186](D186-watch-service-sob-drain-maintenance-so-revisao.md) | watch-service sob drain; maintenance só revisão | Aceita | 0.5.0 (E16–E19) |
| [D202](D202-porta-unificada-do-provedor-de-embeddings-8889.md) | Porta unificada do provedor de embeddings = 8889 | Aceita | 0.5.0 (E16–E19) |
| [D189](D189-confianca-bayesiana-beta-bernoulli.md) | Confiança bayesiana Beta-Bernoulli | Aceita | 0.5.0 (E16–E19) |
| [D190](D190-retencao-por-curva-de-esquecimento.md) | Retenção por curva de esquecimento | Aceita | 0.5.0 (E16–E19) |
| [D191](D191-data-contract-por-tipo.md) | Data contract por tipo | Aceita | 0.5.0 (E16–E19) |
| [D192](D192-autoridade-no-grafo.md) | Autoridade no grafo | Aceita | 0.5.0 (E16–E19) |
| [D193](D193-comunidades-graphrag.md) | Comunidades + GraphRAG | Aceita | 0.5.0 (E16–E19) |
| [D175](D175-idade-no-ranking-sem-query.md) | Idade no ranking sem query | Aceita | 0.5.0 (E16–E19) |
| [D177](D177-contradicts-no-ranking-e-no-prune.md) | contradicts no ranking e no prune | Aceita | 0.5.0 (E16–E19) |
| [D179](D179-fusao-recalibrada-por-medicao.md) | Fusão recalibrada por medição | Aceita | 0.5.0 (E16–E19) |
| [D203](D203-drift-de-ancoras-persistido-e-aplicado-a.md) | Drift de âncoras persistido e aplicado à confiança | Aceita | 0.5.0 (E16–E19) |
| [D204](D204-blocking-minhash-lsh-no-dedup.md) | Blocking MinHash/LSH no dedup | Aceita | 0.5.0 (E16–E19) |
| [D205](D205-metricas-de-fluxo-e-caminho-critico.md) | Métricas de fluxo e caminho crítico | Aceita | 0.5.0 (E16–E19) |
| [D206](D206-stemming-pt-conservador.md) | Stemming PT conservador | Aceita | 0.5.0 (E16–E19) |
| [D207](D207-claims-spo-ontologia-leve-proveniencia.md) | Claims SPO + ontologia leve + proveniência | Aceita | 0.5.0 (E16–E19) |
| [D208](D208-tms-defeasible-drift-kl-js.md) | TMS/defeasible + drift KL/JS | Aceita | 0.5.0 (E16–E19) |
| [D209](D209-superficie-v3.md) | Superfície v3 | Aceita | 0.5.0 (E16–E19) |
| [D210](D210-listas-repeticao-virgula.md) | Listas na CLI: repetição + vírgula | Aceita | 0.5.0 (E16–E19) |
| [D212](D212-conjuntos-fechados-lista-sugestao.md) | Conjuntos fechados: lista + sugestão da mais provável | Aceita | 0.5.0 (E16–E19) |
| [D215](D215-dreno-em-lotes-limitados-e-digest-honesto.md) | Dreno em lotes limitados e `--digest` honesto | Aceita | 0.5.4 |

## Como usar

- O **contrato canônico** é `plan/03_decisoes-fechadas.md`; estes ADRs são a leitura individual.
- Contrato de bytes: [`../TOON.md`](../TOON.md). Bordas: [`../DIVERGENCES.md`](../DIVERGENCES.md).
