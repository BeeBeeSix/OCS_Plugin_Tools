//! VPSI - viewport scale information, ported from VPSI.LSP.
//!
//! Select a viewport on a layout and run VPSI: it prints `VPSCALE = 1:N` with
//! N = 1000 / (viewport height / view height), as the lisp does (paper in mm, model in m).

use ocs_plugin_api::host::{EntityType, Handle, HostApi, HostSettingValue};

/// The text after `1:` for a viewport `paper_height` high showing `view_height` model units.
pub fn scale_text(paper_height: f64, view_height: f64) -> String {
    let n = 1000.0 / (paper_height / view_height);
    let s = format!("{:.2}", n);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

pub fn run(host: &mut dyn HostApi) {
    let layout = match host.system_variable("CTAB") {
        Some(HostSettingValue::Text(t)) => t,
        _ => String::new(),
    };
    if layout.is_empty() || layout.eq_ignore_ascii_case("Model") {
        host.push_error("VPSI: go to a layout, select a viewport, then run VPSI.");
        return;
    }
    let selected: Vec<Handle> = host.selection();
    let mut found: Vec<(i16, String)> = Vec::new();
    for h in selected {
        let Some(EntityType::Viewport(v)) = host.document().get_entity(h) else { continue };
        if v.id <= 1 {
            continue; // the layout's own sheet viewport
        }
        if v.status.perspective {
            host.push_error("VPSI: viewports with perspective view on are not allowed.");
            continue;
        }
        if v.height.abs() < 1e-12 || v.view_height.abs() < 1e-12 {
            continue;
        }
        found.push((v.id, scale_text(v.height.abs(), v.view_height.abs())));
    }
    if found.is_empty() {
        host.push_output("VPSI: select a viewport first, then run VPSI.");
        return;
    }
    let several = found.len() > 1;
    for (id, text) in found {
        if several {
            host.push_output(&format!("Viewport {id}: VPSCALE = 1:{text}"));
        } else {
            host.push_output(&format!("VPSCALE = 1:{text}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::scale_text;

    #[test]
    fn scales_match_the_lisp_formula() {
        assert_eq!(scale_text(100.0, 50.0), "500");
        assert_eq!(scale_text(100.0, 100.0), "1000");
        assert_eq!(scale_text(200.0, 50.0), "250");
        assert_eq!(scale_text(100.0, 33.333), "333.33");
        assert_eq!(scale_text(100.0, 5.0), "50");
    }
}
