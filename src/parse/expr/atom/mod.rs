use either::Either;
use parse::combinator::{identifier, symbol, symbol2};
use parse::id_gen::IdGen;
use parse::tpe::{primitive, type_args};
use parse::tree::{Boolean, Expr, Keyword, MethodCall, Name, Null, Super, This, Type};
use parse::{tpe, ParseResult, Tokens};
use std::cell::RefCell;

pub mod array_access;
pub mod array_initializer;
pub mod constructor_call;
pub mod lambda;
pub mod literal_char;
pub mod method_call;
pub mod name;
pub mod new_array;
pub mod new_object;
pub mod number;
pub mod parenthesized;
pub mod string;

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    if let Ok(ok) = number::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = string::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = literal_char::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = array_initializer::parse(input, id_gen) {
        Ok(ok)
    } else if let Ok(ok) = constructor_call::parse(input, id_gen) {
        Ok(ok)
    } else if let Ok(ok) = parse_prefix_keyword_or_identifier(input, id_gen) {
        Ok(ok)
    } else if let Ok(ok) = parse_lambda_or_parenthesized(input, id_gen) {
        Ok(ok)
    } else {
        Err(input)
    }
}

fn parse_lambda_or_parenthesized<'def, 'r>(
    original: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, _) = symbol('(')(original)?;

    if let Ok((input, _)) = symbol(')')(input) {
        if let Ok((input, _)) = symbol2('-', '>')(input) {
            return lambda::parse(original, id_gen);
        }
    }

    if let Ok(_) = primitive::parse(input) {
        return lambda::parse(original, id_gen);
    }

    let (input, ident) = match identifier(input) {
        Ok(ok) => ok,
        Err(_) => return parenthesized::parse(original, id_gen),
    };

    // a single unknown type param name
    if let Ok((input, _)) = symbol(')')(input) {
        if let Ok(_) = symbol2('-', '>')(input) {
            return lambda::parse(original, id_gen);
        }
    }

    // a param name with type
    if let Ok((
        _,
        Either::Right(Name {
            name: _,
            resolved_opt: _,
        }),
    )) = name::parse(input)
    {
        return lambda::parse(original, id_gen);
    }

    // Unknown type first param
    if let Ok((input, _)) = symbol(',')(input) {
        return lambda::parse(original, id_gen);
    }

    // The first param has typed with type arg
    if let Ok((input, Some(_))) = tpe::type_args::parse(input) {
        if let Ok((
            _,
            Either::Right(Name {
                name: _,
                resolved_opt: _,
            }),
        )) = name::parse(input)
        {
            return lambda::parse(original, id_gen);
        }
    }

    parenthesized::parse(original, id_gen)
}

fn parse_new_object_or_array<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    if let Ok((input, Some(type_args))) = type_args::parse(input) {
        return new_object::parse_tail2(None, input, Some(type_args), id_gen);
    }

    let (input, tpe) = tpe::parse_no_array(input)?;
    let copied = tpe.clone();

    if let Ok((input, expr)) = new_array::parse_tail(input, tpe, id_gen) {
        Ok((input, expr))
    } else {
        match copied {
            Type::Class(class) => new_object::parse_tail3(None, input, None, class, id_gen),
            _ => Err(input),
        }
    }
}

fn parse_prefix_keyword_or_identifier<'def, 'r>(
    original: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, keyword_or_name) = name::parse(original)?;

    match keyword_or_name {
        Either::Left(keyword) => match keyword.name.fragment {
            "true" | "false" => Ok((
                input,
                Expr::Boolean(Boolean {
                    value: keyword.name,
                }),
            )),
            "null" => Ok((
                input,
                Expr::Null(Null {
                    value: keyword.name,
                }),
            )),
            "new" => parse_new_object_or_array(input, id_gen),
            "this" => Ok((
                input,
                Expr::This(This {
                    tpe_opt: None,
                    span: keyword.name,
                }),
            )),
            "super" => Ok((
                input,
                Expr::Super(Super {
                    tpe_opt: None,
                    span: keyword.name,
                }),
            )),
            _ => Err(input),
        },
        Either::Right(name) => {
            if let Ok(_) = symbol2('-', '>')(input) {
                lambda::parse(original, id_gen)
            } else if let Ok((input, args)) = method_call::parse_args(input, id_gen) {
                Ok((
                    input,
                    Expr::MethodCall(MethodCall {
                        prefix_opt: None,
                        name: name.name,
                        type_args_opt: None,
                        args,
                        def_opt: RefCell::new(None),
                    }),
                ))
            } else {
                array_access::parse_tail(input, Expr::Name(name), id_gen)
            }
        }
    }
}
