// Right click with the unit: cycle its vertical-speed step.
fn observe_use_item(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    claimed: &mut bool,
) {
    crate::game::gmut_state::on_use_item(game, stack, claimed);
}
