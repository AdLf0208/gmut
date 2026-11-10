//  The gravitational modulating additional unit's behaviour, as far as the pin
//  allows: the item's mode, the module's rule defaults and the gate the
//  behaviour sits behind.
//
//  Upstream (see `FINDINGS.md`) is a Mekanism MekaSuit module: a
//  `ModuleGravitationalModulatingAdditionalUnit` installed in the bodyarmor
//  with five config items (`fly_always`, `stop_immediately`, `fix_fov`,
//  `vertical_speed`, `mute_modulating_sound`) and a keybind that cycles
//  `vertical_speed` (+1, or -1 while sneaking). All of it is gated on
//  `CommonPlayerTickHandler.isGravitationalModulationReady(player)` — the suit
//  carries a working gravitational modulating unit.
//
//  At this pin there is **no attribute API** (no `AttributeInstance` write for
//  `minecraft:gravity`, no hook that grants flight to a player) and Mekanism
//  itself is unported, so the flight and vertical-speed effects cannot be
//  applied honestly. This slice lands what the pin does allow — the item, its
//  per-stack mode component, the mode toggle and the `/gmut` readout — and
//  records the rest as `blocked(no-attribute-api)` / `blocked(no-mekasuit)`.

/// The unit's item id, as the jar's content names it.
pub const ITEM: &str = "gmut:module_gravitational_modulating_additional_unit";

/// The component the item's chosen vertical-speed step lives in
/// (`ModuleConfigItem` `vertical_speed` upstream, saved with the module).
pub const MODE_COMPONENT: &str = "gmut:vertical_speed";

/// Upstream's `VerticalSpeed`: the vertical-flight multiplier steps, OFF to
/// ULTRA, cycled by the module's keybind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VerticalSpeed {
    /// 1.0x — no additional vertical speed.
    #[default]
    Off,
    /// 1.5x.
    Low,
    /// 2.0x.
    Medium,
    /// 2.5x.
    High,
    /// 3.0x.
    Ultra,
}

impl VerticalSpeed {
    /// Every step, in the order upstream's enum declares them.
    pub const ALL: [Self; 5] = [Self::Off, Self::Low, Self::Medium, Self::High, Self::Ultra];

    /// The step's position in [`Self::ALL`].
    pub const fn index(self) -> usize {
        match self {
            Self::Off => 0,
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Ultra => 4,
        }
    }

    /// The step's lower-case name, as the item component stores it.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Ultra => "ultra",
        }
    }

    /// The vertical-speed multiplier upstream gives this step.
    pub const fn multiplier(self) -> f32 {
        match self {
            Self::Off => 1.0,
            Self::Low => 1.5,
            Self::Medium => 2.0,
            Self::High => 2.5,
            Self::Ultra => 3.0,
        }
    }

    /// The step a component's name stands for, `None` when it names none.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|step| step.name() == name)
    }

    /// The step `shift` places further along the list, wrapping as upstream's
    /// `MathUtils.getByIndexMod` does.
    pub fn adjust(self, shift: i32) -> Self {
        let length = Self::ALL.len() as i32;
        let index = (self.index() as i32 + shift).rem_euclid(length);
        Self::ALL[index as usize]
    }

    /// The next step, wrapping from ULTRA to OFF.
    pub fn next(self) -> Self {
        self.adjust(1)
    }

    /// The previous step, wrapping from OFF to ULTRA.
    pub fn previous(self) -> Self {
        self.adjust(-1)
    }
}

/// The direction a mode switch turns, as upstream's keybind: sneaking goes
/// back one step, holding nothing first goes forward.
pub const fn shift_for(sneaking: bool) -> i32 {
    if sneaking { -1 } else { 1 }
}

/// The module's boolean config items and their defaults, as upstream's `init`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// `fly_always`: keep flying when not sneaking (default off).
    pub fly_always: bool,
    /// `stop_immediately`: stop dead when no direction is held (default on).
    pub stop_immediately: bool,
    /// `fix_fov`: keep the field of view steady while flying (default off).
    pub fix_fov: bool,
    /// `mute_modulating_sound`: silence the suit's modulating sound (default off).
    pub mute_modulating_sound: bool,
}

impl Rules {
    /// The defaults upstream registers: every flag off but `stop_immediately`.
    pub const DEFAULTS: Self = Self {
        fly_always: false,
        stop_immediately: true,
        fix_fov: false,
        mute_modulating_sound: false,
    };

