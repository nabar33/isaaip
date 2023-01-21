
pub(crate) struct Point {
    x: f64,
    y: f64,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Expression {
    Translation{u: f64, v: f64},
    Rotation{u: f64, v: f64, theta: f64},
    Chained(Box<Expression>, Box<Expression>),
    EitherOr{left: Box<Expression>, right: Box<Expression>},
    Iterate(Box<Expression>),
}

pub(crate) struct Program {
    init: Point,
    body: Expression,
}
