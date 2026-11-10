# gmut (Horizon port)

A Horizon port of [gmut](gisellevonbingen-Minecraft/GravitationalModulatingAdditionalUnit) — MIT.

**Status: first slices.** What is below is what the port actually does today.

## Implemented so far

- The item.** `gmut:module_gravitational_modulating_additional_unit`, the
- `minecraft:paper` template with `max_stack_size: 1` and `rarity: rare`
- (upstream's `maxStackSize(1).rarity(Rarity.RARE)`), with the initial
- `gmut:vertical_speed: "off"` component.
- The step.** Upstream's `VerticalSpeed` (`OFF 1.0`, `LOW 1.5`, `MEDIUM 2.0`,
- `HIGH 2.5`, `ULTRA 3.0`), kept **per stack** in the item's own
- `gmut:vertical_speed` component and cycled with `MathUtils.getByIndexMod`'s
- The toggles.** A right click (`native_use_item`, API 13) turns the step

## Not yet ported

- `blocked(no-attribute-api)` — flight, gravity and the vertical-speed
- effect.** Upstream's `fly_always` sets `abilities.flying`, and
- `vertical_speed` adds `j * (speed - 1) * flyingSpeed * 3` to the player's
- delta movement while flying. The pin has no attribute/abilities write and no
- per-tick velocity helper for the acting player, so both are unimplemented:
- the mode is stored and read out, never applied.
- `blocked(no-mekasuit)` — the gate.** Upstream's
- `CommonPlayerTickHandler.isGravitationalModulationReady` needs a worn

## Install

Build the bundle with a native-mod capable launcher, or add the file from the
registry once it is listed. Pinned to Horizon `1d63e0a` and horizon-mixins
`5a64af93` for game 26.3.

## Build from source

```sh
cargo test
python3 tools/horizon.py check --horizon <horizon checkout>
python3 tools/horizon.py build --horizon <horizon checkout>
python3 tools/package.py
```

## Credits and licence

Original mod: gisellevonbingen-Minecraft/GravitationalModulatingAdditionalUnit. Port: AdLf0208. Licence: MIT — see [LICENSE](LICENSE);
the original notices are preserved.
