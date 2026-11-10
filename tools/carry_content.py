#!/usr/bin/env python3
"""Carry Gravitational Modulating Additional Unit's data and assets into content/.

The jar is the only source (never decompiled, never re-shipped): this script
extracts the item's model, its texture, its names and its recipe/advancement,
and writes the two files 26.3 needs that the 1.21.1 jar does not have (the
`assets/<id>/items/<name>.json` item definition, and Horizon's
`data/<id>/horizon/item/<name>.json` template declaration).

    python3 tools/carry_content.py            # write content/
    python3 tools/carry_content.py --check    # list what it would write

Decisions (they differ from the jar on purpose) are recorded in CONTENT.md.
"""
from __future__ import annotations

import argparse
import json
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
JAR = Path(
    ROOT.parent
    / "_shared/licence-cache/GravitationalModulatingAdditionalUnit-1.21.1-6.4.jar"
)
CONTENT = ROOT / "content"

MOD = "gmut"
ITEM = "module_gravitational_modulating_additional_unit"

# The jar's own files this mod's item needs, and nothing else.
CARRIED = [
    f"assets/{MOD}/models/item/{ITEM}.json",
    f"assets/{MOD}/textures/item/{ITEM}.png",
    f"assets/{MOD}/lang/en_us.json",
    f"data/{MOD}/recipe/{ITEM}.json",
    f"data/{MOD}/advancement/recipes/misc/{ITEM}.json",
]

# Keys the 1.21.1 jar spells for NeoForge's `ModuleItem`; 26.3's item name is
# `item.<namespace>.<path>` (creative.rs), and this port's tooltip and tab are
# its own, so those three are written here instead of being carried.
LANG_ADDED = {
    f"item.{MOD}.{ITEM}": "Gravitational Modulating Additional Unit",
    "itemGroup.gmut": "Gravitational Modulating Additional Unit",
    "gmut.tooltip.mode": "Vertical Speed Modifier: %s",
    "gmut.tooltip.modulation": "Requires a Gravitational Modulating Unit",
}

# The item copies a plain item: no blocks_attacks, no food, no durability. One
# per stack and RARE as upstream's `ModuleRegistryObject` builder says
# (`maxStackSize(1).rarity(Rarity.RARE)`); the step lives in the component the
# native half registers (`gmut:vertical_speed`, `VerticalSpeed.OFF` first).
ITEM_DECLARATION = {
    "template": "minecraft:paper",
    "max_stack_size": 1,
    "rarity": "rare",
    "components": {f"{MOD}:vertical_speed": "off"},
}

ITEM_DEFINITION = {
    "model": {
        "type": "minecraft:model",
        "model": f"{MOD}:item/{ITEM}",
    }
}

CREATIVE_TAB = {
    "title": f"itemGroup.{MOD}",
    "icon": f"{MOD}:{ITEM}",
    "items": [f"{MOD}:{ITEM}"],
}


def carried() -> dict[str, bytes]:
    archive = zipfile.ZipFile(JAR)
    return {name: archive.read(name) for name in CARRIED}


def language(jar_lang: bytes) -> bytes:
    names = json.loads(jar_lang)
    names.update(LANG_ADDED)
    return (json.dumps(dict(sorted(names.items())), indent=2, ensure_ascii=False) + "\n").encode()


def outputs() -> dict[str, bytes]:
    files = carried()
    names = {name: data for name, data in files.items() if not name.endswith("en_us.json")}
    names[f"assets/{MOD}/lang/en_us.json"] = language(files[f"assets/{MOD}/lang/en_us.json"])
    names[f"assets/{MOD}/items/{ITEM}.json"] = (
        json.dumps(ITEM_DEFINITION, indent=2) + "\n"
    ).encode()
    names[f"data/{MOD}/horizon/item/{ITEM}.json"] = (
        json.dumps(ITEM_DECLARATION, indent=2) + "\n"
    ).encode()
    names[f"data/{MOD}/horizon/creative_tab/{MOD}.json"] = (
        json.dumps(CREATIVE_TAB, indent=2) + "\n"
    ).encode()
    return dict(sorted(names.items()))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="list, write nothing")
    args = parser.parse_args()
    if not JAR.is_file():
        raise SystemExit(f"missing jar: {JAR}")
    files = outputs()
    for name, data in files.items():
        if args.check:
            print(f"{name} {len(data)} B")
            continue
        path = CONTENT / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
    if not args.check:
        print(f"wrote {len(files)} files under {CONTENT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
