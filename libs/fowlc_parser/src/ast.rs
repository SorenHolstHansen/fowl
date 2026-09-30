use fowlc_interner::InternedStr;
use fowlc_lexer::TokenKind;
use syntree::{FlavorDefault, Node, node::Children};

#[derive(Clone)]
pub struct AstChildren<'a, N> {
    inner: Children<'a, TokenKind, FlavorDefault>,
    _data: std::marker::PhantomData<N>,
}

impl<'a, N> AstChildren<'a, N> {
    pub fn new(children: Children<'a, TokenKind, FlavorDefault>) -> Self {
        AstChildren {
            inner: children,
            _data: std::marker::PhantomData,
        }
    }
}

impl<'a, N: AstNode<'a>> Iterator for AstChildren<'a, N> {
    type Item = N;

    fn next(&mut self) -> Option<N> {
        self.inner.find_map(N::cast)
    }
}

#[inline]
pub(crate) fn child<'a, N: AstNode<'a>>(parent: Node<'a, TokenKind, FlavorDefault>) -> Option<N> {
    parent.children().find_map(N::cast)
}

pub trait AstNode<'a> {
    fn cast(node: Node<'a, TokenKind, FlavorDefault>) -> Option<Self>
    where
        Self: Sized;
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Visibility {
    Public,
    Internal,
    #[default]
    Private,
}

impl<'a> AstNode<'a> for Visibility {
    fn cast(node: Node<'a, TokenKind, FlavorDefault>) -> Option<Visibility> {
        match node.value() {
            TokenKind::Visibility => node.children().find_map(|c| match c.value() {
                TokenKind::Private => Some(Visibility::Private),
                TokenKind::Internal => Some(Visibility::Internal),
                TokenKind::Public => Some(Visibility::Public),
                _ => None,
            }),
            _ => None,
        }
    }
}

pub struct Identifier {
    inner: InternedStr,
}

impl<'a> AstNode<'a> for Identifier {
    fn cast(node: Node<'a, TokenKind, FlavorDefault>) -> Option<Self> {
        match node.value() {
            TokenKind::Identifier(i) => Some(Identifier { inner: i }),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct FnDef<'a> {
    node: Node<'a, TokenKind, FlavorDefault>,
}

impl<'a> AstNode<'a> for FnDef<'a> {
    fn cast(node: Node<'a, TokenKind, FlavorDefault>) -> Option<Self> {
        match node.value() {
            TokenKind::Fn => Some(FnDef { node }),
            _ => None,
        }
    }
}

impl<'a> FnDef<'a> {
    pub fn name(&self) -> Option<Identifier> {
        child(self.node.parent().unwrap())
    }

    pub fn vis(&self) -> Option<Visibility> {
        child(self.node.parent().unwrap())
    }
}

#[derive(Debug)]
pub enum Declaration<'a> {
    Fn(FnDef<'a>),
}

impl<'a> AstNode<'a> for Declaration<'a> {
    fn cast(node: Node<'a, TokenKind, FlavorDefault>) -> Option<Self> {
        match node.value() {
            TokenKind::Declaration => node.children().find_map(|c| match c.value() {
                TokenKind::Fn => Some(Declaration::Fn(FnDef::cast(c)?)),
                _ => None,
            }),
            _ => None,
        }
    }
}