    /// [`Self::DEFAULTS`], for `Default`.
    pub const fn new() -> Self {
        Self::DEFAULTS
    }
}

impl Default for Rules {
    fn default() -> Self {
        Self::new()
    }
}

/// Where the unit is, as the harness reports it: the pin has no MekaSuit
/// install surface, so the closest thing to "installed" is a hand or the pack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    /// In the acting hand.
    Held,
    /// In the off hand.
    OffHand,
    /// Carried elsewhere in the inventory, this many units.
    Carried(u32),
    /// Nowhere in the player's inventory.
    Absent,
}

impl Placement {
    /// The word `/gmut info` prints for this placement.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Held => "held",
            Self::OffHand => "off_hand",
            Self::Carried(_) => "carried",
            Self::Absent => "absent",
        }
    }

    /// The placement with its count, as the info line spells it.
    fn text(self) -> String {
        match self {
            Self::Carried(count) => format!("carried({count})"),
            other => other.name().to_owned(),
        }
    }
}

/// Whether the unit's effects can apply, as upstream's
/// `CommonPlayerTickHandler.isGravitationalModulationReady(player)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gate {
    /// The suit carries a working gravitational modulating unit.
    Ready,
    /// It does not (upstream's own gate, and this pin's only state: Mekanism
    /// is unported, so no suit module can be installed).
    NeedsGravitationalModulatingUnit,
}

impl Gate {
    /// The word `/gmut info` prints for this gate.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::NeedsGravitationalModulatingUnit => "needs_gravitational_modulating_unit",
        }
    }
}

/// The gate for a player whose suit has (or lacks) a working gravitational
/// modulating unit.
pub const fn gate(gravitational_modulator_ready: bool) -> Gate {
    if gravitational_modulator_ready {
        Gate::Ready
    } else {
        Gate::NeedsGravitationalModulatingUnit
    }
}

/// One `/gmut info` reading: where the unit is, its mode, the rule defaults
/// and the gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reading {
    /// Where the player's unit is.
    pub placement: Placement,
    /// The step the stack carries.
    pub mode: VerticalSpeed,
    /// The module's rule flags (their defaults at this pin).
    pub rules: Rules,
    /// Whether the effects could apply.
    pub gate: Gate,
}

impl Reading {
    /// The one line `/gmut info` prints, one field per `key=value` so a
    /// harness can read it back.
    pub fn line(&self) -> String {
        format!(
            "gmut: {}, mode={} ({:.1}x), rules fly_always={}, stop_immediately={}, \
fix_fov={}, mute_modulating_sound={}, modulation={}",
            self.placement.text(),
            self.mode.name(),
            self.mode.multiplier(),
            on_off(self.rules.fly_always),
            on_off(self.rules.stop_immediately),
            on_off(self.rules.fix_fov),
            on_off(self.rules.mute_modulating_sound),
            self.gate.name()
        )
    }
}

/// `on` / `off`, as the info line spells a rule flag.
const fn on_off(flag: bool) -> &'static str {
    if flag { "on" } else { "off" }
}

