// The unit's tooltip: its vertical-speed step and the gate it sits behind.
fn observe_tooltip(
    game: &crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    lines: &mut Vec<(Text, u32)>,
) {
    crate::game::gmut_state::on_tooltip(game, stack, lines);
}
