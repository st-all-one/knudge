# 07 — Grafo, inferência e comunidades

Arestas explícitas no frontmatter são a verdade (D49/D98); o `Graph` é uma **projeção** derivada.
Especificação: [`../specs/grafo.md`](../specs/grafo.md).

## Construir e consultar

```rust
use knudge_core::graph::Graph;

let graph = Graph::build(&kd.store())?;       // do store
let graph = Graph::from_notes(notes)?;        // de notas já lidas
let graph = Graph::from_notes_ref(&notes)?;   // sem consumir o vetor

graph.contains("fact_x");
graph.note_type("fact_x");                    // Option<NoteType>
graph.scope("epic_x");                        // Option<Scope>
graph.status("task_x");                       // Option<Status>
graph.targets("fact_x", EdgeKind::DependsOn);
graph.parent("task_x");                       // via results_in
graph.expand("fact_x", None, 2);              // BFS determinística (depth, kind)
```

## Integridade e ciclos (D45/D46)

```rust
use knudge_core::graph::{Issue, IssueKind};

for issue in graph.integrity() {
    match issue.kind {
        IssueKind::Dangling   => eprintln!("aresta para id ausente: {} -> {}", issue.from, issue.to),
        IssueKind::SelfEdge   => eprintln!("auto-aresta em {}", issue.from),
        _ => {}
    }
}

let supersessao = graph.supersession_cycles();  // membros protegidos (não demolem — D45)
let dependencia = graph.dependency_cycles();
if graph.has_contradictions() { /* há `contradicts` declarado (D177) */ }
```

Integridade alimenta o check `integrity` do `doctor` (doc 08).

## Ontologia e claims (D207)

```rust
use knudge_core::graph::{equivalence_classes, broader_ancestors, has_hierarchy_cycle, claim_conflicts};

let classes = equivalence_classes(&graph);          // same_as -> representante
let amplos = broader_ancestors(&graph, "termo_x");
let ciclo = has_hierarchy_cycle(&graph);            // broader/narrower cíclico

// contradição precisa: mesma (sujeito, relação), objetos divergentes
let conflitos = claim_conflicts(&notes)?;
```

## TMS / defeasible (D208)

```rust
use knudge_core::graph::{retracted, defeated_dependents, defeated_by_replacement};

let retratadas = retracted(&graph);                    // forgotten/superseded como premissas
let derrotados = defeated_dependents(&graph, &retratadas); // dependentes transitivos
let substituidos = defeated_by_replacement(&graph);
```

Nada é apagado: são conjuntos **derivados** usados para ranking/demolição (`DemotionReason`).

## Autoridade: PageRank e PPR (D192)

```rust
use knudge_core::graph::{pagerank, personalized_pagerank};
use std::collections::BTreeSet;

let global = pagerank(&graph);                                // BTreeMap<id, score>
let seeds: BTreeSet<String> = working_set.into_iter().collect();
let local = personalized_pagerank(&graph, &seeds);            // vizinhança das âncoras
```

PPR alimenta o canal `ppr` da fusão (`recall.ppr_weight`, default `0.0`).

## Comunidades (D193)

```rust
use knudge_core::graph::{louvain, WeightedGraph};
use knudge_core::lifecycle::{communities, communities_filtered};

// grafo ponderado (arestas explícitas + âncoras compartilhadas)
let mut wg = WeightedGraph::new(graph.ids());
wg.add_edge("fact_a", "fact_b", 1.0);
let grupos = louvain(&wg);                            // Vec<Vec<String>>

// comunidades prontas a partir de índice+grafo (members + terms)
let com = communities(&index, &graph);
let filtradas = communities_filtered(&index, &graph, &filter);
```

## Sugestões (nunca viram aresta — D49/D50)

```rust
use knudge_core::graph::{extract, SuggestionStore};
use std::collections::BTreeSet;

// alvos válidos = ids conhecidos
let known: BTreeSet<String> = graph.ids().into_iter().map(String::from).collect();
let store = SuggestionStore::new(kd.fs_dyn(), kd.knowledge_dir());

for note in &notes {
    let sugestoes = extract(&note.body, &known);   // conservador (ids, wikilinks, verbos)
    if !sugestoes.is_empty() {
        store.write(note.id()?, &sugestoes)?;      // `.idx/suggestions.jsonl` (derivado)
    }
}
```

Sugestões vivem em `.idx/suggestions.jsonl` (derivado, purgável). Aprovar é um ato explícito:
use `write::link`/`graph::link`.

## Adicionar aresta

```rust
use knudge_core::graph::link;

let mut note = kd.store().read("fact_x")?;
if link(&mut note.frontmatter, EdgeKind::Supports, "fact_y")? {
    note.set_revision(note.revision().saturating_add(1))?;
    kd.store().write(&note)?;   // + evento, idealmente via write::link
}
```

Prefira `write::link(&ctx, from, kind, to)` (doc 05): já valida, incrementa revisão e registra
evento.

## Recomendações

- **Derive uma vez.** `Graph::build`/`from_notes_ref` é O(arestas); reaproveite junto do `Index`.
- **Aresta é declaração, não inferência.** Só `link`/frontmatter criam aresta; sugestões ficam no
  lado.
- **Bidirecionalidade cobrada.** `replaces`/`superseded_by` devem ser espelhados; o `doctor`
  acusa.
- **Proteja ciclos.** Antes de propor demolição, filtre `graph.cycle_members()` (D45).
- **PPR só compensa com arestas densas** — mantenha `recall.ppr_weight` em `0.0` até medir.
