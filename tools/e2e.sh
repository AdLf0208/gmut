#!/usr/bin/env bash
# In-game check for Gravitational Modulating Additional Unit slice 1: the unit
# item registers, its vertical-speed step toggles (right click, sneak+right
# click, and `/gmut`), it is kept per stack, and `/gmut info` reports the
# placement, the rules and the gate the effects sit behind.
#
# Every mod command is typed with `ctl chat "/cmd"` because `ctl command` runs
# the console path and bypasses the native `perform_command` hook.
#
# What the pin does not allow is asserted as such: the gate reads
# `needs_gravitational_modulating_unit` (Mekanism is unported, and the pin has
# no attribute API to grant flight), so the scenario checks the readout rather
# than a flight effect that cannot happen.
#
# Default run:
#  A  no unit -> `/gmut info` answers `absent`, mode off (the command works
#     without the item);
#  B  one unit held -> `held`, mode off (the jar's OFF default);
#  C  right click -> `low` and the module's own mode-change line;
#  D  sneak + right click -> back to `off` (upstream's keybind turns back);
#  E  `/gmut mode high|next|next|prev` -> high, ultra, off (wrap), ultra (wrap);
#  F  a second unit is a separate stack: it still reads `off` while the first
#     is `ultra`, and each keeps its own step across a slot switch;
#  G  with both in the pack and neither held -> `carried(2)`;
#  H  an unknown `gmut:` id is not an item (the namespace is the mod's, not a
#     wildcard).
# The control is the same run with the embedded content unwritable: the id is
# unknown and `/gmut info` says `absent`.
#
# Default run (EMBEDDED=1): the game is launched with **only the native bundle**
# (no `--dev-mod`) and the content comes from the bundle's embedded copy
# (`mixins/gmut/data/gmut-content.hmod`, the AE2 pattern). The run also checks the
# file the helper wrote, and CONTROL=1 (default with EMBEDDED) first launches the
# same binary with `.horizon-cache` unusable, where the item id must be unknown.
# EMBEDDED=0 loads `content/` as a dev mod instead (the pre-embedding shape).
#
# SELFTEST=1 flips C's expected mode (it must FAIL), proving the scenario can
# fail.
set -Eeuo pipefail
export LC_ALL=C
[ -z "${GMUT_DEBUG:-}" ] || set -x

EMBEDDED=${EMBEDDED:-1}
CONTROL=${CONTROL:-$EMBEDDED}

CLONE=${CAMPAIGN_ROOT:-$(cd "$(dirname "$0")/../../.." && pwd)}
PORT=$CLONE/ports/gmut
PIN_BIN=$CLONE/ports/_shared/bin/horizon-1d63e0a5
CTL=${CTL:-$PIN_BIN/horizon-ctl}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
RES=${RES:-${HORIZON_RES:-${HORIZON_RES:-$HOME/.local/horizon-client/26.3}}}
SOCK=${SOCK:-/tmp/opencode/gmut-e2e.sock}
SELFTEST=${SELFTEST:-0}
KEEP=${KEEP:-0}
UNIT=gmut:module_gravitational_modulating_additional_unit

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/gmut-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
say() { ctl chat "/$1" >/dev/null; }
fail() { echo "FAIL: $*" >&2; ctl logs --lines 60 >&2 || true; exit 1; }
on_error() { echo "FAIL: line $1: '$2' exited $?" >&2; ctl logs --lines 40 >&2 || true; }
trap 'on_error "$LINENO" "$BASH_COMMAND"' ERR

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    if [ "$KEEP" = 1 ]; then
        echo "kept scratch: $SCRATCH" >&2
    else
        rm -rf "$SCRATCH" 2>/dev/null || true
    fi
    # Last line of every run: `check-e2e-logs.py` reads it as the verdict.
    echo "EXIT:$status"
}
trap cleanup EXIT

