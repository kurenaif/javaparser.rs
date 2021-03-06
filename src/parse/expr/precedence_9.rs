use parse::combinator::{get_and_not_followed_by, keyword, symbol, symbol2};
use parse::expr::precedence_10;
use parse::id_gen::IdGen;
use parse::tree::{BinaryOperation, Expr, InstanceOf};
use parse::{tpe, ParseResult, Tokens};
use tokenize::span::Span;

fn op<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>> {
    if let Ok(ok) = symbol2('<', '=')(input) {
        Ok(ok)
    } else if let Ok(ok) = symbol2('>', '=')(input) {
        Ok(ok)
    } else if let Ok(ok) = get_and_not_followed_by(symbol('<'), symbol('<'))(input) {
        Ok(ok)
    } else if let Ok(ok) = get_and_not_followed_by(symbol('>'), symbol('>'))(input) {
        Ok(ok)
    } else if let Ok(ok) = keyword("instanceof")(input) {
        Ok(ok)
    } else {
        Err(input)
    }
}

pub fn parse_tail<'def, 'r>(
    left: Expr<'def>,
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    if let Ok((input, operator)) = op(input) {
        if operator.fragment == "instanceof" {
            let (input, tpe) = tpe::parse(input)?;

            Ok((
                input,
                Expr::InstanceOf(InstanceOf {
                    expr: Box::new(left),
                    operator,
                    tpe,
                }),
            ))
        } else {
            let (input, right) = precedence_10::parse(input, id_gen)?;

            let expr = Expr::BinaryOperation(BinaryOperation {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            });

            parse_tail(expr, input, id_gen)
        }
    } else {
        Ok((input, left))
    }
}

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, left) = precedence_10::parse(input, id_gen)?;
    parse_tail(left, input, id_gen)
}

//#[cfg(test)]
//mod tests {
//    use test_common::{generate_tokens, span};
//
//    use super::parse;
//    use parse::tree::{ClassType, Expr, InstanceOf, Name, Type};
//    use parse::Tokens;
//
//    #[test]
//    fn test_instanceof() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//a instanceof Class
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::InstanceOf(InstanceOf {
//                    expr: Box::new(Expr::Name(Name {
//                        name: span(1, 1, "a")
//                    })),
//                    operator: span(1, 3, "instanceof"),
//                    tpe: Type::Class(ClassType {
//                        prefix_opt: None,
//                        name: span(1, 14, "Class"),
//                        type_args_opt: None,
//                        def_opt: None
//                    })
//                })
//            ))
//        );
//    }
//}
