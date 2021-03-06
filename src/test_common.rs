use parse::id_gen::IdGen;
use parse::tpe::primitive::build_type_type;
use parse::tree::{CompilationUnit, PrimitiveType, Type};
use parse::{apply_tokens, Tokens};
use tokenize::span::Span;
use tokenize::token::Token;
use {tokenize, JavaFile};

pub fn span(line: usize, col: usize, fragment: &str) -> Span {
    span2(line, col, fragment, std::ptr::null())
}

pub fn span2<'def>(
    line: usize,
    col: usize,
    fragment: &'def str,
    file: *const JavaFile<'def>,
) -> Span<'def> {
    Span {
        line,
        col,
        fragment,
        file,
    }
}

pub fn generate_tokens(fragment: &str) -> Vec<Token> {
    tokenize::apply(fragment.trim(), std::ptr::null())
        .ok()
        .unwrap()
}

pub fn primitive(line: usize, col: usize, name: &str) -> Type {
    Type::Primitive(PrimitiveType {
        span_opt: Some(span(line, col, name)),
        tpe: build_type_type(name).unwrap(),
    })
}

#[macro_export]
macro_rules! apply_parse {
    (vec $sources:expr) => {{
        let mut files = vec![];

        for (index, source) in $sources.iter().enumerate() {
            files.push(
                ::parse::apply(source.trim(), &format!("file{}.java", index))
                    .ok()
                    .unwrap(),
            );
        }

        files
    }};
    ($($source:expr),*) => {{
        apply_parse!(vec vec![$($source),*])
    }};
}

#[macro_export]
macro_rules! apply_assign_type {
    (vec $x:expr) => {{
        let files = apply_parse!(vec $x);

        let mut units = vec![];
        for file in &files {
            units.push(&file.unit);
        }

        let mut root = ::analyze::resolve::merge(&units);
        ::analyze::resolve::assign_type::apply(&mut root);

        (files, root)
    }};
    ($($x:expr),*) => {{
        apply_assign_type!(vec vec![$($x),*])
    }};
}

#[macro_export]
macro_rules! apply_assign_parameterized_type {
    (vec $x:expr) => {{
        let (files, mut root) = apply_assign_type!(vec $x);
        ::analyze::resolve::assign_parameterized_type::apply(&mut root);

        (files, root)
    }};
    ($($x:expr),*) => {{
        apply_assign_parameterized_type!(vec vec![$($x),*])
    }};
}

#[macro_export]
macro_rules! apply_semantics {
    (vec $x:expr) => {{
        let (mut files, root) = apply_assign_parameterized_type!(vec $x);
        let mut id_hash = ::semantics::id_hash::apply(&root);

        for file in &mut files {
            ::semantics::apply(&mut file.unit, &root, &mut id_hash);
        }

        (files, root)
    }};
    ($($x:expr),*) => {{
        apply_semantics!(vec vec![$($x),*])
    }};
}

#[macro_export]
macro_rules! unwrap {
    ($enum:path, $expr:expr) => {{
        if let $enum(item) = $expr {
            item
        } else {
            panic!()
        }
    }};
}

pub fn apply_analyze_build(source: &str) -> CompilationUnit {
    let tokens = generate_tokens(source);
    let mut id_gen = IdGen {
        uuid: 0,
        path: "".to_string(),
        runner: 0,
    };
    apply_tokens(&tokens, &mut id_gen).ok().unwrap()
}
