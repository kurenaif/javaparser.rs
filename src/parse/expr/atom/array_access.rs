use parse::combinator::{opt, symbol};
use parse::id_gen::IdGen;
use parse::tree::{ArrayAccess, Expr};
use parse::{expr, ParseResult, Tokens};

pub fn parse_index<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, _) = symbol('[')(input)?;
    let (input, index) = expr::parse(input, id_gen)?;
    let (input, _) = symbol(']')(input)?;

    Ok((input, index))
}

pub fn parse_tail<'def, 'r>(
    input: Tokens<'def, 'r>,
    expr: Expr<'def>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, index_opt) = opt(|i| parse_index(i, id_gen))(input)?;

    match index_opt {
        Some(index) => parse_tail(
            input,
            Expr::ArrayAccess(ArrayAccess {
                expr: Box::new(expr),
                index: Box::new(index),
            }),
            id_gen,
        ),
        None => Ok((input, expr)),
    }
}

//#[cfg(test)]
//mod tests {
//    use parse::expr::atom;
//    use parse::tree::{ArrayAccess, Expr, Int, Name};
//    use parse::Tokens;
//    use test_common::{generate_tokens, span};
//
//    #[test]
//    fn test_multi() {
//        assert_eq!(
//            atom::parse(&generate_tokens(
//                r#"
//abc[1][2]
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::ArrayAccess(ArrayAccess {
//                    expr: Box::new(Expr::ArrayAccess(ArrayAccess {
//                        expr: Box::new(Expr::Name(Name {
//                            name: span(1, 1, "abc")
//                        })),
//                        index: Box::new(Expr::Int(Int {
//                            value: span(1, 5, "1")
//                        }))
//                    })),
//                    index: Box::new(Expr::Int(Int {
//                        value: span(1, 8, "2")
//                    }))
//                })
//            ))
//        );
//    }
//}
