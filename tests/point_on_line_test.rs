use acs::{ConstraintSolver, ConstraintType, Line, Point, SolverResult};

#[test]
fn test_point_on_line_constraint() {
    let mut solver = ConstraintSolver::new();

    // Create two points that define a line from (0,0) to (2,2)
    let p1 = Point::new(1, 0.0, 0.0, true); // Fixed start point
    let p2 = Point::new(2, 2.0, 2.0, true); // Fixed end point
    solver.add_point(p1);
    solver.add_point(p2);

    // Create the line
    let line1 = Line::new(1, p1.id, p2.id);
    solver.add_line(line1);

    // Create a point that should be constrained to lie on the line
    // Start it off the line at (1.0, 3.0)
    let p3 = Point::new(3, 1.0, 3.0, false);
    solver.add_point(p3);

    // Add the point-on-line constraint
    solver
        .add_constraint(ConstraintType::PointOnLine(p3.id, line1.id))
        .unwrap();

    let result = solver.solve().unwrap();

    match result {
        SolverResult::Converged { final_error, .. } => {
            assert!(final_error < 1e-6);
        }
        final_result => panic!("Solver should have converged, got: {:?}", final_result),
    }

    let constrained_point = solver.get_point(p3.id).unwrap();
    solver.print_state();

    // The line goes from (0,0) to (2,2), so it has the equation y = x
    // The constrained point should satisfy this equation
    assert!(
        (constrained_point.y - constrained_point.x).abs() < 1e-4,
        "Point should lie on the line y = x, but got ({}, {})",
        constrained_point.x,
        constrained_point.y
    );

    // Verify the point is actually on the line by calculating distance
    let line_start = solver.get_point(p1.id).unwrap();
    let line_end = solver.get_point(p2.id).unwrap();

    let dx = line_end.x - line_start.x;
    let dy = line_end.y - line_start.y;
    let line_length = (dx * dx + dy * dy).sqrt();

    // Distance from point to line using the same formula as in the constraint
    let a = dy;
    let b = -dx;
    let c = dx * line_start.y - dy * line_start.x;
    let distance = (a * constrained_point.x + b * constrained_point.y + c).abs() / line_length;

    assert!(
        distance < 1e-6,
        "Distance from point to line should be near zero, but got {}",
        distance
    );
}

#[test]
fn test_point_on_horizontal_line_constraint() {
    let mut solver = ConstraintSolver::new();

    // Create a horizontal line from (0,1) to (3,1)
    let p1 = Point::new(1, 0.0, 1.0, true); // Fixed start point
    let p2 = Point::new(2, 3.0, 1.0, true); // Fixed end point
    solver.add_point(p1);
    solver.add_point(p2);

    let line1 = Line::new(1, p1.id, p2.id);
    solver.add_line(line1);

    // Create a point off the line
    let p3 = Point::new(3, 1.5, 2.5, false);
    solver.add_point(p3);

    solver
        .add_constraint(ConstraintType::PointOnLine(p3.id, line1.id))
        .unwrap();

    let result = solver.solve().unwrap();

    match result {
        SolverResult::Converged { final_error, .. } => {
            assert!(final_error < 1e-6);
        }
        final_result => panic!("Solver should have converged, got: {:?}", final_result),
    }

    let constrained_point = solver.get_point(p3.id).unwrap();

    // The point should have y-coordinate = 1.0 (on the horizontal line)
    assert!(
        (constrained_point.y - 1.0).abs() < 1e-6,
        "Point should lie on the horizontal line y = 1, but got y = {}",
        constrained_point.y
    );
}

#[test]
fn test_point_on_vertical_line_constraint() {
    let mut solver = ConstraintSolver::new();

    // Create a vertical line from (2,0) to (2,4)
    let p1 = Point::new(1, 2.0, 0.0, true); // Fixed start point
    let p2 = Point::new(2, 2.0, 4.0, true); // Fixed end point
    solver.add_point(p1);
    solver.add_point(p2);

    let line1 = Line::new(1, p1.id, p2.id);
    solver.add_line(line1);

    // Create a point off the line
    let p3 = Point::new(3, 5.0, 2.0, false);
    solver.add_point(p3);

    solver
        .add_constraint(ConstraintType::PointOnLine(p3.id, line1.id))
        .unwrap();

    let result = solver.solve().unwrap();

    match result {
        SolverResult::Converged { final_error, .. } => {
            assert!(final_error < 1e-6);
        }
        final_result => panic!("Solver should have converged, got: {:?}", final_result),
    }

    let constrained_point = solver.get_point(p3.id).unwrap();

    // The point should have x-coordinate = 2.0 (on the vertical line)
    assert!(
        (constrained_point.x - 2.0).abs() < 1e-6,
        "Point should lie on the vertical line x = 2, but got x = {}",
        constrained_point.x
    );
}
