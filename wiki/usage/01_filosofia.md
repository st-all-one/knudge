# 01 · Filosofia

O que o knudge é, por que existe e como ele ajuda — sem jargão. Se você só ler uma página desta
documentação, leia esta.

---

## 1. O problema: memória de projeto não vive na cabeça nem no prompt

Quem desenvolve com agentes de IA conhece a cena: a sessão acaba e **todo o contexto evapora**. Na
próxima vez, alguém (ou algum modelo) descobre de novo a mesma coisa, no mesmo arquivo, pelo mesmo
erro. E quando o conhecimento é registrado, ele se espalha: três notas dizendo quase a mesma coisa,
duas delas desatualizadas, ninguém sabendo qual vale.

Os sintomas são sempre os mesmos:

- **Contaminação:** a memória enche de duplicatas e de informação contraditória; o agente "aprende"
  a coisa errada e repete.
- **Falha de processo:** decisões importantes moram em conversas que ninguém lê; tarefas se perdem
  entre sessões; o trabalho recomeça do zero.
- **Depreciação:** o que era verdade há seis meses continua sendo respondido como se fosse hoje.
- **Vida útil do projeto:** o repositório dura anos, mas o conhecimento que o explica dura uma
  sessão. Trocar de máquina, entrar outro dev, voltar depois de férias — tudo se perde.

O knudge nasceu para resolver isso **onde o código vive**, sem nuvem, sem banco de dados e sem
prender o projeto a uma ferramenta.

---

## 2. O que é o knudge, em uma frase

> **knudge é a memória por projeto de um agente de IA: notas Markdown versionadas ao lado do
> código, que o agente consulta antes de agir e atualiza depois de aprender.**

Ele não é um chat, nem um banco, nem um wiki. É uma camada simples que dá ao agente três verbos
essenciais:

- **buscar** o que já se sabe (`kd ask`);
- **gravar** o que se aprendeu (`kd write`);
- **planejar/executar** o que falta (`kd task`).

E um quarto que fecha o ciclo: **versionar** (`kd sync`).

```
kd ask → kd write → kd task → kd sync
(buscar)  (gravar)   (executar) (commit)
```

---

## 3. As ideias centrais

### 3.1 As notas são a verdade; o índice é só um atalho

Tudo o que o knudge sabe é um arquivo Markdown legível em `.knudge/notas/`. Um índice rápido
acelera a busca, mas ele é **descartável**: se sumir ou divergir, é reconstruído do zero. Você
nunca depende dele para nada permanente.

**Por que importa:** o conhecimento é seu, em texto simples, versionado no seu git. Sem formato
secreto, sem dependência de serviço.

### 3.2 Uma afirmação por nota

Cada nota carrega **uma** afirmação curta e autocontida ("O gateway limita 100 rps por chave"), com
um corpo opcional que explica o porquê. O endereço da nota é derivado do próprio conteúdo: a mesma
afirmação gera o mesmo id em qualquer máquina.

**Por que importa:** notas pequenas são fáceis de achar, revisar, citar e mesclar. É o que torna o
corpus **mergeável pelo git** e barato de consultar.

### 3.3 Busque antes de gravar

Antes de criar qualquer nota, o `kd` procura o que já existe. Se algo parecido já está lá, ele
**avisa**, **mescla** ou **recusa** a duplicata. Você nunca escreve no escuro.

**Por que importa:** é a principal defesa contra a contaminação. A memória fica enxuta porque ela
é construída por adição **verificada**, não por acúmulo.

### 3.4 Âncoras ligam a memória ao código

Uma nota pode apontar para os arquivos que ela toca (`--anchor src/gateway.rs`). Antes de editar
um arquivo, o agente pergunta "o que já se sabe sobre isto?" e recebe só o relevante.

**Por que importa:** a memória fica **situada**. Em vez de despejar contexto, você entrega o
contexto certo, no momento certo.

### 3.5 O knudge propõe; você decide

Comandos como `maintenance learn/prune/compact` **nunca agem sozinhos**: eles listam propostas
(possíveis duplicatas, notas a aposentar, links a criar). A decisão é sempre humana (ou do agente,
explicitamente).

**Por que importa:** automação que apaga ou funde conhecimento em silêncio é um risco. Aqui, nada
some sem um comando explícito, e nada é irreversível de imediato.

### 3.6 Contexto pequeno, resposta direta

A saída padrão de uma busca é uma linha por resultado: `id | afirmação | score | motivo`. O corpo
completo só aparece quando pedido. O agente gasta pouco contexto para decidir muito.

