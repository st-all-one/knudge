# D139 — `graph` enxuto e `plan.md` ancorando vários épicos (floresta)

> **Status:** decisão fechada (registrada em `plan/03_decisoes-fechadas.md`); **implementação
> pendente**. Origem: revisão de `kd task graph`/Programa externo. Revisa **D115/D116/D119**.
> Sem execução.

## 1. Decisões

1. **`kd task graph` enxuto.** O texto passa a `id|kind|status|statement (done/total)`, indentado
   por profundidade. **`role` e `mode` saem do texto** (rótulos derivados, verbosos); seguem no
   `--json`, que por sua vez perde `owner` (D136). Sem flag nova — o `graph` fica leve; quem quer
   os rótulos usa `--json`.
2. **`plan.md` pode ancorar vários épicos-raiz.** Hoje `root_for_path` devolve **um** (o menor
   id); passa a `roots_for_path`, devolvendo **todos** os containers sem pai cujo `anchors` casa o
   path, em ordem de `id` (determinístico). `kd task graph --program plan.md` renderiza a
   **floresta** inteira; `--root <ID>` continua raiz única. O `plan.md` vira o "super-épico" que
   agrupa o universo da demanda daquele corpo (D119).

## 2. Consequências

- **Texto do `graph`** deixa de carregar `role`/`mode`/`owner`; o `--json` mantém `role`/`mode`/
  `progress` (só `owner` sai).
- `program.rs`: `root_for_path(&[Note], &str) -> Result<Option<String>>` vira
  `roots_for_path(&[Note], &str) -> Result<Vec<String>>` (ordem de `id`).
- `doctor` `program-anchor`: `root_for_path(...).is_none()` → `roots_for_path(...).is_empty()`.
- `resolve_roots` (`graph.rs`) passa a devolver `Vec` com **todos** os roots do programa.
- `mode` (D116) segue derivado, mas deixa de aparecer no texto; `role` (D115) idem.

## 3. Raio de alcance (quando executar)

- `task/program.rs` (`roots_for_path`), `task/mod.rs` (export), `commands/task/graph.rs`
  (`resolve_roots` + render), `health/doctor/checks.rs` (`program_anchor_check`),
  `docs/05-task.md`, `prime.rs`, goldens/testes de programa e de graph.

## 4. Aceite

- [ ] Texto do `graph`: `id|kind|status|statement (done/total)`, sem `role`/`mode`/`owner`.
- [ ] `graph --program plan.md` com **2+ épicos** ancorados renderiza a floresta (ordem de `id`).
- [ ] `graph --root <ID>` segue renderizando uma árvore só.
- [ ] `doctor` acusa `program-anchor` quando **nenhum** épico está ancorado ao `plan.md`.
- [ ] Docs, `prime` e goldens atualizados; `make check` verde.