/// Upstream's `displayModeChange` line for one step: its label and multiplier.
pub fn mode_line(mode: VerticalSpeed) -> String {
    format!(
        "Vertical Speed Modifier: {} ({:.1}x)",
        mode.name(),
        mode.multiplier()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_steps_are_off_low_medium_high_ultra() {
        let names: Vec<&str> = VerticalSpeed::ALL.iter().map(|s| s.name()).collect();
        assert_eq!(names, ["off", "low", "medium", "high", "ultra"]);
    }

    #[test]
    fn each_step_knows_its_own_index() {
        for (index, step) in VerticalSpeed::ALL.iter().enumerate() {
            assert_eq!(step.index(), index, "step {step:?}");
        }
    }

    #[test]
    fn the_multipliers_are_upstreams() {
        let multipliers: Vec<f32> = VerticalSpeed::ALL.iter().map(|s| s.multiplier()).collect();
        assert_eq!(multipliers, [1.0, 1.5, 2.0, 2.5, 3.0]);
    }

    #[test]
    fn every_step_reads_back_from_its_name() {
        for step in VerticalSpeed::ALL {
            assert_eq!(VerticalSpeed::from_name(step.name()), Some(step));
        }
    }

    #[test]
    fn an_unknown_or_differently_cased_name_is_no_step() {
        assert_eq!(VerticalSpeed::from_name("bogus"), None);
        assert_eq!(VerticalSpeed::from_name("OFF"), None);
        assert_eq!(VerticalSpeed::from_name(""), None);
    }

    #[test]
    fn next_wraps_from_ultra_to_off() {
        assert_eq!(VerticalSpeed::Medium.next(), VerticalSpeed::High);
        assert_eq!(VerticalSpeed::Ultra.next(), VerticalSpeed::Off);
    }

    #[test]
    fn previous_wraps_from_off_to_ultra() {
        assert_eq!(VerticalSpeed::Medium.previous(), VerticalSpeed::Low);
        assert_eq!(VerticalSpeed::Off.previous(), VerticalSpeed::Ultra);
    }

    #[test]
    fn a_shift_of_one_is_the_next_step() {
        for step in VerticalSpeed::ALL {
            assert_eq!(step.adjust(1), step.next(), "step {step:?}");
            assert_eq!(step.adjust(-1), step.previous(), "step {step:?}");
        }
    }

    #[test]
    fn a_larger_shift_wraps_around_the_list() {
        assert_eq!(VerticalSpeed::High.adjust(2), VerticalSpeed::Off);
        assert_eq!(VerticalSpeed::Off.adjust(-3), VerticalSpeed::Medium);
        assert_eq!(VerticalSpeed::Off.adjust(10), VerticalSpeed::Off);
        assert_eq!(VerticalSpeed::Off.adjust(0), VerticalSpeed::Off);
    }

    #[test]
    fn sneaking_turns_the_mode_switch_backwards() {
        assert_eq!(shift_for(false), 1);
        assert_eq!(shift_for(true), -1);
    }

    #[test]
    fn the_rule_defaults_are_upstreams_config_defaults() {
        let rules = Rules::new();
        assert_eq!(rules, Rules::DEFAULTS);
        assert!(!rules.fly_always);
        assert!(rules.stop_immediately);
        assert!(!rules.fix_fov);
        assert!(!rules.mute_modulating_sound);
        assert_eq!(Rules::default(), rules);
    }

    #[test]
    fn a_ready_suit_passes_the_gate_and_a_bare_one_does_not() {
        assert_eq!(gate(true), Gate::Ready);
        assert_eq!(gate(false), Gate::NeedsGravitationalModulatingUnit);
        assert_eq!(Gate::Ready.name(), "ready");
        assert_eq!(
            Gate::NeedsGravitationalModulatingUnit.name(),
            "needs_gravitational_modulating_unit"
        );
    }

    #[test]
    fn each_placement_has_its_own_word() {
        assert_eq!(Placement::Held.name(), "held");
        assert_eq!(Placement::OffHand.name(), "off_hand");
        assert_eq!(Placement::Carried(2).name(), "carried");
        assert_eq!(Placement::Absent.name(), "absent");
    }

    #[test]
    fn the_info_line_names_every_field() {
        let reading = Reading {
            placement: Placement::Held,
            mode: VerticalSpeed::Medium,
            rules: Rules::new(),
            gate: Gate::NeedsGravitationalModulatingUnit,
        };
        assert_eq!(
            reading.line(),
            "gmut: held, mode=medium (2.0x), rules fly_always=off, \
stop_immediately=on, fix_fov=off, mute_modulating_sound=off, \
modulation=needs_gravitational_modulating_unit"
        );
    }

    #[test]
    fn the_info_line_shows_a_carried_count_and_an_absent_unit() {
        let rules = Rules::new();
        let carried = Reading {
            placement: Placement::Carried(3),
            mode: VerticalSpeed::Ultra,
            rules,
            gate: Gate::NeedsGravitationalModulatingUnit,
        };
        assert!(
            carried
                .line()
                .starts_with("gmut: carried(3), mode=ultra (3.0x)")
        );
        let absent = Reading {
            placement: Placement::Absent,
            mode: VerticalSpeed::Off,
            rules,
            gate: Gate::NeedsGravitationalModulatingUnit,
        };
        assert!(absent.line().starts_with("gmut: absent, mode=off (1.0x)"));
    }

    #[test]
    fn a_mode_change_reads_as_upstreams_label_and_multiplier() {
        assert_eq!(
            mode_line(VerticalSpeed::Low),
            "Vertical Speed Modifier: low (1.5x)"
        );
        assert_eq!(
            mode_line(VerticalSpeed::Ultra),
            "Vertical Speed Modifier: ultra (3.0x)"
        );
    }
}
