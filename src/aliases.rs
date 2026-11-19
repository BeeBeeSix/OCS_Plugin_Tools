//! Command aliases from TT-START.LSP, run on the host.
//!
//! Most just start a host command (the host then asks for the rest). The layer-wide ones
//! (`F*`, `O*`, `T*`, `A*` ...) change the layer table directly instead of typing `-LAYER`.

use ocs_plugin_api::host::{HostApi, HostSettingValue, LayerConfig};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Op {
    Freeze,
    Off,
    On,
    Thaw,
    /// Thaw and turn on.
    ThawOn,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Act {
    /// Host command line(s); `esc_last` clears a prompt the last line leaves open.
    Host(&'static str, bool),
    /// An operation on every layer (the current layer is left alone).
    Layers(Op),
    /// `Layers(ThawOn)` then REGENALL.
    ThawOnRegen,
    /// Thaw layer 0 and make it current, then the host line (the lisp's `layer s 0`).
    Zero(&'static str),
    /// System variables to set, as host `SETVAR name value` lines (the plugin API can only set
    /// CLAYER and SNAPANG directly).
    Vars(&'static [(&'static str, &'static str)]),
}

const AH_VARS: &[(&str, &str)] = &[
    ("ANGBASE", "0"),
    ("ANGDIR", "0"),
    ("LUNITS", "2"),
    ("LUPREC", "3"),
    ("AUNITS", "0"),
    ("AUPREC", "3"),
];
// ANGBASE is in degrees for the host's SETVAR.
const AV_VARS: &[(&str, &str)] = &[
    ("ANGBASE", "90"),
    ("ANGDIR", "1"),
    ("LUNITS", "2"),
    ("LUPREC", "3"),
    ("AUNITS", "1"),
    ("AUPREC", "4"),
];

/// Every alias, with its action.
pub const TABLE: &[(&str, Act)] = &[
    ("Y", Act::Host("U", false)),
    ("UC3", Act::Host("UCS 3POINT", false)),
    ("3F", Act::Host("3DFACE", false)),
    ("AE", Act::Host("AREA OBJECT", false)),
    ("C2", Act::Host("CIRCLE 2P", false)),
    ("DDE", Act::Host("ATTEDIT", false)),
    ("DLE", Act::Host("LEADER", false)),
    ("F*", Act::Layers(Op::Freeze)),
    ("O*", Act::Layers(Op::Off)),
    ("LOF", Act::Layers(Op::Off)),
    ("LON", Act::Layers(Op::On)),
    ("T*", Act::Layers(Op::Thaw)),
    ("A*", Act::ThawOnRegen),
    ("OL", Act::Host("LAYER OFF", false)),
    ("OE", Act::Vars(&[("OSMODE", "1")])),
    ("OI", Act::Vars(&[("OSMODE", "32")])),
    ("ON", Act::Vars(&[("OSMODE", "0")])),
    ("IB", Act::Zero("INSERT")),
    ("CB", Act::Host("COPYBASE 0,0,0", false)),
    ("PG", Act::Host("PASTECLIP 0,0,0", false)),
    ("PGA", Act::Host("PASTECLIP 0,0,0\nAUDIT YES", true)),
    ("CW", Act::Host("UCS WORLD\nCOPYBASE 0,0,0", false)),
    ("PW", Act::Host("UCS WORLD\nPASTECLIP 0,0,0", false)),
    ("T2M", Act::Host("TXT2MTXT", false)),
    ("CLOUD", Act::Host("REVCLOUD", false)),
    ("UE", Act::Host("UCS OBJECT", false)),
    ("UG", Act::Host("UCS GEODETIC", false)),
    ("UP", Act::Host("UCS PROJECT", false)),
    ("UPS", Act::Host("UCS SAVE PROJECT", false)),
    ("UGS", Act::Host("UCS SAVE GEODETIC", false)),
    ("VR", Act::Host("-VIEW R", false)),
    ("VS", Act::Host("-VIEW S", false)),
    ("ZD", Act::Host("ZOOM D", false)),
    ("ZX", Act::Host("ZOOM E", false)),
    ("ZP", Act::Host("ZOOM P", false)),
    ("ZW", Act::Host("ZOOM W", false)),
    ("ZO", Act::Host("ZOOM 0.5X", false)),
    ("ZI", Act::Host("ZOOM 2X", false)),
    ("Z8", Act::Host("ZOOM .8X", false)),
    ("LOGO", Act::Host("LOGFILEON", false)),
    ("LOGF", Act::Host("LOGFILEOFF", false)),
    ("AH", Act::Vars(AH_VARS)),
    ("AV", Act::Vars(AV_VARS)),
    ("PUA", Act::Zero("-PURGE ALL *")),
    ("PULI", Act::Zero("-PURGE LINETYPES *")),
    ("PUB", Act::Zero("-PURGE BLOCKS *")),
];

pub fn lookup(cmd: &str) -> Option<Act> {
    let up = cmd.to_ascii_uppercase();
    TABLE.iter().find(|(n, _)| *n == up).map(|(_, a)| *a)
}

/// The layers an `Op` changes, never the current one.
pub fn layer_targets(layers: &[String], current: &str) -> Vec<String> {
    layers.iter().filter(|l| !l.eq_ignore_ascii_case(current)).cloned().collect()
}

fn op_cfg(name: &str, op: Op) -> LayerConfig {
    let mut c = LayerConfig { name: name.to_string(), ..Default::default() };
    match op {
        Op::Freeze => c.frozen = Some(true),
        Op::Off => c.off = Some(true),
        Op::On => c.off = Some(false),
        Op::Thaw => c.frozen = Some(false),
        Op::ThawOn => {
            c.frozen = Some(false);
            c.off = Some(false);
        }
    }
    c
}

pub fn run(host: &mut dyn HostApi, cmd: &str, act: Act) {
    match act {
        Act::Host(lines, esc_last) => crate::views::run_lines(host, cmd, lines, esc_last),
        Act::Layers(op) => layers(host, cmd, op),
        Act::ThawOnRegen => {
            layers(host, cmd, Op::ThawOn);
            host.execute_command("REGENALL");
        }
        Act::Zero(line) => {
            host.modify_layer(LayerConfig {
                name: "0".to_string(),
                frozen: Some(false),
                ..Default::default()
            });
            if let Err(e) = host.set_system_variable("CLAYER", HostSettingValue::Text("0".to_string())) {
                host.push_error(&format!("{cmd}: could not make layer 0 current: {e}"));
            }
            crate::views::run_lines(host, cmd, line, false);
        }
        Act::Vars(vars) => {
            for (name, v) in vars {
                if !host.execute_command(&format!("SETVAR {name} {v}")) {
                    host.push_error(&format!("{cmd}: the host did not set {name}"));
                }
            }
        }
    }
}

fn layers(host: &mut dyn HostApi, cmd: &str, op: Op) {
    let current = match host.system_variable("CLAYER") {
        Some(HostSettingValue::Text(t)) => t,
        _ => host.document().header.current_layer_name.clone(),
    };
    let names: Vec<String> = host.document().layers.iter().map(|l| l.name.clone()).collect();
    host.begin_undo_group(cmd);
    let mut failed = 0;
    for n in layer_targets(&names, &current) {
        if !host.modify_layer(op_cfg(&n, op)) {
            failed += 1;
        }
    }
    host.end_undo_group();
    if failed > 0 {
        host.push_error(&format!("{cmd}: {failed} layer(s) could not be changed."));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_alias_resolves_once() {
        let mut seen = std::collections::HashSet::new();
        for (n, _) in TABLE {
            assert!(seen.insert(*n), "duplicate {n}");
            assert!(lookup(&n.to_lowercase()).is_some());
        }
        assert!(lookup("NOPE").is_none());
    }

    #[test]
    fn lisp_mappings() {
        assert_eq!(lookup("c2"), Some(Act::Host("CIRCLE 2P", false)));
        assert_eq!(lookup("ZW"), Some(Act::Host("ZOOM W", false)));
        assert_eq!(lookup("OE"), Some(Act::Vars(&[("OSMODE", "1")])));
        assert_eq!(lookup("PUB"), Some(Act::Zero("-PURGE BLOCKS *")));
        assert_eq!(lookup("a*"), Some(Act::ThawOnRegen));
    }

    #[test]
    fn current_layer_is_skipped() {
        let layers = ["0".to_string(), "Site".to_string(), "ROAD".to_string()];
        assert_eq!(layer_targets(&layers, "site"), ["0", "ROAD"]);
    }

    #[test]
    fn layer_op_settings() {
        assert_eq!(op_cfg("A", Op::Freeze).frozen, Some(true));
        assert_eq!(op_cfg("A", Op::Off).off, Some(true));
        let c = op_cfg("A", Op::ThawOn);
        assert_eq!((c.frozen, c.off), (Some(false), Some(false)));
    }

    #[test]
    fn units_presets_match_the_lisp() {
        let Some(Act::Vars(av)) = lookup("AV") else { panic!() };
        assert!(av.contains(&("ANGDIR", "1")) && av.contains(&("AUNITS", "1")) && av.contains(&("ANGBASE", "90")));
        let Some(Act::Vars(ah)) = lookup("AH") else { panic!() };
        assert!(ah.contains(&("ANGDIR", "0")) && ah.contains(&("ANGBASE", "0")));
    }
}
