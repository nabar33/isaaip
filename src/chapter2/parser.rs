
use nom::{
    IResult,
    bytes::complete::tag,
    branch::alt,
    character::complete::{char, multispace0},
    Err,
    number::complete::double,
    sequence::{Tuple, delimited},
};

use super::language::*;

fn comma_separator(text: &str) -> IResult<&str, ()> {
    let (text, _) = (multispace0, char(','), multispace0).parse(text)?;
    Ok((text, ()))
}

fn semicolon_separator(text: &str) -> IResult<&str, ()> {
    let (text, _) = (multispace0, char(';'), multispace0).parse(text)?;
    Ok((text, ()))
}

fn float_pair(text: &str) -> IResult<&str, (f64, f64)> {
    let (text, (f1, _, f2)) = (double, comma_separator, double).parse(text)?;
    Ok((text, (f1, f2)))
}

fn float_triple(text: &str) -> IResult<&str, (f64, f64, f64)> {
    let (text, (f1, _, f2, _, f3)) = (double, comma_separator, double, comma_separator, double).parse(text)?;
    Ok((text, (f1, f2, f3)))
}

fn parenthesized_float_pair(text: &str) -> IResult<&str, (f64, f64)> {
    let (text, (_, _, float_pair, _, _)) = (char('('), multispace0, float_pair, multispace0, char(')')).parse(text)?;
    Ok((text, float_pair))
}

fn parenthesized_float_triple(text: &str) -> IResult<&str, (f64, f64, f64)> {
    let (text, (_, _, float_triple, _, _)) = (char('('), multispace0, float_triple, multispace0, char(')')).parse(text)?;
    Ok((text, float_triple))
}

fn translation_expression(text: &str) -> IResult<&str, Expression> {
    let (text, (_, _, (u, v))) = (tag("translation"), multispace0, parenthesized_float_pair).parse(text)?;
    Ok((text, Expression::Translation { u, v }))
}

fn rotation_expression(text: &str) -> IResult<&str, Expression> {
    let (text, (_, _, (u, v, theta))) = (tag("rotation"), multispace0, parenthesized_float_triple).parse(text)?;
    Ok((text, Expression::Rotation { u, v, theta }))
}

fn iterate_expression(text: &str) -> IResult<&str, Expression> {
    let (text, (_, _, _, _, body, _, _)) = (tag("iter"), multispace0, char('('), multispace0, expression, multispace0, char(')')).parse(text)?;
    Ok((text, Expression::Iterate(Box::new(body))))
}

fn eitheror_leaf(text: &str) -> IResult<&str, Expression> {
    let (text, (_, _, expr, _, _)) = (char('{'), multispace0, expression, multispace0, char('}')).parse(text)?;
    Ok((text, expr))
}

fn eitheror_expression(text: &str) -> IResult<&str, Expression> {
    let (text, (left, _, _, _, right)) = (eitheror_leaf, multispace0, tag("or"), multispace0, eitheror_leaf).parse(text)?;
    Ok((text, Expression::EitherOr { left: Box::new(left), right: Box::new(right) }))
}

fn expression(text: &str) -> IResult<&str, Expression> {
    let (remaining_text, expr) = alt((translation_expression, rotation_expression, iterate_expression, eitheror_expression))(text)?;
    match semicolon_separator(remaining_text) {
        Ok((text_after_semicolon, _)) => {
            let (trailing_text, additional_expression) = expression(text_after_semicolon)?;
            // check trailing_text
            Ok((trailing_text, Expression::Chained(Box::new(expr), Box::new(additional_expression))))
        }
        Err(nom::Err::Error(inner_error)) => {
            if inner_error.input.is_empty() || inner_error.input.starts_with(")") || inner_error.input.starts_with("}") {
                Ok((remaining_text, expr))
            } else {
                Err(nom::Err::Failure(inner_error))
            }
        },
        Err(error) => Err(error),
    }
}

#[test]
fn test_basic_expressions() {
    let raw_translation_expression = "translation ( 0.7, 18.65 )";
    let expected_expression = Expression::Translation { u: 0.7, v: 18.65 };
    let (_, parsed_expression) = translation_expression(raw_translation_expression).unwrap();
    assert_eq!(expected_expression, parsed_expression);

    let raw_rotation_expression = "rotation( 1.15, 0.8, 0.553)";
    let expected_expression = Expression::Rotation { u: 1.15, v: 0.8, theta: 0.553 };
    let (_, parsed_expression) = rotation_expression(raw_rotation_expression).unwrap();
    assert_eq!(expected_expression, parsed_expression);
}


#[test]
fn test_iterate_expressions() {
    let raw_expression = "iter( rotation(0.1, 0.2, 0.3) )";
    let expected_expression = Expression::Iterate( Box::new(
                                                Expression::Rotation { u: 0.1, v: 0.2, theta: 0.3 })
                                          );
    let (_, parsed_expression) = expression(raw_expression).unwrap();
    assert_eq!(expected_expression, parsed_expression);

    let raw_expression = "iter( iter( translation(0.1, 0.2) ) )";
    let expected_expression = Expression::Iterate( Box::new(
                                              Expression::Iterate(Box::new(
                                                  Expression::Translation { u: 0.1, v: 0.2 })
                                              ))
                                          );
    let (_, parsed_expression) = expression(raw_expression).unwrap();
    assert_eq!(expected_expression, parsed_expression);
}

#[test]
fn test_complex_expression() {
    let raw_expression = "iter(translation(12.0, 0.4); rotation(0.2, 0.3, 0.5)); translation( 8, 15 )";
    let expected_expression =
      Expression::Chained(Box::new(
        Expression::Iterate(Box::new(
            Expression::Chained(
                Box::new(Expression::Translation { u: 12.0, v: 0.4 }),
                Box::new(Expression::Rotation { u: 0.2, v: 0.3, theta: 0.5 })
            )
        ))),
        Box::new(Expression::Translation { u: 8.0, v: 15.0 }),
      );
    let (_, parsed_expression) = expression(raw_expression).unwrap();
    assert_eq!(expected_expression, parsed_expression);
}

#[test]
fn test_multiline_expression() {
    let raw_expression = 
r"iter(
    translation(12.0, 0.4);
    rotation(0.2, 0.3, 0.5)
);
translation( 8.0, 15.0 )
";
    let expected_expression =
      Expression::Chained(Box::new(
        Expression::Iterate(Box::new(
            Expression::Chained(
                Box::new(Expression::Translation { u: 12.0, v: 0.4 }),
                Box::new(Expression::Rotation { u: 0.2, v: 0.3, theta: 0.5 })
            )
        ))),
        Box::new(Expression::Translation { u: 8.0, v: 15.0 }),
      );
    let (_, parsed_expression) = expression(raw_expression).unwrap();
    assert_eq!(expected_expression, parsed_expression);
}
