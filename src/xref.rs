//! XFZ / XTW / XFIX - xref layer tools, ported from XFIX.LSP (Ryan Wunderlich, Cadalyst 2008).
//!
//! The lisp works on the layer named like each xref: XFZ freezes it, XTW thaws it, XFIX moves
//! every xref reference onto it and locks it.

use ocs_plugin_api::host::{Document, EntityType, Handle, HostApi, LayerConfig};
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cmd {
    Freeze,
    Thaw,
    Fix,
}

pub fn command(name: &str) -> Option<Cmd> {
    match name.to_ascii_uppercase().as_str() {
        "XFZ" => Some(Cmd::Freeze),
        "XTW" => Some(Cmd::Thaw),
        "XFIX" => Some(Cmd::Fix),
        _ => None,
    }
}

/// One reference to an xref.
#[derive(Clone, Debug)]
pub struct Ins {
    pub handle: Handle,
    pub block: String,
    pub layer: String,
}

fn same(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

fn xref_inserts(doc: &Document) -> Vec<Ins> {
    let xrefs: HashSet<String> = doc
        .block_records
        .iter()
        .filter(|b| b.flags.is_xref || !b.xref_path.is_empty())
        .map(|b| b.name.to_ascii_uppercase())
        .collect();
    doc.entities()
        .filter_map(|e| match e {
            EntityType::Insert(i) if xrefs.contains(&i.block_name.to_ascii_uppercase()) => Some(Ins {
                handle: i.common.handle,
                block: i.block_name.clone(),
                layer: i.common.layer.clone(),
            }),
            _ => None,
        })
        .collect()
}

/// The layer-table spelling of `name`, if the drawing has that layer.
fn layer_named<'a>(layers: &'a [String], name: &str) -> Option<&'a String> {
    layers.iter().find(|l| same(l, name))
}

/// XFZ / XTW: the existing layers named like a referenced xref (each once), and the names with
/// no such layer.
pub fn layer_targets(inserts: &[Ins], layers: &[String]) -> (Vec<String>, Vec<String>) {
    let mut seen = HashSet::new();
    let (mut found, mut missing) = (Vec::new(), Vec::new());
    for i in inserts {
        if !seen.insert(i.block.to_ascii_uppercase()) {
            continue;
        }
        match layer_named(layers, &i.block) {
            Some(l) => found.push(l.clone()),
            None => missing.push(i.block.clone()),
        }
    }
    (found, missing)
}

#[derive(Debug, Default, PartialEq)]
pub struct FixPlan {
    /// Indices into the inserts whose layer differs from the xref name.
    pub moves: Vec<usize>,
    /// Layers to create first (named like the xref).
    pub create: Vec<String>,
    /// Layers to lock afterwards.
    pub lock: Vec<String>,
}

/// XFIX: every reference not on the layer named like its xref moves there; that layer is
/// created if missing and locked.
pub fn fix_plan(inserts: &[Ins], layers: &[String]) -> FixPlan {
    let mut plan = FixPlan::default();
    let mut seen = HashSet::new();
    for (n, i) in inserts.iter().enumerate() {
        if same(&i.block, &i.layer) {
            continue;
        }
        plan.moves.push(n);
        if seen.insert(i.block.to_ascii_uppercase()) {
            match layer_named(layers, &i.block) {
                Some(l) => plan.lock.push(l.clone()),
                None => {
                    plan.create.push(i.block.clone());
                    plan.lock.push(i.block.clone());
                }
            }
        }
    }
    plan
}

fn layer_cfg(name: &str) -> LayerConfig {
    LayerConfig { name: name.to_string(), ..Default::default() }
}

pub fn run(host: &mut dyn HostApi, cmd: Cmd) {
    let (inserts, layers) = {
        let doc = host.document();
        (xref_inserts(doc), doc.layers.iter().map(|l| l.name.clone()).collect::<Vec<_>>())
    };
    if inserts.is_empty() {
        host.push_output("No xrefs in this drawing.");
        return;
    }
    match cmd {
        Cmd::Freeze | Cmd::Thaw => freeze_thaw(host, cmd == Cmd::Freeze, &inserts, &layers),
        Cmd::Fix => fix(host, &inserts, &layers),
    }
}