launch() { # launch WORLD
    local world=$1
    rm -f "$SOCK"
    cd "$SCRATCH"
    local args=(--game "$GAME" --world "$world" --seed 20261105 --creative --headless
                --wait --resources "$RES")
    if [ "$EMBEDDED" = 1 ]; then
        echo "  launch: no --dev-mod; the bundle carries its own content" >&2
    else
        args+=(--dev-mod "$PORT/content")
    fi
    "$CTL" --socket "$SOCK" launch "${args[@]}" >/dev/null
    ctl wait-ready 120 >/dev/null
    ctl wait 40 >/dev/null
}

# `/gmut info`'s line, quoted and parsed.
info() {
    say "gmut info"
    ctl wait 8 >/dev/null
    ctl messages 16 | jq -r '.chat[]' | grep -m1 'gmut: ' || true
}
mode_of() { sed -n 's/.*mode=\([a-z][a-z]*\) .*/\1/p' <<<"$1"; }
expect_info() { # expect_info FRAGMENT
    local line
    line=$(info)
    echo "  $line" >&2
    [ -n "$line" ] || fail "the unit answered no info line at all"
    grep -qF -- "$1" <<<"$line" || fail "the info line lacks '$1': $line"
}
expect_mode() { # expect_mode MODE
    local line mode
    line=$(info)
    echo "  $line" >&2
    mode=$(mode_of "$line")
    [ -n "$mode" ] || fail "the info line carries no mode: $line"
    [ "$mode" = "$1" ] || fail "the mode is '$mode', expected '$1'"
}

# The mode-change line the item prints is upstream's `displayModeChange`. A
# use-hook reply reaches the client's chat display as a `[CHAT]` line but not
# the control socket's message list (`ctl messages` carries command replies),
# so it is read back from the game log, newest last.
use_message() {
    ctl logs --lines 120 | grep -o 'Vertical Speed Modifier: [a-z]* ([0-9.]*x)' | tail -1 || true
}
# A right click of the held unit, aimed at the sky (the item's own use).
use_unit() {
    local out
    ctl look 0 -90 >/dev/null           # aim at the sky: the item's own use
    ctl wait 2 >/dev/null
    out=$(ctl use-item 2>&1) || true
    echo "  use-item: $(tr -d '\n' <<<"$out" | cut -c1-120)" >&2
    ctl wait 8 >/dev/null
}
expect_mode_line() { # expect_mode_line MODE LABEL
    local line
    line=$(use_message)
    echo "  message: ${line:-<none>}" >&2
    [ "$line" = "Vertical Speed Modifier: $1 ($2)" ] \
        || fail "the mode-change line is '${line:-<none>}', expected 'Vertical Speed Modifier: $1 ($2)'"
}
# The unit stacks the inventory holds, by slot.
units() { ctl inventory | jq -c --arg item "$UNIT" '[.hotbar[] | select(.item == $item) | {slot, count}]'; }

# The control for EMBEDDED=1: the same binary (every hook is in it) with the
# embedded content made unavailable — `.horizon-cache` is a plain file, so the
# helper can neither compare nor write its content file and falls back to
# `Continue`. The id must then be unknown; if it resolves, the run proves nothing
# about the embedded file.
control_leg() {
    echo "== control: the same game with no embedded content file =="
    rm -rf "$SCRATCH/.horizon-cache"
    : > "$SCRATCH/.horizon-cache"
    launch GmutE2EControl
    ctl give "$UNIT" 1 >/dev/null 2>&1 || true
    ctl wait 4 >/dev/null
    ctl inventory | jq -e --arg item "$UNIT" '[.hotbar[], .main[] | select(.item == $item)] | length == 0' >/dev/null \
        || fail "the control game has $UNIT, so the content did not come from the embedded file"
    expect_info "gmut: absent, mode=off (1.0x)"
    [ -f "$SCRATCH/.horizon-cache" ] && [ ! -s "$SCRATCH/.horizon-cache" ] \
        || fail "the control's .horizon-cache was replaced: the helper wrote its content"
    echo "  without the embedded file: no unit item, the reading is absent"
    ctl quit >/dev/null 2>&1 || true
    rm -f "$SCRATCH/.horizon-cache"
    sleep 2
}

