use either::Either::{self, Left, Right};

pub trait Isomorphism<Out> {
    fn inn(out: Out) -> Self;
    fn out(self) -> Out;
}

// impl<T, Out> Isomorphism<Out> for T
// where
//     T: From<Out>,
//     Out: From<T>,
// {
//     fn inn(out: Out) -> Self {
//         Self::from(out)
//     }

//     fn out(self) -> Out {
//         Out::from(self)
//     }
// }

impl<T, U> Isomorphism<(U, T)> for (T, U) {
    fn inn(out: (U, T)) -> Self {
        out.out()
    }

    fn out(self) -> (U, T) {
        // split(p2, p1, out)
        (self.1, self.0)
    }
}

// impl<T, U, V> Isomorphism<(T, (U, V))> for ((T, U), V) {
//     fn inn(out: (T, (U, V))) -> Self {
//         ((out.0, (out.1).0), (out.1).1)
//     }

//     fn out(self) -> Self {
//         self.inn()
//     }
// }

impl<A, B, C> Isomorphism<(A, Either<B, C>)> for Either<(A, B), (A, C)> {
    fn inn(out: (A, Either<B, C>)) -> Self {
        match out {
            (a, Left(b)) => Left((a, b)),
            (a, Right(c)) => Right((a, c)),
        }
    }

    fn out(self) -> (A, Either<B, C>) {
        match self {
            Left((a, b)) => (a, Left(b)),
            Right((a, c)) => (a, Right(c)),
        }
    }
}

impl<A, B, C> Isomorphism<(A, B, C)> for (A, (B, C)) {
    fn inn((a, b, c): (A, B, C)) -> Self {
        (a, (b, c))
    }

    fn out(self) -> (A, B, C) {
        let (a, (b, c)) = self;
        (a, b, c)
    }
}

impl<A, B, C> Isomorphism<(A, B, C)> for ((A, B), C) {
    fn inn((a, b, c): (A, B, C)) -> Self {
        ((a, b), c)
    }

    fn out(self) -> (A, B, C) {
        let ((a, b), c) = self;
        (a, b, c)
    }
}

impl<A> Isomorphism<(A, ())> for A {
    fn inn(out: (A, ())) -> Self {
        out.0
    }

    fn out(self) -> (A, ()) {
        (self, ())
    }
}

// impl<A> Isomorphism<((), A)> for A {
//     fn inn(out: ((), A)) -> Self {
//         out.1
//     }

//     fn out(self) -> ((), A) {
//         ((), self)
//     }
// }
//

impl<A, B> Isomorphism<Either<A, B>> for Either<B, A> {
    fn inn(out: Either<A, B>) -> Self {
        out.out()
    }

    // either :: (a -> c) -> (b -> c) -> Either a b -> c
    //
    // coswap = either Right Left
    //
    fn out(self) -> Either<A, B> {
        self.either(Right, Left)
    }
}

// impl<A, B, C> Isomorphism<Either<Either<A, B>, C>> for Either<A, Either<B, C>> {
//     fn inn(out: Either<Either<A, B>, C>) -> Self {
//         todo!()
//     }

//     fn out(self) -> Either<Either<A, B>, C> {
//         todo!()
//     }
// }

impl<A, B, C> Isomorphism<(Either<C, A>, B)> for Either<(C, B), (A, B)> {
    fn inn((e, b): (Either<C, A>, B)) -> Self {
        match e {
            Left(c) => Left((c, b)),
            Right(a) => Right((a, b)),
        }
    }

    fn out(self) -> (Either<C, A>, B) {
        match self {
            Left((c, b)) => (Left(c), b),
            Right((a, b)) => (Right(a), b),
        }
    }
}

// trait Foo {

// }

// trait Bla {

// }

// impl<T> Foo for T where T: Bla {}

// impl<T, U> Foo for (T, U) {}

// impl Bla for (i32, i32) {}

// trait Foo: Sized {
//     fn foo() -> Option<Self>;
// }

// impl<T> Foo for T
// where
//     T: Default,
// {
//     fn foo() -> Option<Self> {
//         Some(Self::default())
//     }
// }

// #[derive(Default)]
// struct Lmao;

// impl Foo for Lmao {
//     fn foo() -> Option<Self> {
//         Some(Lmao)
//     }
// }

// ```hs
// class Default a where
//     def :: a
//
// class Foo a where
//     foo :: Maybe a
//
// instance Default a => Foo a where
//     foo = Just $ def
//
// ```
