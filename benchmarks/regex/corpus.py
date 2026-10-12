#!/usr/bin/env python3
"""Generate the deterministic regex benchmark corpus.

    python3 benchmarks/regex/corpus.py OUT [--size-mb 16] [--seed 1]

About 7/8 of the file is English-like ASCII prose with planted dates, emails and names; the last
1/8 is Unicode-heavy (Greek, Cyrillic, CJK and accented Latin). The text is made from a built-in
word list and a seeded generator, so every run on every machine writes the same bytes. The words
"hello" and "zzz" never occur (two cases rely on that). Combining marks are never used, so the
engines' different \\w definitions agree on the text.
"""
from __future__ import annotations

import argparse
import hashlib
import random
import sys

ENGLISH = """the of and to in a is that it was for on are as with his they at be this from have or by one had not
but what all were when we there can an your which their said if do will each about how up out them then she
many some so these would other into has more her two like him see time could no make than first been its who
now people my made over did down only way find use may water long little very after words called just where
most know get through back much before go good new write our me man too any day same right look think also
around another came come work three word must because does part even place well such here take why help put
different away again off went old number great tell men say small every found still between name should home
big give air line set own under read last never us left end along while might next sound below saw something
thought both few those always looked show large often together asked house don't world going want school
important until form food keep children feet land side without boy once animals life enough took sometimes
four head above kind began almost live page got earth need far hand high year mother light country father
let night picture being study second soon story since white ever paper hard near sentence better best across
during today however sure knew try told young sun thing whole hear example heard several change answer room
sea against top turned learn point city play toward five using himself usually running singing walking
morning evening reading building sleeping thinking drawing painting listening wandering talking following
inspector detective street garden window river market letter station evidence mystery journey""".split()

NAMES = ["Sherlock", "Holmes", "Watson", "Irene", "Adler", "Mycroft", "Lestrade", "Moriarty", "Hudson"]
DOMAINS = ["example", "mail", "corp", "lab", "inbox", "post"]
USERS = ["alice", "bob", "carol", "dave", "erin", "frank", "grace", "heidi", "ivan", "judy", "kate_9", "l33t"]

GREEK = """και του της των στο την που για από με είναι ένα αυτό πολύ όταν μέρα νύχτα θάλασσα άνθρωπος ζωή
φως κόσμος αγάπη ελευθερία σοφία ιστορία μουσική πόλη βιβλίο ταξίδι όνειρο""".split()
CYRILLIC = """и в не на я быть он с что а по это она этот к но они мы как из у который то за свой что весь год
мир человек жизнь день рука работа слово место лицо друг город земля вода книга дорога ночь""".split()
CJK = """我们的人在有他这中大来上国个到说你们地也子时道出而要于就下得可你年生自会那后能对着事其里所去行过家十用发天如然作方成者多日都三小军二无同么经法当起与好看学进种将还分此心前面又定见只主没公从""".replace("\n", "")
LATIN = """café naïve résumé façade crème brûlée jalapeño piñata über Straße Mädchen größer señor niño año
été à côté où déjà fenêtre forêt hôtel île Zürich Åland smørrebrød Łódź Dvořák Çağrı Ñandú Öl""".split()


def ascii_section(rng: random.Random, size: int) -> str:
    out: list[str] = []
    total = 0
    words = ENGLISH
    while total < size:
        parts: list[str] = []
        for _ in range(rng.randint(6, 18)):
            r = rng.random()
            if r < 0.0045:
                parts.append(f"{rng.randint(1990, 2029):04d}-{rng.randint(1, 12):02d}-{rng.randint(1, 28):02d}")
            elif r < 0.0075:
                parts.append(f"{rng.choice(USERS)}@{rng.choice(DOMAINS)}.com")
            elif r < 0.0100:
                parts.append(rng.choice(NAMES))
            elif r < 0.0112:
                parts.append(f"{rng.choice(USERS)}.{rng.randint(1, 99)}@{rng.choice(DOMAINS)}.org")
            else:
                parts.append(rng.choice(words))
        line = " ".join(parts)
        line = line[0].upper() + line[1:] + rng.choice([".", ".", ".", "!", "?", ","]) + "\n"
        out.append(line)
        total += len(line)
        if rng.random() < 0.05:
            out.append("\n")
            total += 1
    return "".join(out)


def unicode_section(rng: random.Random, size: int) -> str:
    out: list[str] = []
    total = 0
    while total < size:
        parts: list[str] = []
        for _ in range(rng.randint(6, 16)):
            r = rng.random()
            if r < 0.30:
                parts.append(rng.choice(GREEK))
            elif r < 0.55:
                parts.append(rng.choice(CYRILLIC))
            elif r < 0.75:
                n = rng.randint(2, 6)
                parts.append("".join(rng.choice(CJK) for _ in range(n)))
            elif r < 0.93:
                parts.append(rng.choice(LATIN))
            elif r < 0.96:
                parts.append(f"{rng.randint(1990, 2029):04d}-{rng.randint(1, 12):02d}-{rng.randint(1, 28):02d}")
            else:
                parts.append(rng.choice(ENGLISH))
        line = " ".join(parts) + rng.choice([".", "。", "!", ";"]) + "\n"
        out.append(line)
        total += len(line.encode())
    return "".join(out)


def generate(size_mb: float, seed: int) -> bytes:
    rng = random.Random(seed)
    total = int(size_mb * 1024 * 1024)
    uni = total // 8
    text = ascii_section(rng, total - uni) + unicode_section(rng, uni)
    data = text.encode("utf-8")
    assert b"zzz" not in data and b"hello" not in data.lower(), "corpus must not contain zzz or hello"
    return data


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("out")
    ap.add_argument("--size-mb", type=float, default=16)
    ap.add_argument("--seed", type=int, default=1)
    args = ap.parse_args()
    data = generate(args.size_mb, args.seed)
    with open(args.out, "wb") as f:
        f.write(data)
    print(f"wrote {args.out}: {len(data)} bytes, sha256 {hashlib.sha256(data).hexdigest()[:16]}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
