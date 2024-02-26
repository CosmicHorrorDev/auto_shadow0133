use crate::{filter::HamReason, types::Token};

use super::{Context, Status};

use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use syn::Ident;

// These just have to be precise enough to reasonably not match Rust-Game related content
// // Function / method / function-like macro
// - [\w!]\(\)
// // Scope resolution op
// - \w::[\w\{]
// // semi-colon line end
// - ^.+;$
// // Template arguments
// - <[\w]+(, [\w]+)*,?>
// // Scope / block
// - Some { followed by a later }
pub fn filter(ctx: Context) -> Option<Status> {
    for token in ctx.post.tokens() {
        if let Token::Code { lang, text } = token {
            if let Some(lang) = lang {
                return Some(Status::Ham(HamReason::FencedCodeBlock(lang)));
            }

            if let Some(heuristic) = is_rust(&text) {
                return Some(Status::Ham(HamReason::DetectedRustCode(heuristic)));
            }
        }
    }

    None
}

#[derive(Default)]
struct StateMachine {
    state: State,
}

impl StateMachine {
    fn new() -> Self {
        Self::default()
    }

    fn munch(&mut self, token: &TokenTree) {
        let prev_state = std::mem::take(&mut self.state);

        self.state = match (prev_state, token) {
            // Final state is a trap
            (f @ State::Final(_), _) => f,
            // Look for either a curly-brace group, or check if we finish off an empty function /
            // method / function-like macro call. The reason we check for empty is because
            // Rust-Game text may have text in parens, but is unlikely to have empty parens
            (state, TokenTree::Group(group)) => {
                if group.delimiter() == Delimiter::Brace {
                    State::Final(Heuristic::CurlyBracePair)
                } else {
                    let empty_parens =
                        group.delimiter() == Delimiter::Parenthesis && group.stream().is_empty();
                    match (empty_parens, state) {
                        (true, State::Ident(ident)) => State::Final(Heuristic::empty_fn(ident)),
                        (true, State::ExclamationPoint(ident)) => {
                            State::Final(Heuristic::empty_fn_like_macro(ident))
                        }
                        (true, State::Method(ident)) => {
                            State::Final(Heuristic::empty_method(ident))
                        }
                        _ => State::Limbo,
                    }
                }
            }
            (State::SecondColon(lft), TokenTree::Ident(rgt)) => {
                State::Final(Heuristic::double_colon(lft, rgt.to_owned()))
            }
            (State::Dot, TokenTree::Ident(ident)) => State::Method(ident.to_owned()),
            (_, TokenTree::Ident(ident)) => {
                // Bail early if we see a unique keyword
                if let Some(keyword) = Keyword::new(&ident.to_string()) {
                    State::Final(Heuristic::Keyword(keyword))
                } else {
                    State::Ident(ident.to_owned())
                }
            }
            (State::Ident(ident), TokenTree::Punct(punct)) => {
                let c = punct.as_char();
                let spacing = punct.spacing();

                match (c, spacing) {
                    ('!', Spacing::Alone) => State::ExclamationPoint(ident),
                    (':', Spacing::Joint) => State::FirstColon(ident),
                    ('.', Spacing::Alone) => State::Dot,
                    _ => State::Limbo,
                }
            }
            (State::FirstColon(ident), TokenTree::Punct(punct)) => {
                if punct.as_char() == ':' && punct.spacing() == Spacing::Alone {
                    State::SecondColon(ident)
                } else {
                    State::Limbo
                }
            }
            (_, TokenTree::Punct(punct)) => {
                if punct.as_char() == '.' && punct.spacing() == Spacing::Alone {
                    State::Dot
                } else {
                    State::Limbo
                }
            }
            _ => State::Limbo,
        };
    }

    fn finished(&self) -> Option<Heuristic> {
        if let State::Final(heuristic) = &self.state {
            Some(heuristic.to_owned())
        } else {
            None
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
enum State {
    #[default]
    Limbo,
    Dot,
    Method(Ident),
    Ident(Ident),
    FirstColon(Ident),
    SecondColon(Ident),
    ExclamationPoint(Ident),
    Final(Heuristic),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Heuristic {
    DoubleColon(String),
    CurlyBracePair,
    EmptyFunction(String),
    EmptyMethod(String),
    EmptyFunctionlikeMacro(String),
    Keyword(Keyword),
}

impl Heuristic {
    fn double_colon(i1: Ident, i2: Ident) -> Self {
        Self::DoubleColon(format!("{i1}::{i2}"))
    }

    fn empty_fn(i: Ident) -> Self {
        Self::EmptyFunction(format!("{i}()"))
    }

    fn empty_method(i: Ident) -> Self {
        Self::EmptyMethod(format!(".{i}()"))
    }

    fn empty_fn_like_macro(i: Ident) -> Self {
        Self::EmptyFunctionlikeMacro(format!("{i}!()"))
    }
}

/// Keywords that are unlikely to appear in Rust-Game posts
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Keyword {
    BTreeMap,
    Derive,
    Enum,
    Eq,
    F32,
    F64,
    Fn,
    HashMap,
    I8,
    I16,
    I32,
    I64,
    I128,
    Impl,
    PartialEq,
    Println,
    RwLock,
    Self_,
    U8,
    U16,
    U32,
    U64,
    U128,
    Vec,
}

impl Keyword {
    fn new(s: &str) -> Option<Self> {
        let keyword = match s {
            "BTreeMap" => Self::BTreeMap,
            "derive" => Self::Derive,
            "enum" => Self::Enum,
            "Eq" => Self::Eq,
            "f32" => Self::F32,
            "f64" => Self::F64,
            "fn" => Self::Fn,
            "HashMap" => Self::HashMap,
            "i8" => Self::I8,
            "i16" => Self::I16,
            "i32" => Self::I32,
            "i64" => Self::I64,
            "i128" => Self::I128,
            "impl" => Self::Impl,
            "PartialEq" => Self::PartialEq,
            "println" => Self::Println,
            "RwLock" => Self::RwLock,
            "Self" => Self::Self_,
            "u8" => Self::U8,
            "u16" => Self::U16,
            "u32" => Self::U32,
            "u64" => Self::U64,
            "u128" => Self::U128,
            "Vec" | "vec" => Self::Vec,
            _ => return None,
        };

        Some(keyword)
    }
}

fn is_rust(s: &str) -> Option<Heuristic> {
    let tokens: TokenStream = syn::parse_str(s).ok()?;
    is_rust_helper(tokens)
}

fn is_rust_helper(tokens: TokenStream) -> Option<Heuristic> {
    let mut it = tokens.into_iter();
    let mut state_machine = StateMachine::new();

    while let Some(token) = it.next() {
        state_machine.munch(&token);

        if let ret @ Some(_) = state_machine.finished() {
            return ret;
        }

        if let TokenTree::Group(group) = token {
            if let ret @ Some(_) = is_rust_helper(group.stream()) {
                return ret;
            }
        }
    }

    None
}