# EMBEDDED=1: the bundle must have written its content out at startup, byte for
# byte the delivered content mod.
embedded_content_check() {
    local cache=$SCRATCH/.horizon-cache/gmut-embedded-content.hmod
    [ -f "$cache" ] || fail "the bundle did not write its embedded content to $cache"
    cmp -s "$cache" "$PORT/content/gmut.hmod" \
        || fail "the embedded content differs from content/gmut.hmod"
    echo "  embedded content: $(wc -c < "$cache" | tr -d ' ') B at $cache, identical to content/gmut.hmod"
}

echo "== launch (Overworld, one native bundle; no --dev-mod) =="
if [ "$CONTROL" = 1 ]; then control_leg; fi
launch GmutE2E
if [ "$EMBEDDED" = 1 ]; then embedded_content_check; fi

echo "== A: no unit yet -> absent =="
expect_info "gmut: absent, mode=off (1.0x)"

echo "== B: the unit held -> held, the jar's OFF default =="
ctl give "$UNIT" 1 >/dev/null
ctl wait 4 >/dev/null
ctl select "$UNIT" >/dev/null
ctl wait 4 >/dev/null
ctl inventory | jq -e --arg item "$UNIT" '[.hotbar[] | select(.item == $item)] | length == 1' >/dev/null \
    || fail "the unit item did not register: $(ctl inventory | jq -c '{hotbar,main}')"
echo "  unit: $(units)"
expect_info "gmut: held, mode=off (1.0x)"

echo "== C: a right click turns the step forward =="
use_unit
expect_mode_line low "1.5x"
if [ "$SELFTEST" = 1 ]; then
    echo "(selftest: asserting the step went to high)"
    expect_mode high
else
    expect_mode low
fi

echo "== D: sneaking turns it back =="
ctl key shift down >/dev/null
ctl wait 2 >/dev/null
use_unit
expect_mode_line off "1.0x"
ctl key shift up >/dev/null
ctl wait 2 >/dev/null
expect_mode off

echo "== E: /gmut mode|next|prev, with both wraps =="
say "gmut mode high"
ctl wait 8 >/dev/null
expect_mode high
say "gmut next"
ctl wait 8 >/dev/null
expect_mode ultra
say "gmut next"
ctl wait 8 >/dev/null
expect_mode off
say "gmut prev"
ctl wait 8 >/dev/null
expect_mode ultra

echo "== F: a second unit is its own stack =="
ctl give "$UNIT" 1 >/dev/null
ctl wait 4 >/dev/null
echo "  units: $(units)"
ctl select 2 >/dev/null
ctl wait 4 >/dev/null
expect_info "gmut: held, mode=off (1.0x)"
say "gmut mode low"
ctl wait 8 >/dev/null
expect_mode low
ctl select 1 >/dev/null
ctl wait 4 >/dev/null
expect_mode ultra
ctl select 2 >/dev/null
ctl wait 4 >/dev/null
expect_mode low

echo "== G: both in the pack, none held -> carried(2) =="
ctl select 9 >/dev/null
ctl wait 4 >/dev/null
expect_info "gmut: carried(2)"

echo "== H: an unknown gmut: id is not an item =="
before=$(ctl inventory | jq -c '[.hotbar[], .main[] | select(.item != null)] | length')
ctl give gmut:module_gravitational_modulating_additional_bogus 1 >/dev/null 2>&1 || true
ctl wait 4 >/dev/null
after=$(ctl inventory | jq -c '[.hotbar[], .main[] | select(.item != null)] | length')
[ "$before" = "$after" ] || fail "an unknown gmut: id was given ($before -> $after items)"
echo "  unknown id changed nothing ($after stacks)"

echo "== quit =="
ctl quit >/dev/null 2>&1 || true
sleep 2

echo "PASS: one .hnmod with no --dev-mod installed the unit and its behaviour: the step turned forward on a right click and back while sneaking, /gmut mode|next|prev set and wrapped it, two units kept their own steps, an unheld pair read carried(2), an unknown gmut: id was refused, the gate reported needs_gravitational_modulating_unit throughout (no attribute API to grant flight), and with the embedded content unwritable the id was unknown"
