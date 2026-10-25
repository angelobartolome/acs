// The typed sketch API from Swift (UniFFI, module ACSSketch, compiled here with the generated
// ACSSketch.swift), and the C ABI from the same ACS module. Run by tests/swift_test.rs.
import ACS
import Foundation

func check(_ condition: Bool, _ message: @autoclosure () -> String, line: Int = #line) {
    if !condition {
        print("line \(line): \(message())")
        exit(1)
    }
}

func point(_ p: Primitive?) -> (Double, Double) {
    guard case let .point(_, x, y, _, _, _, _) = p else { fatalError("not a point") }
    return (x, y)
}

func near(_ a: (Double, Double), _ b: (Double, Double)) -> Bool {
    abs(a.0 - b.0) < 1e-8 && abs(a.1 - b.1) < 1e-8
}

func byId(_ response: SolveResponse, _ id: String) -> Primitive? {
    response.primitives.first { p in
        switch p {
        case let .point(pid, _, _, _, _, _, _): return pid == id
        default: return false
        }
    }
}

// A square: a Parameter, a line and a two-point horizontal, an x/y pin.
let square: [Primitive] = [
    .point(id: "a", x: 0, y: 0, fixed: true),
    .point(id: "b", x: 9, y: 1),
    .point(id: "c", x: 9, y: 11),
    .point(id: "d", x: 1, y: 10),
    .param(id: "side", value: 10, name: "Side"),
    .line(id: "ab", p1: "a", p2: "b"),
    .line(id: "bc", p1: "b", p2: "c"),
    .line(id: "cd", p1: "c", p2: "d"),
    .line(id: "da", p1: "d", p2: "a"),
    .horizontal(id: "h1", of: .line(line: "ab")),
    .horizontal(id: "h2", of: .points(a: "c", b: "d")),
    .vertical(id: "v1", of: .line(line: "bc")),
    .vertical(id: "v2", of: .line(line: "da")),
    .distance(id: "w", a: "a", b: "b", value: .parameter("side")),
    .equal(id: "e", a: .id("ab"), b: .id("bc")),
]
let solved = try solve(request: SolveRequest(primitives: square))
check(solved.status == .converged, "square: \(solved.status)")
check(solved.dof == 0, "square dof: \(String(describing: solved.dof))")
check(near(point(byId(solved, "c")), (10, 10)), "c: \(point(byId(solved, "c")))")
check(solved.fullyConstrained?.contains("ab") == true, "ab fully constrained")

// analyze: false leaves the analysis out.
let quick = try solve(request: SolveRequest(primitives: square, analyze: false))
check(quick.dof == nil && quick.conflicting == nil, "analyze: false")

// An arc's sweep, and points on an ellipse's minor axis.
let curves: [Primitive] = [
    .point(id: "o", x: 0, y: 0, fixed: true),
    .point(id: "s", x: 2, y: 0),
    .point(id: "e", x: 0, y: 2),
    .arc(id: "arc", center: "o", start: "s", end: "e", radius: 2, startAngle: 0, endAngle: 1.5708),
    .angle(id: "sweep", of: .arc(arc: "arc"), value: .number(Double.pi / 3)),
    .point(id: "ec", x: 10, y: 0, fixed: true),
    .point(id: "ef", x: 13, y: 0, fixed: true),
    .ellipse(id: "el", center: "ec", focus: "ef", radmin: 2, fixed: true),
    .point(id: "m1", x: 10.5, y: 2.5),
    .point(id: "m2", x: 9.5, y: -1.5),
    .ellipseAxis(id: "minor", ellipse: "el", at: .pair(a: "m1", b: "m2"), which: .minor),
]
let curvesSolved = try solve(request: SolveRequest(primitives: curves))
check(curvesSolved.status == .converged, "curves: \(curvesSolved.status)")
let m1 = point(byId(curvesSolved, "m1"))
check(abs(m1.0 - 10) < 1e-8 && abs(abs(m1.1) - 2) < 1e-8, "m1: \(m1)")

// A rejection throws, naming the constraint.
do {
    _ = try solve(request: SolveRequest(primitives: square + [.horizontal(id: "bad", of: .line(line: "ghost"))]))
    check(false, "a bad constraint should throw")
} catch let SketchError.Invalid(message, constraintId) {
    check(constraintId == "bad", "constraintId: \(String(describing: constraintId))")
    check(message.contains("ghost"), message)
}

// A spline's curve without solving.
let curve = try splineCurve(handles: [Xy(x: 0, y: 0), Xy(x: 1, y: 2), Xy(x: 3, y: 2.5), Xy(x: 4, y: -1)], interpolated: true)
let first = curve.controlPoints.first!, last = curve.controlPoints.last!
check(curve.degree == 3 && near((first.x, first.y), (0, 0)) && near((last.x, last.y), (4, -1)), "curve: \(curve)")

// The C ABI, from the same module.
var response: UnsafeMutablePointer<CChar>?
let code = P3DSketch_Solve(#"{"version": 1, "primitives": []}"#, &response)
let text = String(cString: response!)
P3DSketch_Free(response)
check(code == 0 && text.contains("converged"), "C ABI: \(code) \(text)")

print("ok")