**Por que importa:** contexto de modelo é caro e limitado. Uma memória que devolve páginas inteiras
é tão inútil quanto não ter memória.

### 3.7 Determinismo e simplicidade

O mesmo corpus, com a mesma pergunta, devolve o mesmo resultado — sempre. Nada depende de sorte,
horário ou ordem de arquivos. A ferramenta evita complexidade que o caso não paga.

**Por que importa:** resultados reproduzíveis são testáveis, auditáveis e confiáveis.

---

## 4. Como isso beneficia quem desenvolve

| Antes | Com o knudge |
|---|---|
| Redescobrir a mesma coisa a cada sessão | `kd ask` responde em uma linha |
| Conhecimento espalhado e contraditório | dedup na escrita + detecção de contradição |
| Decisões perdidas em conversas | decisão vira nota versionada, com o porquê |
| Tarefas esquecidas entre sessões | `kd task` + `kd rewind` retomam o trabalho |
| Dúvida sobre o que é atual | shelf-life, drift e `forget` aposentam o obsoleto |
| Onboarding lento de outro dev/agente | o corpus vive no repo, versionado e buscável |
| Contexto caro e ruidoso | respostas curtas e situadas por âncora |

---

## 5. Como o knudge mitiga cada risco

### 5.1 Contaminação de conhecimento

- **Dedup na escrita:** o `write` compara com o que existe e cria, mescla ou recusa.
- **Uma afirmação por nota:** menos ambiguidade, mais granularidade.
- **Contradição explícita:** notas podem declarar que se contradizem; o `doctor` aponta o conflito.
- **Revisão versionada:** atualizar não apaga — cria uma nova revisão e marca a antiga como
  substituída.

### 5.2 Falhas de processo

- **Trabalho com evidência:** fechar uma tarefa exige um resultado (sucesso/parcial/falha); nada
  fecha "porque sim".
- **Retomada de contexto:** `kd rewind` reconstrói "onde eu estava" dentro de um orçamento.
- **Ciclo explícito:** buscar → gravar → executar → versionar, sempre na mesma ordem.
- **Propostas, não surpresas:** a manutenção sugere; você aprova.

### 5.3 Depreciação de conhecimento

- **Ciclo de vida por tipo:** conhecimento fundamental não expira; o tático e o observacional têm
  prazo e podem ser aposentados.
- **Renovação por uso:** cada sucesso confirmado estende a validade da nota.
- **Detecção de obsolescência:** âncoras quebradas, contradições e mudança de vocabulário de um
  tópico fazem o knudge **propor** aposentadoria.
- **Esquecer é reversível:** `forget` marca; `restore` desfaz; a remoção física é o último passo.

### 5.4 Vida útil do projeto

- **Tudo em texto no seu git:** notas e eventos versionados; o índice é reconstruível.
- **Trabalho em equipe:** notas endereçadas por conteúdo fazem merge natural; o cache de vetores
  é versionável para reindexar sem reinferência.
- **Sem dependência externa:** a memória sobrevive a troca de máquina, de modelo e de ferramenta.

---

## 6. O que o knudge **não** é

- **Não é um chatbot.** É uma memória consultável por linha de comando (e por MCP).
- **Não é um banco de dados.** São arquivos Markdown; não há servidor nem schema rígido.
- **Não é um serviço de nuvem.** Tudo fica no seu projeto; nada sai da máquina.
- **Não é um wiki.** É feito para um agente **agir** (buscar/gravar/planejar), não para leitura
  passiva.
- **Não é uma caixa-preta.** Se o índice sumir, o corpus continua lá, legível.

---

## 7. Como o knudge faz, em alto nível

1. **Funda** a memória no projeto (`kd init`) e ensina o agente a usá-la.
2. **Busca** com texto, âncora ou grafo, devolvendo ponteiros curtos (`kd ask`).
3. **Grava** com verificação de duplicata e ligação ao código (`kd write`).
4. **Planeja** o trabalho numa árvore simples: épico → issue → tarefa (`kd task`).
5. **Retoma** o contexto entre sessões (`kd rewind`).
6. **Cuida** da saúde e da atualidade do corpus (`kd doctor`, `kd maintenance`).
7. **Versiona** tudo o que importa (`kd sync`).

Cada um desses passos tem um guia dedicado nesta documentação — comece pelo
[ciclo](02_ciclo.md) e depois vá ao comando que você precisa.

---

## 8. Próximo passo

➡️ [02 · O ciclo e a CLI](02_ciclo.md) · [00 · Quickstart](00_quickstart.md)
