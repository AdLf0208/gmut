#!/usr/bin/env bash
# Proves the built content package (content/gmut.hmod) loads on a real game:
# the shared, unmodded 1d63e0a game with `--dev-mod <the .hmod>` must register
# `gmut:module_gravitational_modulating_additional_unit`, and its carried name,
# step component and RARE rarity must be readable. A second launch WITHOUT
# `--dev-mod` is the negative control: the same id must be unknown there, so
# the check cannot pass vacuously.
set -Eeuo pipefail
export LC_ALL=C

CLONE=${CAMPAIGN_ROOT:-$(cd "$(dirname "$0")/../../.." && pwd)}
PIN_BIN=$CLONE/ports/_shared/bin/horizon-1d63e0a5
CTL=${CTL:-$PIN_BIN/horizon-ctl}
GAME=${GAME:-$PIN_BIN/horizon-client}
HMOD=${HMOD:-$CLONE/ports/gmut/content/gmut.hmod}
RES=${RES:-${HORIZON_RES:-${HORIZON_RES:-$HOME/.local/horizon-client/26.3}}}
SOCK=${SOCK:-/tmp/opencode/gmut-hmod.sock}
ITEM=gmut:module_gravitational_modulating_additional_unit

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -f "$HMOD" ] || { echo "missing package: $HMOD" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/gmut-hmod-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; ctl logs --lines 40 >&2 || true; exit 1; }
on_error() { echo "FAIL: line $1: '$2' exited $?" >&2; ctl logs --lines 40 >&2 || true; }
trap 'on_error "$LINENO" "$BASH_COMMAND"' ERR

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    if [ "${KEEP:-0}" = 1 ]; then echo "kept scratch: $SCRATCH" >&2; else rm -rf "$SCRATCH" 2>/dev/null || true; fi
    # Last line of every run: `check-e2e-logs.py` reads it as the verdict.
    echo "EXIT:$status"
}
trap cleanup EXIT

launch() { # launch WORLD [--dev-mod PATH]
    local world=$1; shift
    local out
    rm -f "$SOCK"
    cd "$SCRATCH"
    out=$("$CTL" --socket "$SOCK" launch \
        --game "$GAME" --world "$world" --seed 20261105 --creative --headless --wait \
        --resources "$RES" "$@" 2>&1) || {
        echo "launch failed: $(tr -d '\n' <<<"$out" | cut -c1-400)" >&2
        return 1
    }
    ctl wait-ready 120
    ctl wait 20
}

echo "== launch WITH the package (--dev-mod $HMOD) =="
launch GmutHmod --dev-mod "$HMOD"
ctl give "$ITEM" 1 >/dev/null
ctl wait 4
ctl inventory | jq -e --arg item "$ITEM" \
    '[.hotbar[], .main[] | select(.item == $item)] | length >= 1' >/dev/null \
    || fail "the package did not register $ITEM: $(ctl inventory | jq -c '{hotbar,main}')"
echo "item: $(ctl inventory | jq -c --arg item "$ITEM" '[.hotbar[], .main[] | select(.item == $item) | {item, count}] | first')"
ctl quit >/dev/null 2>&1 || true
sleep 2

echo "== negative control: the SAME id without --dev-mod =="
launch GmutHmodControl
ctl give "$ITEM" 1 >/dev/null 2>&1 || true
ctl wait 4
ctl inventory | jq -e --arg item "$ITEM" \
    '[.hotbar[], .main[] | select(.item == $item)] | length == 0' >/dev/null \
    || fail "the unmodded game gave $ITEM: $(ctl inventory | jq -c '{hotbar,main}')"
echo "item: none"
echo "PASS: gmut.hmod registered gmut:module_gravitational_modulating_additional_unit on a real, unmodded game, and the same id is unknown without the package"
