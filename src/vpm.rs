//! VPM - viewport outline to model space, ported from VP-OUTLINE.LSP (Jimmy Bergmark).
//!
//! Select a viewport on a layout and run VPM: a closed polyline on layer `VP` (colour 41) is
//! drawn in model space along the area the viewport shows. A clipped viewport follows its clip
//! boundary. The conversion follows the host's own viewport maths: with the DXF twist angle
//! `t`, paper = centre + R(t) * scale * (model - target) for a plan view.

use ocs_plugin_api::host::document::entities::LwPolyline;
use ocs_plugin_api::host::document::types::{Color, Vector2};
use ocs_plugin_api::host::{
    EntityType, Handle, HostApi, HostSettingValue, LayerConfig, TableOperation,
};

const LAYER: &str = "VP";
const LAYER_COLOUR: i16 = 41;

/// What the outline needs from a viewport entity.
#[derive(Clone, Debug)]
pub struct Vp {
    pub center: [f64; 2],
    pub width: f64,
    pub height: f64,
    pub view_center: [f64; 2],
    pub target: [f64; 3],
    pub dir: [f64; 3],
    pub view_height: f64,
    /// DXF view twist, radians.
    pub twist: f64,
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn unit(v: [f64; 3]) -> [f64; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-12 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

/// Screen right and up of the viewport in WCS: the arbitrary-axis DCS of the view direction,
/// turned by minus the view twist.
fn axes(vp: &Vp) -> ([f64; 3], [f64; 3]) {
    let n = unit(vp.dir);
    let x0 = if n[0].abs() < 1.0 / 64.0 && n[1].abs() < 1.0 / 64.0 {
        unit(cross([0.0, 1.0, 0.0], n))
    } else {
        unit(cross([0.0, 0.0, 1.0], n))
    };
    let y0 = cross(n, x0);
    let (s, c) = (-vp.twist).sin_cos();
    let x = [x0[0] * c + y0[0] * s, x0[1] * c + y0[1] * s, x0[2] * c + y0[2] * s];
    let y = [-x0[0] * s + y0[0] * c, -x0[1] * s + y0[1] * c, -x0[2] * s + y0[2] * c];
    (x, y)
}

/// A paper-space point of the viewport as a model-space XY (Z is dropped, as the lisp does).
pub fn paper_to_model(vp: &Vp, p: [f64; 2]) -> [f64; 2] {
    let (x, y) = axes(vp);
    let s = vp.height / vp.view_height;
    let u = vp.view_center[0] + (p[0] - vp.center[0]) / s;
    let v = vp.view_center[1] + (p[1] - vp.center[1]) / s;
    [vp.target[0] + x[0] * u + y[0] * v, vp.target[1] + x[1] * u + y[1] * v]
}

/// The outline in model space: the clip boundary's points (paper space) when given, else the
/// viewport rectangle.
pub fn outline(vp: &Vp, clip: Option<&[[f64; 2]]>) -> Vec<[f64; 2]> {
    let rect;
    let pts: &[[f64; 2]] = match clip {
        Some(c) => c,
        None => {
            let (hw, hh) = (vp.width.abs() / 2.0, vp.height.abs() / 2.0);
            let (cx, cy) = (vp.center[0], vp.center[1]);
            rect = [[cx - hw, cy - hh], [cx + hw, cy - hh], [cx + hw, cy + hh], [cx - hw, cy + hh]];
            &rect
        }
    };
    pts.iter().map(|&p| paper_to_model(vp, p)).collect()
}

fn area(pts: &[[f64; 2]]) -> f64 {
    let mut a = 0.0;
    for i in 0..pts.len() {
        let (p, q) = (pts[i], pts[(i + 1) % pts.len()]);
        a += p[0] * q[1] - q[0] * p[1];
    }
    a.abs() / 2.0
}

pub fn run(host: &mut dyn HostApi) {
    let layout = match host.system_variable("CTAB") {
        Some(HostSettingValue::Text(t)) => t,
        _ => String::new(),
    };
    if layout.is_empty() || layout.eq_ignore_ascii_case("Model") {
        host.push_error("VPM: go to a layout, select a viewport, then run VPM.");
        return;
    }
    let selected: Vec<Handle> = host.selection();
    let mut outlines: Vec<Vec<[f64; 2]>> = Vec::new();
    for h in selected {
        let doc = host.document();
        let Some(EntityType::Viewport(v)) = doc.get_entity(h) else { continue };
        if v.id <= 1 {
            continue; // the layout's own sheet viewport
        }
        if v.status.perspective {
            host.push_error("VPM: perspective viewports are not supported.");
            continue;
        }
        if v.view_height.abs() < 1e-12 || v.height.abs() < 1e-12 {
            continue;
        }
        let vp = Vp {
            center: [v.center.x, v.center.y],
            width: v.width,
            height: v.height,
            view_center: [v.view_center.x, v.view_center.y],
            target: [v.view_target.x, v.view_target.y, v.view_target.z],
            dir: [v.view_direction.x, v.view_direction.y, v.view_direction.z],
            view_height: v.view_height,
            twist: v.twist_angle,
        };
        let mut clip = None;
        if !v.clip_boundary_handle.is_null() {
            match doc.get_entity(v.clip_boundary_handle) {
                Some(EntityType::LwPolyline(p)) => {
                    clip = Some(p.vertices.iter().map(|q| [q.location.x, q.location.y]).collect::<Vec<_>>());
                    if p.vertices.iter().any(|q| q.bulge != 0.0) {
                        host.push_output("VPM: clip boundary arcs are drawn as straight segments.");
                    }
                }
                _ => host.push_output("VPM: clip boundary is not a polyline; used the viewport rectangle."),
            }
        }
        let pts = outline(&vp, clip.as_deref());
        if pts.len() < 3 || area(&pts) < 1e-9 {
            host.push_error("VPM: the viewport looks along the XY plane, so its outline has no area.");
            continue;
        }
        outlines.push(pts);
    }
    if outlines.is_empty() {
        host.push_output("VPM: select a viewport first, then run VPM.");
        return;
    }

    host.begin_undo_group("VPM");
    host.add_layer(LayerConfig {
        name: LAYER.to_string(),
        color: Some(Color::from_index(LAYER_COLOUR)),
        ..Default::default()
    });
    // add_entity goes to the current space, so draw from the Model tab and come back.
    let back = host
        .table_operation(TableOperation::LayoutSetCurrent { name: "Model".to_string() })
        .is_ok();
    let mut drawn = 0;
    for pts in &outlines {
        let mut pl = LwPolyline::from_points(pts.iter().map(|p| Vector2::new(p[0], p[1])).collect());
        pl.is_closed = true;
        pl.common.layer = LAYER.to_string();
        if !host.add_entity(EntityType::LwPolyline(pl)).is_null() {
            drawn += 1;
        }
    }
    if back {
        let _ = host.table_operation(TableOperation::LayoutSetCurrent { name: layout });
    }
    host.end_undo_group();
    host.push_output(&format!("VPM: drew {drawn} outline(s) on layer {LAYER} in model space."));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 200 x 100 viewport centred at (100,100) on the sheet showing 50 model units of height,
    /// so 2 paper units per model unit, looking at (1000,2000).
    fn plan(twist_deg: f64) -> Vp {
        Vp {
            center: [100.0, 100.0],
            width: 200.0,
            height: 100.0,
            view_center: [0.0, 0.0],
            target: [1000.0, 2000.0, 0.0],
            dir: [0.0, 0.0, 1.0],
            view_height: 50.0,
            twist: twist_deg.to_radians(),
        }
    }

    fn close(a: &[[f64; 2]], b: &[[f64; 2]]) {
        assert_eq!(a.len(), b.len());
        for (p, q) in a.iter().zip(b) {
            assert!((p[0] - q[0]).abs() < 1e-9 && (p[1] - q[1]).abs() < 1e-9, "{a:?} != {b:?}");
        }
    }

    #[test]
    fn untwisted_plan_view_rectangle() {
        close(
            &outline(&plan(0.0), None),
            &[[950.0, 1975.0], [1050.0, 1975.0], [1050.0, 2025.0], [950.0, 2025.0]],
        );
    }

    /// Ground truth from the host's model-to-paper transform: paper = centre + R(90 deg) * 2 *
    /// (model - target), so the sheet's lower-left corner (0,50) shows model (975,2050).
    #[test]
    fn twisted_plan_view_matches_the_host_transform() {
        close(
            &outline(&plan(90.0), None),
            &[[975.0, 2050.0], [975.0, 1950.0], [1025.0, 1950.0], [1025.0, 2050.0]],
        );
    }

    #[test]
    fn view_center_offsets_the_view() {
        let mut vp = plan(0.0);
        vp.view_center = [10.0, -5.0];
        close(&outline(&vp, None)[..1], &[[960.0, 1970.0]]);
    }

    #[test]
    fn clip_boundary_points_are_mapped_instead_of_the_rectangle() {
        let clip = [[100.0, 100.0], [120.0, 100.0], [120.0, 110.0]];
        close(&outline(&plan(0.0), Some(&clip)), &[[1000.0, 2000.0], [1010.0, 2000.0], [1010.0, 2005.0]]);
    }

    #[test]
    fn front_view_outline_has_no_area_in_plan() {
        let mut vp = plan(0.0);
        vp.dir = [0.0, -1.0, 0.0];
        assert!(area(&outline(&vp, None)) < 1e-9);
    }
}
