use crate::wonkiness::wonkiness;
use kurbo::{BezPath, ParamCurveArclen};
use skrifa::outline::OutlinePen;

#[derive(Debug, Default)]
pub(crate) struct Paths {
    path: BezPath,
}

impl OutlinePen for Paths {
    fn move_to(&mut self, x: f32, y: f32) {
        self.path.move_to((x as f64, y as f64));
    }
    fn close(&mut self) {
        self.path.close_path();
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.path.line_to((x as f64, y as f64));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.path
            .quad_to((x1 as f64, y1 as f64), (x2 as f64, y2 as f64));
    }
    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.path.curve_to(
            (cx0 as f64, cy0 as f64),
            (cx1 as f64, cy1 as f64),
            (x as f64, y as f64),
        );
    }
}

fn bezpath_to_skia_path(bez: &BezPath) -> skia_safe::Path {
    let mut path = skia_safe::Path::new();
    for el in bez.elements() {
        match el {
            kurbo::PathEl::MoveTo(p) => {
                path.move_to((p.x as f32, p.y as f32));
            }
            kurbo::PathEl::LineTo(p) => {
                path.line_to((p.x as f32, p.y as f32));
            }
            kurbo::PathEl::QuadTo(p1, p2) => {
                path.quad_to((p1.x as f32, p1.y as f32), (p2.x as f32, p2.y as f32));
            }
            kurbo::PathEl::CurveTo(p1, p2, p3) => {
                path.cubic_to(
                    (p1.x as f32, p1.y as f32),
                    (p2.x as f32, p2.y as f32),
                    (p3.x as f32, p3.y as f32),
                );
            }
            kurbo::PathEl::ClosePath => {
                path.close();
            }
        }
    }
    path
}

fn skia_path_to_bezpath(path: &skia_safe::Path) -> BezPath {
    let mut bez = BezPath::new();
    let points_count = path.count_points();
    let mut points = vec![skia_safe::Point::default(); points_count];
    let _count_returned = path.get_points(&mut points);

    let verb_count = path.count_verbs();
    let mut verbs = vec![0_u8; verb_count];
    let _count_returned_verbs = path.get_verbs(&mut verbs);

    let mut i = 0;
    for verb in verbs {
        match verb {
            0 => {
                bez.move_to((points[i].x as f64, points[i].y as f64));
                i += 1;
            }
            1 => {
                bez.line_to((points[i].x as f64, points[i].y as f64));
                i += 1;
            }
            2 => {
                bez.quad_to(
                    (points[i].x as f64, points[i].y as f64),
                    (points[i + 1].x as f64, points[i + 1].y as f64),
                );
                i += 2;
            }
            4 => {
                bez.curve_to(
                    (points[i].x as f64, points[i].y as f64),
                    (points[i + 1].x as f64, points[i + 1].y as f64),
                    (points[i + 2].x as f64, points[i + 2].y as f64),
                );
                i += 3;
            }
            5 => {
                bez.close_path();
            }
            _ => {}
        }
    }
    bez
}
impl Paths {
    pub fn wonkiness(&self) -> f32 {
        let mut cleaned = vec![];
        // Prep the path. First split into closed paths
        for el in self.path.elements() {
            if matches!(el, kurbo::PathEl::MoveTo(_)) {
                cleaned.push(BezPath::new());
            }
            cleaned.last_mut().unwrap().push(*el);
        }
        cleaned.iter().map(wonkiness).sum::<f32>()
    }

    pub(crate) fn remove_overlaps(&self) -> Paths {
        // println!("Kurbo path: {:?}", self.path.to_svg());
        let skia_path = bezpath_to_skia_path(&self.path);
        // println!("Path: {:?}", skia_path.to_svg());
        // println!(
        //     "Simplified: {:?}",
        //     skia_safe::simplify(&skia_path).unwrap().to_svg()
        // );
        let simple: BezPath = skia_safe::simplify(&skia_path)
            .map(|p| skia_path_to_bezpath(&p))
            .unwrap_or(self.path.clone());
        Paths { path: simple }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_cross_not_wonky() {
        let cross = Paths {
            path: BezPath::from_svg("M 100 100 L 100 200 L 120 200 L 120 100 L 100 100 Z M 75 150 L 75 175 L 150 175L 150 150 L 75 150 Z").unwrap(),
        };
        assert_relative_eq!(cross.wonkiness(), 0.0);
        let removed = cross.remove_overlaps();
        println!("Removed: {:?}", removed.path.to_svg());
        assert_relative_eq!(removed.wonkiness(), 0.0);
    }

    #[test]
    fn test_dagger_not_much_wonkier() {
        let dagger = Paths {
            path: BezPath::from_svg("M 100 100 L 100 200 L 120 200 L 120 50 L 100 100 Z M 75 150 L 75 175 L 150 175L 150 150 L 75 150 Z").unwrap()
        };
        let before = dagger.wonkiness();
        let removed = dagger.remove_overlaps();
        println!("Removed: {:?}", removed.path.to_svg());
        let after = removed.wonkiness();
        assert_relative_eq!(before, after, epsilon = 0.1);
    }

    #[test]
    fn test_upoint_not_wonkier() {
        let upoint = Paths {
            path: BezPath::from_svg(
                "M 1 20 Q 0 51 0 51 Q 0 51 19 51 Q 38 51 38 51 Q 38 51 38 51 L 38 0 Q 0 0 0 0 L 1 20 Z"
            )
            .unwrap(),
        };
        let before = upoint.wonkiness();
        let removed = upoint.remove_overlaps();
        println!("Removed: {:?}", removed.path.to_svg());
        let after = removed.wonkiness();
        let change = (after / before - 1.0) * 100.0;
        println!("Wonkiness before: {:}, after: {:}", before, after);
        println!("Wonkiness change: {:}", change);

        assert!(change < 25.0);
    }
}
