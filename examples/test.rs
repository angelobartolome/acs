use acs::{
    CoincidentConstraint, Constraint, EqualXConstraint, EqualYConstraint, HorizontalConstraint,
    ParallelConstraint, Point, VerticalConstraint, solve_constraints,
};

fn main() {
    let mut points = vec![
        Point {
            id: 0,
            x: 2.0,
            y: 1.0,
            fixed: false,
        },
        Point {
            id: 1,
            x: 2.0,
            y: 15.0,
            fixed: false,
        },
        // Point {
        //     id: 2,
        //     x: 5.0,
        //     y: 1.0,
        //     fixed: false,
        // },
        // Point {
        //     id: 3,
        //     x: 6.0,
        //     y: 15.0,
        //     fixed: false,
        // },
    ];

    let constraints: Vec<Box<dyn Constraint>> = vec![
        // Box::new(VerticalConstraint { p1: 0, p2: 1 }),
        // Box::new(HorizontalConstraint { p1: 1, p2: 2 }),
        // Box::new(ParallelConstraint::new(0, 1, 2, 3)),
        // Box::new(EqualYConstraint::new(0, 3.0)),
        Box::new(CoincidentConstraint::new(0, 1)),
    ];

    solve_constraints(&mut points, &constraints, 20);

    println!("{:?}", points);
}
