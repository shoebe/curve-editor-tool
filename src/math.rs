// below was all AI generated

pub trait EguiPlotPointExt {
    fn distance(&self, other: Self) -> f64;
}

impl EguiPlotPointExt for egui_plot::PlotPoint {
    fn distance(&self, other: Self) -> f64 {
        let dist_sq = (self.x - other.x).powi(2) + (self.y - other.y).powi(2);
        dist_sq.sqrt()
    }
}

/// Returns (distance, closest_point) from a point to a line segment defined by two endpoints.
/// The closest point is clamped to the line segment bounds [0, 1].
pub fn closest_point_on_line(
    point: egui::Pos2,
    line_start: egui::Pos2,
    line_end: egui::Pos2,
) -> (f32, egui::Pos2) {
    // Vector from line_start to line_end
    let ab = line_end - line_start;

    // Vector from line_start to point
    let ap = point - line_start;

    // Squared length of line segment
    let ab_squared = ab.dot(ab);

    // Handle degenerate case where line segment has zero length
    if ab_squared < 1e-10 {
        let distance = ap.length();
        return (distance, line_start);
    }

    // Calculate parameter t (projection of point onto line)
    let t = ap.dot(ab) / ab_squared;

    // Clamp t to [0, 1] to stay within line segment
    let t_clamped = t.clamp(0.0, 1.0);

    // Calculate closest point on line segment
    let closest = line_start + t_clamped * ab;

    // Calculate distance from point to closest point
    let distance = (point - closest).length();

    (distance, closest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_on_line() {
        // Point is directly on the line segment
        let point = egui::Pos2::new(0.5, 0.5);
        let line_start = egui::Pos2::new(0.0, 0.0);
        let line_end = egui::Pos2::new(1.0, 1.0);

        let (distance, closest) = closest_point_on_line(point, line_start, line_end);

        assert!((distance - 0.0).abs() < 1e-10, "Distance should be 0");
        assert!((closest.x - 0.5).abs() < 1e-10, "Closest x should be 0.5");
        assert!((closest.y - 0.5).abs() < 1e-10, "Closest y should be 0.5");
    }

    #[test]
    fn test_point_perpendicular_to_middle() {
        // Point is perpendicular to the middle of the line
        let point = egui::Pos2::new(0.5, 0.0);
        let line_start = egui::Pos2::new(0.0, 0.0);
        let line_end = egui::Pos2::new(1.0, 0.0);

        let (distance, closest) = closest_point_on_line(point, line_start, line_end);

        assert!((distance - 0.0).abs() < 1e-10, "Distance should be 0");
        assert!((closest.x - 0.5).abs() < 1e-10, "Closest x should be 0.5");
        assert!((closest.y - 0.0).abs() < 1e-10, "Closest y should be 0.0");
    }

    #[test]
    fn test_point_above_line() {
        // Point is above a horizontal line
        let point = egui::Pos2::new(0.5, 1.0);
        let line_start = egui::Pos2::new(0.0, 0.0);
        let line_end = egui::Pos2::new(1.0, 0.0);

        let (distance, closest) = closest_point_on_line(point, line_start, line_end);

        assert!((distance - 1.0).abs() < 1e-10, "Distance should be 1.0");
        assert!((closest.x - 0.5).abs() < 1e-10, "Closest x should be 0.5");
        assert!((closest.y - 0.0).abs() < 1e-10, "Closest y should be 0.0");
    }

    #[test]
    fn test_point_clamped_to_start() {
        // Point projects before line start, should clamp to start
        let point = egui::Pos2::new(-0.5, 0.0);
        let line_start = egui::Pos2::new(0.0, 0.0);
        let line_end = egui::Pos2::new(1.0, 0.0);

        let (distance, closest) = closest_point_on_line(point, line_start, line_end);

        assert!((distance - 0.5).abs() < 1e-10, "Distance should be 0.5");
        assert!(
            (closest.x - 0.0).abs() < 1e-10,
            "Closest x should be 0.0 (clamped to start)"
        );
        assert!((closest.y - 0.0).abs() < 1e-10, "Closest y should be 0.0");
    }

    #[test]
    fn test_point_clamped_to_end() {
        // Point projects after line end, should clamp to end
        let point = egui::Pos2::new(1.5, 0.0);
        let line_start = egui::Pos2::new(0.0, 0.0);
        let line_end = egui::Pos2::new(1.0, 0.0);

        let (distance, closest) = closest_point_on_line(point, line_start, line_end);

        assert!((distance - 0.5).abs() < 1e-10, "Distance should be 0.5");
        assert!(
            (closest.x - 1.0).abs() < 1e-10,
            "Closest x should be 1.0 (clamped to end)"
        );
        assert!((closest.y - 0.0).abs() < 1e-10, "Closest y should be 0.0");
    }

    #[test]
    fn test_degenerate_line() {
        // Line segment with zero length (both endpoints are the same)
        let point = egui::Pos2::new(1.0, 1.0);
        let line_start = egui::Pos2::new(0.0, 0.0);
        let line_end = egui::Pos2::new(0.0, 0.0);

        let (distance, closest) = closest_point_on_line(point, line_start, line_end);

        let expected_distance = (2.0_f32).sqrt();
        assert!(
            (distance - expected_distance).abs() < 1e-10,
            "Distance should be sqrt(2)"
        );
        assert!((closest.x - 0.0).abs() < 1e-10, "Closest x should be 0.0");
        assert!((closest.y - 0.0).abs() < 1e-10, "Closest y should be 0.0");
    }

    #[test]
    fn test_diagonal_line() {
        // Point near a diagonal line
        let point = egui::Pos2::new(0.0, 1.0);
        let line_start = egui::Pos2::new(0.0, 0.0);
        let line_end = egui::Pos2::new(1.0, 1.0);

        let (distance, closest) = closest_point_on_line(point, line_start, line_end);

        // Closest point should be at (0.5, 0.5) with distance sqrt(0.5)
        let expected_distance = (0.5_f32).sqrt();
        assert!(
            (distance - expected_distance).abs() < 1e-10,
            "Distance should be sqrt(0.5)"
        );
        assert!((closest.x - 0.5).abs() < 1e-10, "Closest x should be 0.5");
        assert!((closest.y - 0.5).abs() < 1e-10, "Closest y should be 0.5");
    }
}
