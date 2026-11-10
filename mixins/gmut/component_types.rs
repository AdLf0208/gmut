// Before content loads: register the component id the unit's item declares.
fn observe_component_types(types: &mut Vec<String>) {
    crate::game::gmut_state::on_component_types(types);
}
