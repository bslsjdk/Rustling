#!/usr/bin/env python3
"""Bake the three static Noto Sans SC faces used by the Android launcher.

The bundled game text renderer uses ab_glyph, which reads a variable font's
outlines at their default weight. Noto Sans SC defaults to Thin, so the variable
font must not be substituted for these instances. Download the full official
source from google/fonts (ofl/notosanssc/NotoSansSC[wght].ttf, commit
2fe1e8c6b137fb13041a7cfca6fdcd8a910ea4dc, SHA-256
a3041811a78c361b1de50f953c805e0244951c21c5bd412f7232ef0d899af0da) and pass it
here. Only the three baked instances are committed, not the 17 MB source.

    python3 -m venv /tmp/ft && /tmp/ft/bin/pip install fonttools
    /tmp/ft/bin/python tools/bake_noto.py 'NotoSansSC[wght].ttf'

Source: google/fonts, ofl/notosanssc, SIL Open Font License 1.1
(crates/horizon-theme/fonts/OFL-NotoSansSC.txt).
"""
import sys
from pathlib import Path

if __name__ == "__main__" and len(sys.argv) != 2:
    sys.exit(__doc__)

from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

OUT = Path(__file__).resolve().parent.parent / "crates/horizon-theme/fonts"


def main():
    for style, weight in (("Regular", 400), ("Medium", 500), ("SemiBold", 600)):
        font = instantiateVariableFont(TTFont(sys.argv[1]), {"wght": weight})
        names = font["name"]
        names.names = [n for n in names.names if n.nameID not in (16, 17, 21, 22, 25)]
        full = f"Noto Sans SC {style}"
        ps = f"NotoSansSC-{style}"
        for nid, value in ((1, "Noto Sans SC"), (2, style), (3, ps), (4, full), (6, ps)):
            names.setName(value, nid, 3, 1, 0x409)
            names.removeNames(nameID=nid, platformID=1)
        os2 = font["OS/2"]
        os2.usWeightClass = weight
        os2.fsSelection = (os2.fsSelection & ~0x61) | (0x40 if weight == 400 else 0)
        font["head"].macStyle = 0
        out = OUT / f"Noto-{style}.ttf"
        font.save(out)
        print(f"{out.name}: {weight}, {out.stat().st_size // 1024} KiB")


if __name__ == "__main__":
    main()