fn freeze_thaw(host: &mut dyn HostApi, freeze: bool, inserts: &[Ins], layers: &[String]) {
    let (found, missing) = layer_targets(inserts, layers);
    let verb = if freeze { "XFZ" } else { "XTW" };
    host.begin_undo_group(verb);
    let mut done = 0;
    for name in &found {
        let cfg = LayerConfig { frozen: Some(freeze), ..layer_cfg(name) };
        if host.modify_layer(cfg) {
            done += 1;
        } else {
            host.push_error(&format!("{verb}: could not change layer {name} (is it current?)"));
        }
    }
    host.end_undo_group();
    host.push_output(&format!(
        "{} the layers of {done} out of {} xref(s).",
        if freeze { "Frozen" } else { "Thawed" },
        found.len() + missing.len(),
    ));
    for m in &missing {
        host.push_output(&format!("{verb}: no layer named {m}."));
    }
}

fn fix(host: &mut dyn HostApi, inserts: &[Ins], layers: &[String]) {
    let plan = fix_plan(inserts, layers);
    host.begin_undo_group("XFIX");
    for name in &plan.create {
        host.add_layer(layer_cfg(name));
    }
    let mut fixed = 0;
    for &n in &plan.moves {
        let i = &inserts[n];
        let Some(EntityType::Insert(mut ins)) = host.document().get_entity(i.handle).cloned() else {
            continue;
        };
        ins.common.layer = i.block.clone();
        if host.update_entity(EntityType::Insert(ins)) {
            fixed += 1;
        } else {
            host.push_error(&format!("XFIX: could not move {} (its layer is locked?)", i.block));
        }
    }
    for name in &plan.lock {
        host.modify_layer(LayerConfig { locked: Some(true), ..layer_cfg(name) });
    }
    host.end_undo_group();
    host.push_output(&format!(
        "Fixed the insertion layers of {fixed} out of {} xref reference(s).",
        inserts.len()
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ins(block: &str, layer: &str) -> Ins {
        Ins { handle: Handle::NULL, block: block.into(), layer: layer.into() }
    }

    #[test]
    fn freeze_targets_each_xref_layer_once_and_names_the_missing() {
        let inserts = [ins("SITE", "0"), ins("site", "A"), ins("ROAD", "0"), ins("TREES", "0")];
        let layers = ["0".to_string(), "Site".to_string(), "ROAD".to_string()];
        let (found, missing) = layer_targets(&inserts, &layers);
        assert_eq!(found, ["Site", "ROAD"]);
        assert_eq!(missing, ["TREES"]);
    }

    #[test]
    fn fix_moves_only_misplaced_references_and_locks_their_layers() {
        let inserts = [ins("SITE", "SITE"), ins("SITE", "0"), ins("ROAD", "A"), ins("ROAD", "A")];
        let layers = ["0".to_string(), "A".to_string(), "Site".to_string()];
        let plan = fix_plan(&inserts, &layers);
        assert_eq!(plan.moves, [1, 2, 3]);
        assert_eq!(plan.create, ["ROAD"]);
        assert_eq!(plan.lock, ["Site", "ROAD"]);
    }

    #[test]
    fn fix_leaves_correct_references_alone() {
        let plan = fix_plan(&[ins("SITE", "site")], &["SITE".to_string()]);
        assert_eq!(plan, FixPlan::default());
    }

    #[test]
    fn command_names() {
        assert_eq!(command("xfz"), Some(Cmd::Freeze));
        assert_eq!(command("XTW"), Some(Cmd::Thaw));
        assert_eq!(command("XFIX"), Some(Cmd::Fix));
        assert_eq!(command("XF"), None);
    }
}
