use parse::combinator::symbol;
use parse::def::{class, modifiers};
use parse::id_gen::IdGen;
use parse::tree::Statement;
use parse::{expr, ParseResult, Tokens};

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Statement<'def>> {
    let (input, modifiers) = modifiers::parse(input, id_gen)?;
    let (input, _) = class::parse_prefix(input)?;
    let (input, class) = class::parse_tail(input, modifiers, id_gen)?;

    Ok((input, Statement::Class(class)))
}

//#[cfg(test)]
//mod tests {
//    use super::parse;
//    use parse::tree::{
//        ArrayAccess, Assigned, Assignment, Class, ClassBody, Expr, FieldAccess, Int, Keyword,
//        MethodCall, Modifier, Name, Statement,
//    };
//    use parse::Tokens;
//    use std::cell::RefCell;
//    use test_common::{generate_tokens, span};
//
//    #[test]
//    fn test() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//strictfp class Test {}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Statement::Class(Class {
//                    modifiers: vec![Modifier::Keyword(Keyword {
//                        name: span(1, 1, "strictfp")
//                    })],
//                    name: span(1, 16, "Test"),
//                    type_params: vec![],
//                    extend_opt: None,
//                    implements: vec![],
//                    body: ClassBody { items: vec![] },
//                    def_opt: RefCell::new(None),
//                })
//            ))
//        );
//    }
//}
