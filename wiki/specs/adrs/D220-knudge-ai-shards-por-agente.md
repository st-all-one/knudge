# D220 — Memória em shards por agente (`.knudge/ai/<area>/<agent_id>.jsonl`)

- **Status:** Aceita
- **Categoria:** Eixo de memória / layout de shards (D218)

## Contexto

D218 introduziu a fonte JSONL read-only e o `kd ai`, mas com um ficheiro por **área**
(`notes.jsonl`, `session-log.jsonl`). Vários agentes a escrever no mesmo ficheiro criam contenção:
cada append exige abrir o ficheiro partilhado, e um crash de um escritor pode truncar a linha de
outro. Um **manifesto** com a lista de ficheiros/offsets resolveria a descoberta, mas vira um
segundo estado mutável a corromper e a sincronizar.

## Decisão

O layout canônico passa a ser **um shard por escritor**:

```
.knudge/ai/
  memory/<agent_id>.jsonl        # memória do agente (área default)
  session-log/<agent_id>.jsonl   # projeção do log (activity=derive)
  events/                        # reservado (não implementado)
  README.md                      # contrato do diretório (opcional, do agente)
```

- **`Source::JsonlDir(path)`** lê o diretório: varre `*.jsonl` em **ordem lexicográfica** e mescla
  os shards (D223). `Source::Jsonl(path)` continua válido para um ficheiro explícito;
  `Source::at(fs, path)` escolhe diretório ou ficheiro pelo FS (caminho inexistente conta como
  ficheiro, e fonte ausente = corpus vazio, exit 0).
- **`--source PATH` aceita ficheiro ou diretório** — o utilizador aponta para um shard, para a
  pasta de shards ou para uma fonte JSONL avulsa, sem flag nova.
- **Sem manifesto mutável**: o leitor descobre os shards listando o diretório. Nada a atualizar
  quando um agente nasce ou morre.
- **`.knudge/ai` segue `knowledge.persist_in_project`** (D34/D213), como o resto do `.knudge/`:
  versionado por default (com o derivado excluído) ou local-only.
- **`events/` é reservado**: existe no contrato do diretório, não é lido por nenhuma fonte e não
  entra na varredura de shards (extensão `.jsonl` em `events/` não é memória).
- O nome do shard é derivado do `agent_id` por `shard_file_name`: ids seguros
  (`[A-Za-z0-9._-]`, ≤ 96 chars, ≠ `.`/`..`) viram `<agent_id>.jsonl`; qualquer outro vira
  `<prefixo-saneado>-<hex8>.jsonl`, com `hex8` = SHA-256 do **id original** truncado a 4 bytes.

## Consequências

- Escritores não contendem: cada append toca **o seu** ficheiro. A leitura concorrente não precisa
  de lock (D223).
- Conflitos entre agentes são possíveis (o mesmo `id` em dois shards) e resolvidos de forma
  determinística na leitura — o preço de não haver manifesto nem coordenador.
- Renomear um `agent_id` cria um shard novo; o antigo continua a ser lido até ser removido (o
  knudge nunca apaga a fonte — ela é do agente).
- O nome saneado é **determinístico e sem colisões práticas**, e o hash preserva a identidade
  original (`DIVERGENCES.md` #121).
- Um `session-log` é apenas mais uma área de shards: o contrato de projeção (`activity=derive`,
  D218) não muda.
