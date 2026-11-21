#!/usr/bin/env python3
"""One-off: wrap the community's hardcoded Chinese launcher strings with i18n::tr.

The community commit replaced English string literals with Chinese in place.
This reads that commit's diff to recover (English, Chinese) pairs, then rewrites
only the Chinese literals in the working tree as

    crate::ha_launcher::i18n::tr("English original", "中文")

so a Chinese system shows Chinese and every other system shows the English
original. Internal tokens (paths, match patterns, identifiers where en == zh),
placeholders and format-style macro arguments are left alone.
"""
import difflib
import json
import os
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASE = "ff303502a4f7982551a0e0d3622d9268efe9e829"
COMMIT = "20f72626f6"
STR = re.compile(r'"((?:[^"\\]|\\.)*)"')
HAN = re.compile(r"[\u4e00-\u9fff]")
MACROS = ("format!", "format_args!", "write!", "writeln!", "print!", "println!",
          "eprint!", "eprintln!", "panic!", "assert!", "assert_eq!", "assert_ne!",
          "debug_assert!", "todo!", "unimplemented!", "matches!", "info!", "warn!",
          "error!", "debug!", "trace!")
CALL = 'crate::ha_launcher::i18n::tr'


def strings(line):
    return STR.findall(line)


def mapping_for(path):
    old = subprocess.run(["git", "show", f"{BASE}:{path}"], cwd=ROOT,
                         capture_output=True, text=True).stdout.splitlines()
    new = subprocess.run(["git", "show", f"{COMMIT}:{path}"], cwd=ROOT,
                         capture_output=True, text=True).stdout.splitlines()
    mapping = {}
    for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, old, new).get_opcodes():
        if tag != "replace":
            continue
        for a, b in zip(old[i1:i2], new[j1:j2]):
            en, zh = strings(a), strings(b)
            if len(en) != len(zh):
                continue
            for e, z in zip(en, zh):
                if e == z or not HAN.search(z) or any(c in z for c in "{}%"):
                    continue
                z = z.replace('\\"', '"')
                e = e.replace('\\"', '"')
                prev = mapping.get(z)
                if prev is None or prev == e:
                    mapping[z] = e
    return mapping


def in_macro(text, pos):
    line_start = text.rfind("\n", 0, pos) + 1
    prefix = text[line_start:pos]
    for m in MACROS:
        at = prefix.rfind(m)
        if at >= 0 and prefix[at + len(m):].startswith("("):
            call = prefix[at + len(m) + 1:]
            if call.count("(") >= call.count(")"):
                return True
    return False


def main():
    files = subprocess.run(["git", "diff", "--name-only", f"{BASE}..{COMMIT}"],
                           cwd=ROOT, capture_output=True, text=True).stdout.splitlines()
    files = [f for f in files if f.endswith(".rs")]
    total = replaced = 0
    per_file = {}
    for path in files:
        mapping = mapping_for(path)
        if not mapping:
            continue
        target = ROOT / path
        text = target.read_text(encoding="utf-8")

        def repl(m):
            nonlocal replaced
            zh = m.group(1).replace('\\"', '"')
            if zh not in mapping or in_macro(text, m.start()):
                return m.group(0)
            en = mapping[zh]
            replaced += 1
            return f'{CALL}({json.dumps(en, ensure_ascii=False)}, {json.dumps(zh, ensure_ascii=False)})'

        new_text = STR.sub(repl, text)
        total += len(mapping)
        if new_text != text:
            if not os.environ.get("DRY"):
                target.write_text(new_text, encoding="utf-8")
            per_file[path] = sum(1 for m in STR.finditer(text)
                                 if m.group(1).replace('\\"', '"') in mapping)
    print(f"mapped={total} replaced={replaced}")
    for path, n in sorted(per_file.items(), key=lambda x: -x[1]):
        print(f"  {n:4d}  {path}")


if __name__ == "__main__":
    main()
