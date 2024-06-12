use std::f64::consts::PI;

use kurbo::{
    BezPath, ParamCurve, ParamCurveArclen, ParamCurveCurvature, ParamCurveDeriv, PathSeg, Vec2,
};

trait SegCurvature {
    fn curvature(&self, t: f64) -> f64;
    fn tangent(&self, t: f64) -> Vec2;
}

fn angle_between(v1: Vec2, v2: Vec2) -> f64 {
    let dot = v1.dot(v2);
    (dot / (v1.length() * v2.length())).acos()
}

impl SegCurvature for PathSeg {
    fn curvature(&self, t: f64) -> f64 {
        match self {
            PathSeg::Line(line) => line.curvature(t),
            PathSeg::Quad(quad) => quad.curvature(t),
            PathSeg::Cubic(cubic) => cubic.curvature(t),
        }
    }
    fn tangent(&self, t: f64) -> Vec2 {
        match self {
            PathSeg::Line(line) => line.deriv().eval(t),
            PathSeg::Quad(quad) => quad.deriv().eval(t),
            PathSeg::Cubic(cubic) => cubic.deriv().eval(t),
        }
        .to_vec2()
    }
}

pub(crate) fn wonkiness(path: &BezPath) -> f32 {
    let mut path_wonk = 0.0;
    log::debug!("\nConsidering path {:?}", path);
    let is_closed = if let Some(last_el) = path.elements().last() {
        matches!(last_el, kurbo::PathEl::ClosePath)
    } else {
        false
    };
    let segs: Vec<PathSeg> = path
        .segments()
        .filter(|seg| seg.arclen(0.1) > 0.0)
        .collect();

    let pairs: Box<dyn Iterator<Item = (&PathSeg, &PathSeg)>> = if is_closed {
        Box::new(segs.iter().zip(segs.iter().cycle().skip(1)))
    } else {
        Box::new(segs.iter().zip(segs.iter().skip(1)))
    };

    for (seg, next_seg) in pairs {
        let in_curvature = seg.curvature(0.95);
        let out_curvature = next_seg.curvature(0.05);
        // log::debug!(
        //     "in_curvature: {}, out_curvature: {}",
        //     in_curvature,
        //     out_curvature,
        // );
        let curvaturediff = (in_curvature - out_curvature).abs();
        let in_tangent = seg.tangent(0.95);
        let out_tangent = next_seg.tangent(0.05);
        // log::debug!("inc_tangent: {}, out_tangent: {}", in_tangent, out_tangent);
        // log::debug!(
        //     "inc_angle: {}°, out_angle: {}°",
        //     (in_tangent.atan2().to_degrees()),
        //     out_tangent.atan2().to_degrees()
        // );
        let angle_between = angle_between(out_tangent, in_tangent);
        let anglediff = (angle_between - ((angle_between) / (PI / 2.0)).round() * (PI / 2.0)).abs();
        let total_len = seg.arclen(0.1) + next_seg.arclen(0.1);
        let contribution = (1.0 + curvaturediff) * (anglediff / total_len);
        // let contribution = anglediff;
        if contribution != 0.0 {
            log::debug!("in_seg: {:?}, out_seg: {:?}", seg, next_seg);
            log::debug!("Angle between: {}°", angle_between.to_degrees());
            log::info!("curvaturediff: {}, anglediff: {}", curvaturediff, anglediff);
            log::debug!("Contribution: {}", contribution);
        }
        path_wonk += contribution;
    }
    log::debug!("Total wonkiness: {}\n\n", path_wonk);

    path_wonk as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use test_log::test;

    #[test]
    fn test_wonk_colinear() {
        // Colinear lines are not wonky
        let path = BezPath::from_svg("M 0 0 L 1 1 L 2 2").unwrap();
        assert_relative_eq!(wonkiness(&path), 0.0);
    }
    #[test]
    fn test_wonk_right_angle() {
        // Right angles are not wonky
        let path = BezPath::from_svg("M 0 0 L 1 0 L 1 1").unwrap();
        assert_relative_eq!(wonkiness(&path), 0.0);
    }
    #[test]
    fn test_wonk_sharp1() {
        // Sharper angles are wonkier than wider angles
        let path1 = BezPath::from_svg("M 0 0 L 1 1 L 1.5 0").unwrap();
        let path2 = BezPath::from_svg("M 0 0 L 1 1 L 2.5 0").unwrap();
        assert!(wonkiness(&path1) > wonkiness(&path2));
    }
    // #[test]
    // fn test_wonk_sharp2() {
    //     // Sharper angles are wonkier than wider angles
    //     let path1 = BezPath::from_svg("M 0 1 L 1 0 L 0 2").unwrap();
    //     let path2 = BezPath::from_svg("M 0 0 L 1 0 L 0 2").unwrap();
    //     assert!(wonkiness(&path1) > wonkiness(&path2));
    // }
    #[test]
    fn test_wonk_short() {
        // Shorter segments are wonkier than longer segments
        let path1 = BezPath::from_svg("M 0 1 L 1 0 L 0 2").unwrap();
        let path2 = BezPath::from_svg("M 0 2 L 2 0 L 0 4").unwrap();
        assert!(wonkiness(&path1) > wonkiness(&path2));
    }
    #[test]
    fn test_wonk_g2() {
        // G2 continuous segments are not (very) wonky
        let path = BezPath::from_svg("M 0 0 C 0 0.3 0.3 1 1 1 C 1.7 1 2 0.3 2 0").unwrap();
        assert!(wonkiness(&path) < 0.1);
    }
    #[test]
    fn test_wonk_not_g2() {
        // But non-G2 continuous segments are much wonkier
        let path1 = BezPath::from_svg("M 0 0 C 0 0.3 0.3 1 1 1 C 1.7 1 2 0.3 2 0").unwrap();
        let path2 = BezPath::from_svg("M 0 0 C 0 0.3 0.6 1 1 1 C 2 1 2 0.3 2 0").unwrap();
        assert!(wonkiness(&path2) > 2.0 * wonkiness(&path1));
    }
    #[test]
    fn test_add_point() {
        // Adding a continuous point doesn't make the path much wonkier
        let path1 = BezPath::from_svg("M 0 0 C 0 0.3 0.3 1 1 1 C 1.7 1 2 0.3 2 0").unwrap();
        let path2 = BezPath::from_svg(
            "M 0 0 C 0 0.18 0.1 0.49 0.33 0.72 C 0.49 0.88 0.71 1 1 1 C 1.7 1 2 0.3 2 0",
        )
        .unwrap();
        let wonk1 = wonkiness(&path1);
        let wonk2 = wonkiness(&path2);

        assert!(wonk2 >= wonk1);
        assert!(wonk2 < 1.5 * wonk1);
    }

    #[test]
    fn test_add_good_point() {
        // Adding a good-ish point doesn't make the path much wonkier
        let path1 = BezPath::from_svg("M 0 0 L 1 2 L 2 0").unwrap();
        let path2 = BezPath::from_svg("M 0 0 L 0.45 1 L 1 2 L 2 0").unwrap();
        let wonk1 = wonkiness(&path1);
        let wonk2 = wonkiness(&path2);
        assert!(wonk2 > 0.9 * wonk1);
        assert!(wonk2 < 1.5 * wonk1);
    }
    #[test]
    fn test_add_bad_point() {
        // But adding wobbly bits to a path makes it a lot wonkier
        let path1 = BezPath::from_svg("M 0 0 L 0.5 0.5 L 1 1 L 1.5 0.5 L 2 0").unwrap();
        let path2 = BezPath::from_svg("M 0 0 L 0.5 0.5 L  0.8 0.9 L 1 1 L 1.5 0.5 L 1.7 0.2 L 2 0")
            .unwrap();
        let wonk1 = wonkiness(&path1);
        let wonk2 = wonkiness(&path2);
        assert!(wonk2 > 4.0 * wonk1);
    }

    #[test]
    fn test_quad_to_line() {
        // Turning quadratics into lines shouldn't make it wonkier
        let path1 = BezPath::from_svg(
            "M1 20Q0 51 0 51Q0 51 19 51Q38 51 38 51Q38 51 38 51L38 0Q0 0 0 0L1 20Z",
        )
        .unwrap();
        let path2 = BezPath::from_svg("M0 51L1 20L0 0L38 0L38 51L0 51Z").unwrap();
        let wonk1 = wonkiness(&path1);
        let wonk2 = wonkiness(&path2);
        assert!(wonk2 < 1.1 * wonk1);
    }
}
