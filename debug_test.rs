use acs::{Constraint, Point, VerticalConstraint, solve_constraints};

fn main() {
    println!("Starting debug test...");

    let mut points = vec![
        Point {
            id: 0,
            x: 2.0,
            y: 1.0,
            fixed: false,
        },
        Point {
            id: 1,
            x: 5.0,
            y: 4.0,
            fixed: false,
        },
    ];

    println!("Initial points: {:?}", points);

    let constraints: Vec<Box<dyn Constraint>> = vec![Box::new(VerticalConstraint { p1: 0, p2: 1 })];

    println!("Created vertical constraint: p1=0, p2=1");

    // Test the constraint manually
    println!("Testing constraint manually:");
    let residual = constraints[0].residual(&points);
    println!("Residual: {:?}", residual);

    let jacobian = constraints[0].jacobian(&points);
    println!("Jacobian: {}", jacobian);

    solve_constraints(&mut points, &constraints, 5);

    println!("Final points: {:?}", points);
}
