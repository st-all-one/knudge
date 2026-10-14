#!/usr/bin/env python3
"""Análise pareada da bateria A/B de embeddings (stdlib, sem deps).

Lê os `.tsv` de resultados (rank do primeiro relevante por consulta, por família) gerados por
`ab-embeddinggemma2 --dump` e produz:
  - tabela de R@1 e MRR por cenário/família;
  - diferença pareada vs. o cenário de referência com IC95% por bootstrap.

Uso:  python3 analyze.py
"""
import os
import random

HERE = os.path.dirname(os.path.abspath(__file__))
RES = os.path.join(HERE, "results")

# Ordem dos cenários na tabela (o primeiro é a referência).
SCEN = [
    "granite384",
    "gemma768",
    "gemma384",
    "gemma768p",
    "gemma384p",
    "gemma768pt",
    "gemma384pt",
]
FAMS = ["self", "kw", "body", "anchor", "tag"]


def load(path):
    out = {f: [] for f in FAMS}
    with open(path) as fh:
        for line in fh:
            fam, rank = line.rstrip("\n").split("\t")
            out[fam].append(int(rank))
    return out


data = {s: load(os.path.join(RES, f"{s}.tsv")) for s in SCEN}


def r1(ranks):
    return sum(1 for r in ranks if r == 0) / len(ranks)


def mrr(ranks):
    return sum(1.0 / (r + 1) for r in ranks) / len(ranks)


print("== R@1 por família ==\n")
hdr = f"{'família':8s} {'n':>5s} " + " ".join(f"{s:>11s}" for s in SCEN)
print(hdr)
for fam in FAMS:
    vals = [r1(data[s][fam]) for s in SCEN]
    n = len(data[SCEN[0]][fam])
    print(f"{fam:8s} {n:5d} " + " ".join(f"{v*100:10.2f}%" for v in vals))

print("\n== MRR por família ==\n")
print(hdr)
for fam in FAMS:
    vals = [mrr(data[s][fam]) for s in SCEN]
    n = len(data[SCEN[0]][fam])
    print(f"{fam:8s} {n:5d} " + " ".join(f"{v:11.4f}" for v in vals))

random.seed(20261014)


def boot_diff(a, b, f, B=2000):
    """IC95% da diferença b-a por bootstrap pareado."""
    n = len(a)
    diffs = []
    for _ in range(B):
        sa, sb = 0.0, 0.0
        for _ in range(n):
            i = random.randrange(n)
            sa += f(a[i])
            sb += f(b[i])
        diffs.append((sb - sa) / n)
    diffs.sort()
    return diffs[int(0.025 * B)], diffs[int(0.975 * B)]


def f_r1(r):
    return 1.0 if r == 0 else 0.0


def f_mrr(r):
    return 1.0 / (r + 1)


print(f"\n== Diferença pareada (ponto [IC95% bootstrap]) vs {SCEN[0]} ==\n")
base = dict(data[SCEN[0]])
for s in SCEN[1:]:
    print(f"--- {s} ---")
    for fam in FAMS:
        a, b = base[fam], data[s][fam]
        d1 = r1(b) - r1(a)
        lo1, hi1 = boot_diff(a, b, f_r1)
        dm = mrr(b) - mrr(a)
        lom, him = boot_diff(a, b, f_mrr)
        win = sum(1 for x, y in zip(a, b) if y < x)
        tie = sum(1 for x, y in zip(a, b) if y == x)
        loss = sum(1 for x, y in zip(a, b) if y > x)
        print(
            f"  {fam:8s} ΔR@1={d1*100:+6.2f}pp [{lo1*100:+6.2f},{hi1*100:+6.2f}] | "
            f"ΔMRR={dm:+.4f} [{lom:+.4f},{him:+.4f}] | W/T/L={win}/{tie}/{loss}"
        )
