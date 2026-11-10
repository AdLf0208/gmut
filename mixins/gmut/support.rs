// Game-side glue for the gravitational modulating additional unit (injected
// into `crate::game` as `gmut_state`; it needs the game, so it is not part of
// the standalone library).
//
// Upstream's module keeps its mode in a Mekanism `ModuleConfigItem` on the
// MekaSuit stack, cycles it from a keybind packet, and gates every effect on
// `CommonPlayerTickHandler.isGravitationalModulationReady(player)`. This pin
// has no keybind→server path, no MekaSuit and no attribute API, so the slice
// keeps the mode as the item's own `gmut:vertical_speed` component, cycled by
// right click or `/gmut`, and reports the gate instead of faking it.

use horizon_client::items::ItemStack;
use horizon_client::text::Text;
use horizon_world::nbt::{JavaString, Tag};

use crate::game::gmut::{
    Gate, ITEM, MODE_COMPONENT, Placement, Reading, Rules, VerticalSpeed, gate, mode_line,
    shift_for,
};
use crate::game::native_server::{NativeCommand, NativeServerPlayer};

/// The item id the jar registers (`GMUTItems`/`GMUTModules`).
const UNIT: &str = ITEM;

fn is_unit(item: &str) -> bool {
    item == UNIT
}

fn is_unit_stack(stack: &ItemStack) -> bool {
    is_unit(&stack.item)
}

/// Upstream's gate (`CommonPlayerTickHandler.isGravitationalModulationReady`):
/// the worn suit carries a working gravitational modulating unit. Mekanism is
/// unported at this pin and there is no armor/module API to read, so the gate
/// is **always** closed; `/gmut info` says so instead of pretending otherwise.
fn modulation_ready() -> bool {
    false
}

/// The step a unit stack carries, OFF when it carries none.
fn read_mode(stack: &ItemStack) -> VerticalSpeed {
    match stack.data().and_then(|data| data.other(MODE_COMPONENT)) {
        Some(Tag::String(name)) => {
            VerticalSpeed::from_name(&name.to_string_lossy()).unwrap_or_default()
        }
        _ => VerticalSpeed::default(),
    }
}

/// Store a step on a unit stack, as the module's config item keeps it.
fn write_mode(stack: &mut ItemStack, mode: VerticalSpeed) {
    stack.data_mut().set_other(
        MODE_COMPONENT,
        Some(Tag::String(JavaString::from(mode.name()))),
    );
    stack.tidy();
}

/// How many units the player's 36 inventory slots and off hand hold.
///
/// Not `native_count_items`: that helper counts only *plain* stacks
/// (`plain()` requires `extra.is_none()`), and this item always carries its
/// `gmut:vertical_speed` component, so the helper would always answer 0.
fn carried_count(game: &crate::game::Game<'_>) -> u32 {
    game.inventory
        .iter()
        .take(36)
        .chain(std::iter::once(&game.offhand))
        .filter_map(Option::as_ref)
        .filter(|stack| is_unit_stack(stack))
        .map(|stack| stack.count)
        .sum()
}

/// Where the player's unit is: the acting hand, the off hand, or the pack.
fn placement(game: &crate::game::Game<'_>) -> Placement {
    if game.held().is_some_and(is_unit_stack) {
        return Placement::Held;
    }
    if game.offhand.as_ref().is_some_and(is_unit_stack) {
        return Placement::OffHand;
    }
    match carried_count(game) {
        0 => Placement::Absent,
        count => Placement::Carried(count),
    }
}

/// The reading `/gmut info` prints for the acting player.
fn reading(game: &crate::game::Game<'_>) -> Reading {
    let mode = game
        .held()
        .filter(|stack| is_unit_stack(stack))
        .or_else(|| game.offhand.as_ref().filter(|stack| is_unit_stack(stack)))
        .map_or(VerticalSpeed::default(), read_mode);
    Reading {
        placement: placement(game),
        mode,
        rules: Rules::new(),
        gate: gate(modulation_ready()),
    }
}

/// Run `change` on the acting player's held item and put the result back,
/// with the hand told the stack changed; false when the hand is empty.
fn change_held(
    game: &mut crate::game::Game<'_>,
    change: impl FnOnce(&ItemStack) -> ItemStack,
) -> bool {
    let slot = game.held_slot();
    let Some(stack) = game.stack_in_mut(slot).take() else {
        return false;
    };
    *game.stack_in_mut(slot) = Some(change(&stack));
    game.hand_stack_replaced(slot);
    true
}

