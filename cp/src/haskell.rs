// curry :: ((a, b) -> c) -> a -> b -> c

pub use either::{Either, Left, Right};

pub const fn curry<A, B, C>(f: impl FnOnce((A, B)) -> C) -> impl FnOnce(A, B) -> C {
    |a, b| f((a, b))
}

// uncurry :: (a -> b -> c) -> (a, b) -> c
pub const fn uncurry<A, B, C>(f: impl FnOnce(A, B) -> C) -> impl FnOnce((A, B)) -> C {
    |(a, b)| f(a, b)
}

// (.) :: (b -> c) -> (a -> b) -> a -> c
pub const fn dot<A, B, C>(f: impl FnOnce(B) -> C, g: impl FnOnce(A) -> B) -> impl FnOnce(A) -> C {
    |a| f(g(a))
}

pub const fn id<A>(a: A) -> A {
    a
}

pub const fn constant<A, B>(a: A) -> impl FnOnce(B) -> A {
    |_| a
}

pub const fn flip_once<A, B, C>(f: impl FnOnce(A, B) -> C) -> impl FnOnce(B, A) -> C {
    |b, a| f(a, b)
}

pub const fn flip_mut<A, B, C>(mut f: impl FnMut(A, B) -> C) -> impl FnMut(B, A) -> C {
    move |b, a| f(a, b)
}

pub const fn flip<A, B, C>(f: impl Fn(A, B) -> C) -> impl Fn(B, A) -> C {
    move |b, a| f(a, b)
}

pub const fn either<A, B, C>(
    f: impl FnOnce(A) -> C,
    g: impl FnOnce(B) -> C,
) -> impl FnOnce(Either<A, B>) -> C {
    |either| either.map_either(f, g).either_into()
}

pub trait HigherKindType {
    type H<B>;
}

pub trait Functor<A>: HigherKindType {
    fn map<B>(self, f: impl FnMut(A) -> B) -> Self::H<B>;
}

pub trait Applicative<A>: Functor<A> {
    fn pure(a: A) -> Self;
}

pub trait Monad<A>: Applicative<A> {
    fn bind<B>(self, f: impl FnMut(A) -> Self::H<B>) -> Self::H<B>;
}

impl<A> HigherKindType for Vec<A> {
    type H<B> = Vec<B>;
}

impl<A> Functor<A> for Vec<A> {
    fn map<B>(self, f: impl FnMut(A) -> B) -> Self::H<B> {
        self.into_iter().map(f).collect()
    }
}

impl<A> Applicative<A> for Vec<A> {
    fn pure(a: A) -> Self {
        vec![a]
    }
}

impl<A> Monad<A> for Vec<A> {
    fn bind<B>(self, f: impl FnMut(A) -> Self::H<B>) -> Self::H<B> {
        self.into_iter().flat_map(f).collect()
    }
}

impl<A> HigherKindType for Option<A> {
    type H<B> = Option<B>;
}

impl<A> Functor<A> for Option<A> {
    fn map<B>(self, f: impl FnMut(A) -> B) -> Self::H<B> {
        self.map(f)
    }
}

impl<A> Applicative<A> for Option<A> {
    fn pure(a: A) -> Self {
        Some(a)
    }
}

impl<A> Monad<A> for Option<A> {
    fn bind<B>(self, f: impl FnMut(A) -> Self::H<B>) -> Self::H<B> {
        self.and_then(f)
    }
}
