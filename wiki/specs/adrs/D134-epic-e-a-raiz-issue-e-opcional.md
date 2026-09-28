# D134 — epic é a raiz, issue é opcional e scope=plan sai

- **Status:** Aceita
- **Categoria:** Impactos no panorama (D102–D171)

## Contexto

Bloco **Impactos no panorama (D102–D171)**. Linhagem: Revisa D52/D93/D113/D115/D119/D127.

## Decisão

**`epic` é a raiz, `issue` é opcional e `scope=plan` sai.** Conclui a depreciação de D119: o enum `scope` passa a `{epic, issue, task}` (`Scope::ALL` com 3) e `plan` deixa de ser um nível. O `epic` **não tem pai** e pode viver sozinho; ancorá-lo em `plan.md`/qualquer arquivo é **fortemente recomendado** — `doctor` emite **warning**, não erro. A hierarquia deixa de exigir o pai imediatamente externo: vale qualquer pai de **rank estritamente menor** (`epic < issue < task`), então `epic → task` direto é válido e o `issue` vira opcional. O nível deixa de ser intrínseco ao `scope` e passa a ser **derivado da árvore**; o papel (D115) é derivado por profundidade. `task plan --submit` gera folhas `task`. Migração: `doctor --fix` reescreve `scope: plan` → `scope: epic` (lossless — mesmo `type=container`, mesmo id por D95), em vez de deixar a nota raiz sumir no skip tolerante. Revisa D52/D93/D113/D115/D119/D127.

## Impacto

- Decisão de contrato/projeto propagada para código, testes e documentação.

---

> Fonte: [`plan/03_decisoes-fechadas.md`](../../../plan/03_decisoes-fechadas.md).