/// Put `mode` on the held unit; the step it replaced, or `None` when the
/// player holds no unit.
fn set_held_mode(game: &mut crate::game::Game<'_>, mode: VerticalSpeed) -> Option<VerticalSpeed> {
    if !game.held().is_some_and(is_unit_stack) {
        return None;
    }
    let previous = game.held().map_or(VerticalSpeed::default(), read_mode);
    change_held(game, |stack| {
        let mut one = stack.clone();
        write_mode(&mut one, mode);
        one
    })
    .then_some(previous)
}

/// Right click with the unit (`native_use_item`, API 13): cycle the step, as
/// upstream's keybind does — forward, or back while sneaking.
#[allow(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_use_item(game: &mut crate::game::Game<'_>, stack: &ItemStack, claimed: &mut bool) {
    if !is_unit_stack(stack) {
        return;
    }
    *claimed = true;
    let shift = shift_for(game.native_sneaking());
    let next = read_mode(stack).adjust(shift);
    if set_held_mode(game, next).is_some() {
        game.native_feedback(Text::literal(mode_line(next)));
    }
}

/// The component id the carried item declares its step in.
pub fn on_component_types(types: &mut Vec<String>) {
    types.push(MODE_COMPONENT.to_owned());
}

/// The unit's tooltip: the step it carries and the gate its effects sit
/// behind (`stop_immediately` and the rest are the module's own defaults).
#[allow(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_tooltip(_game: &crate::game::Game<'_>, stack: &ItemStack, lines: &mut Vec<(Text, u32)>) {
    if !is_unit_stack(stack) {
        return;
    }
    let mode = read_mode(stack);
    let label = format!("{} ({:.1}x)", mode.name(), mode.multiplier());
    lines.push((Text::tr("gmut.tooltip.mode", &[&label]), 0xAAAAAA));
    if gate(modulation_ready()) == Gate::NeedsGravitationalModulatingUnit {
        lines.push((Text::tr("gmut.tooltip.modulation", &[]), 0xFF5555));
    }
}

/// `/gmut info|mode|next|prev`: the harness controls the in-game check drives.
/// Upstream has no commands; its keybind, its config items and the MekaSuit
/// GUI are the controls, and none of the three exists at this pin.
pub fn on_register_commands(commands: &mut Vec<NativeCommand>) {
    commands.push(NativeCommand::new("gmut info"));
    commands.push(NativeCommand::op("gmut mode <speed:word>", 2));
    commands.push(NativeCommand::op("gmut next", 2));
    commands.push(NativeCommand::op("gmut prev", 2));
}

/// The steps `/gmut mode` accepts, for its usage line.
fn step_list() -> String {
    VerticalSpeed::ALL
        .iter()
        .map(|step| step.name())
        .collect::<Vec<_>>()
        .join("|")
}

/// Run a `/gmut` command a player typed in chat.
#[allow(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_perform_command(
    game: &mut crate::game::Game<'_>,
    _player: &NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    let words: Vec<&str> = command.split_whitespace().collect();
    match words.as_slice() {
        ["gmut", "info"] => {
            *ran = true;
            let line = reading(game).line();
            game.native_feedback(Text::literal(line));
        }
        ["gmut", "mode", speed] => {
            *ran = true;
            let Some(mode) = VerticalSpeed::from_name(speed) else {
                game.native_feedback(Text::literal(format!(
                    "Usage: /gmut mode <{}>",
                    step_list()
                )));
                return;
            };
            report_mode(game, mode);
        }
        ["gmut", "next"] => {
            *ran = true;
            cycle(game, true);
        }
        ["gmut", "prev"] => {
            *ran = true;
            cycle(game, false);
        }
        _ => {}
    }
}

/// Turn the held unit one step (forward, or back) and report where it landed.
#[allow(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
fn cycle(game: &mut crate::game::Game<'_>, forward: bool) {
    let Some(current) = game
        .held()
        .filter(|stack| is_unit_stack(stack))
        .map(read_mode)
    else {
        game.native_feedback(Text::literal("Hold the unit first."));
        return;
    };
    let next = if forward {
        current.next()
    } else {
        current.previous()
    };
    report_mode(game, next);
}

/// Put `mode` on the held unit and say which step it landed on.
#[allow(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
fn report_mode(game: &mut crate::game::Game<'_>, mode: VerticalSpeed) {
    match set_held_mode(game, mode) {
        Some(_) => game.native_feedback(Text::literal(mode_line(mode))),
        None => game.native_feedback(Text::literal("Hold the unit first.")),
    }
}
