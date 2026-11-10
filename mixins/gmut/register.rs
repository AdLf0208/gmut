// Declare `/gmut info|mode|next|prev`.
fn observe_register(commands: &mut Vec<NativeCommand>) {
    crate::game::gmut_state::on_register_commands(commands);
}
