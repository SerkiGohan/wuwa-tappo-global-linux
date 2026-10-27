use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HitRegion {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_id: Option<u64>,
}

impl HitRegion {
    pub fn is_valid(&self) -> bool {
        [self.left, self.top, self.width, self.height]
            .iter()
            .all(|n| n.is_finite())
            && self.width >= 0.0
            && self.height >= 0.0
    }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.left && y >= self.top && x < self.left + self.width && y < self.top + self.height
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize)]
pub struct Insets {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

/// Convert a physical screen point to the CSS pixel space used by WebKit.
pub fn physical_to_css(
    x: f64,
    y: f64,
    origin_x: f64,
    origin_y: f64,
    webview_scale: f64,
) -> (f64, f64) {
    let scale = webview_scale.max(f64::EPSILON);
    ((x - origin_x) / scale, (y - origin_y) / scale)
}

/// Convert CSS pixels back to the payload space expected by the official
/// frontend, which divides by Tauri's scale factor and subtracts the window
/// origin. When both scales match this is the original physical position.
pub fn css_to_payload(
    css_x: f64,
    css_y: f64,
    origin_x: f64,
    origin_y: f64,
    tauri_scale: f64,
) -> (f64, f64) {
    (
        origin_x + css_x * tauri_scale,
        origin_y + css_y * tauri_scale,
    )
}

pub fn work_area_insets(
    screen: (f64, f64, f64, f64),
    work: (f64, f64, f64, f64),
    scale: f64,
) -> Insets {
    Insets {
        left: ((work.0 - screen.0) / scale).max(0.0),
        top: ((work.1 - screen.1) / scale).max(0.0),
        right: ((screen.0 + screen.2 - work.0 - work.2) / scale).max(0.0),
        bottom: ((screen.1 + screen.3 - work.1 - work.3) / scale).max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_regions_use_css_pixels_and_half_open_edges() {
        let region = HitRegion {
            left: -10.0,
            top: 20.0,
            width: 40.0,
            height: 50.0,
            member_id: Some(7),
        };
        assert!(region.contains(-10.0, 20.0));
        assert!(!region.contains(30.0, 50.0));
        assert!(!region.contains(0.0, 70.0));
        assert_eq!(serde_json::to_value(region).unwrap()["memberId"], 7);
    }

    #[test]
    fn scaled_dock_insets_are_logical() {
        let i = work_area_insets((0., 0., 3024., 1964.), (0., 74., 3024., 1760.), 2.);
        assert_eq!(i.top, 37.);
        assert_eq!(i.bottom, 65.);
        assert_eq!(i.left, 0.);
    }

    #[test]
    fn secondary_monitor_negative_origin_does_not_change_insets() {
        let i = work_area_insets(
            (-1920., 200., 1920., 1080.),
            (-1870., 225., 1870., 1055.),
            1.,
        );
        assert_eq!(i.left, 50.);
        assert_eq!(i.top, 25.);
        assert_eq!(i.bottom, 0.);
    }

    #[test]
    fn fractional_webkit_scale_is_used_for_css_hit_regions() {
        let (x, y) = physical_to_css(2509., 1285., 0., 75., 1.5);
        assert!((x - 1_672.666_666).abs() < 1e-6);
        assert!((y - 806.666_666).abs() < 1e-6);
        let (px, py) = css_to_payload(x, y, 0., 75., 1.0);
        assert!((px - 1_672.666_666).abs() < 1e-6);
        assert!((py - 881.666_666).abs() < 1e-6);
    }

    #[test]
    fn matching_scales_keep_the_original_physical_payload() {
        let (x, y) = physical_to_css(1280., 720., 0., 0., 2.);
        assert_eq!((x, y), (640., 360.));
        assert_eq!(css_to_payload(x, y, 0., 0., 2.), (1280., 720.));
    }

    #[test]
    fn rejects_invalid_region_geometry() {
        let r = HitRegion {
            left: 0.,
            top: 0.,
            width: -1.,
            height: 5.,
            member_id: None,
        };
        assert!(!r.is_valid());
    }
}
