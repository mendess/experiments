// (1) Product ------------------------------------------------------------------------------------

use crate::{cp, haskell::*};

/// ```
/// split :: (a -> b) -> (a -> c) -> a -> (b,c)
/// split f g x = (f x, g x)
/// ```
pub const fn split<A, B, C>(
    f: impl FnOnce(A) -> B,
    g: impl FnOnce(A) -> C,
) -> impl FnOnce(A) -> (B, C)
where
    A: Clone,
{
    |a| (f(a.clone()), g(a))
}

/// ```
/// (><) :: (a -> b) -> (c -> d) -> (a,c) -> (b,d)
/// f >< g = split (f . p1) (g . p2)
/// ```
pub const fn cross<A, B, C, D>(
    f: impl FnOnce(A) -> B,
    g: impl FnOnce(C) -> D,
) -> impl FnOnce((A, C)) -> (B, D) {
    |(a, c)| (f(a), g(c))
}

// the 0-adic split -------------------------------------------------------------------------------

/// ```
/// (!) :: a -> ()
/// (!) = const ()
/// ```
pub fn bang<A>(a: A) {
    constant(())(a)
}

// Renamings --------------------------------------------------------------------------------------

/// ```
/// p1        = fst
/// ```
pub fn p1<A, B>((a, _): (A, B)) -> A {
    a
}

/// ```
/// p2        = snd
/// ```
pub fn p2<A, B>((_, b): (A, B)) -> B {
    b
}

// (2) Coproduct ----------------------------------------------------------------------------------

// Renamings

/// ```
/// i1      = Left
/// ```
pub const fn i1<A, B>(a: A) -> Either<A, B> {
    Either::Left(a)
}

/// ```
/// i2      = Right
/// ```
pub const fn i2<A, B>(b: B) -> Either<A, B> {
    Either::Right(b)
}

/// ```
/// (-|-) :: (a -> b) -> (c -> d) -> Either a c -> Either b d
/// f -|- g = either (i1 . f) (i2 . g)
/// ```
pub const fn plus<A, B, C, D>(
    f: impl FnOnce(A) -> B,
    g: impl FnOnce(C) -> D,
) -> impl FnOnce(Either<A, C>) -> Either<B, D> {
    |x| x.map_either(f, g)
}

// McCarthy's conditional:

/// ```
/// cond p f g = (either f g) . (grd p)
/// ```
pub const fn cond<A, B>(
    predicate: impl FnOnce(&A) -> bool,
    then: impl FnOnce(A) -> B,
    r#else: impl FnOnce(A) -> B,
) -> impl FnOnce(A) -> B {
    dot(either(then, r#else), grd(predicate))
}

// (3) Exponentiation

/// ```
/// ap :: (a -> b,a) -> b
/// ap = uncurry ($)
/// ```
pub fn ap<A, B>(t: (impl FnOnce(A) -> B, A)) -> B {
    (t.0)(t.1)
}

/// ```
/// expn :: (b -> c) -> (a -> b) -> a -> c
/// expn f = curry (f . ap)
/// ```
pub const fn expn<A, B, C>(f: impl FnOnce(B) -> C, g: impl FnOnce(A) -> B) -> impl FnOnce(A) -> C {
    |a| curry(cp!(f.ap))(g, a)
}

/// ```
/// p2p :: (a, a) -> Bool -> a
/// p2p p b = if b then (snd p) else (fst p) -- pair to predicate
/// ```
pub fn p2p<A>(pair: (A, A), b: bool) -> A {
    cond(|_| b, p2, p1)(pair)
}

// (4) Others -------------------------------------------------------------------------------------

/// ```
/// grd :: (a -> Bool) -> a -> Either a a
/// grd p x = if p x then Left x else Right x
/// ```
pub const fn grd<A>(predicate: impl FnOnce(&A) -> bool) -> impl FnOnce(A) -> Either<A, A> {
    |a| if predicate(&a) { i1(a) } else { i2(a) }
}
// (5) Natural isomorphisms

/// ```
/// swap :: (a,b) -> (b,a)
/// swap = split p2 p1
/// ```
pub fn swap<A, B>((a, b): (A, B)) -> (B, A) {
    (b, a)
}

/// ```
/// assocr :: ((a,b),c) -> (a,(b,c))
/// assocr = split ( p1 . p1 ) (p2 >< id)
/// ```
pub fn assocr<A, B, C>(tuple: ((A, B), C)) -> (A, (B, C))
where
    ((A, B), C): Clone,
{
    split(
        dot::<_, (A, _), _>(p1, p1),
        cross::<(A, _), _, _, _>(p2, id),
    )(tuple)
}

/// ```
/// assocl :: (a,(b,c)) -> ((a,b),c)
/// assocl = split ( id >< p1 ) ( p2 . p2 )
/// ```
pub fn assocl<A, B, C>(tuple: (A, (B, C))) -> ((A, B), C)
where
    (A, (B, C)): Clone,
    B: Clone,
{
    split(
        cross::<_, _, (_, C), _>(id, p1),
        dot::<_, (_, C), _>(p2, p2),
    )(tuple)
}

/// ```
/// undistr :: Either (a,b) (a,c) -> (a,Either b c)
/// undistr = either ( id >< i1 ) ( id >< i2 )
/// ```
pub fn undistr<A, B, C>(e: Either<(A, B), (A, C)>) -> (A, Either<B, C>) {
    either(cp!(id >< i1), cp!(id >< i2))(e)
}

/// ```
/// undistl :: Either (b, c) (a, c) -> (Either b a, c)
/// undistl = either ( i1 >< id ) ( i2 >< id )
/// ```
pub fn undistl<A, B, C>(e: Either<(B, C), (A, C)>) -> (Either<B, A>, C) {
    either(cp!(i1 >< id), cp!(i2 >< id))(e)
}

/// ```
/// flatr :: (a,(b,c)) -> (a,b,c)
/// flatr (a,(b,c)) = (a,b,c)
/// ```
pub fn flatr<A, B, C>((a, (b, c)): (A, (B, C))) -> (A, B, C) {
    (a, b, c)
}

/// ```
/// flatl :: ((a,b),c) -> (a,b,c)
/// flatl ((b,c),d) = (b,c,d)
/// ```
pub fn flatl<A, B, C>(((b, c), a): ((B, C), A)) -> (B, C, A) {
    (b, c, a)
}

// pwnil = split id (!)

/// ```
/// br :: a -> (a, ())
/// br = split id (!) -- bang on the right, old pwnil means "pair with nil"
/// ```
pub const fn br<A>(a: A) -> (A, ()) {
    // split(id, ())(a) clone bound
    (a, ())
}

/// ```
/// bl :: a -> ((), a)
/// bl = swap . br
/// ```
pub fn bl<A>(a: A) -> ((), A) {
    cp!(swap.br)(a)
}

/// ```
/// coswap :: Either a b -> Either b a
/// coswap = either i2 i1
/// ```
pub fn coswap<A, B>(e: Either<A, B>) -> Either<B, A> {
    either(i2, i1)(e)
}

/// ```
/// coassocr :: Either (Either a b) c -> Either a (Either b c)
/// coassocr = either (id -|- i1) (i2 . i2)
/// ```
pub fn coassocr<A, B, C>(e: Either<Either<A, B>, C>) -> Either<A, Either<B, C>> {
    either(cp!(id -|- i1), cp!(i2.i2))(e)
}

/// ```
/// coassocl :: Either b (Either a c) -> Either (Either b a) c
/// coassocl = either (i1.i1) (i2 -|- id)
/// ```
pub fn coassocl<A, B, C>(e: Either<B, Either<A, C>>) -> Either<Either<B, A>, C> {
    either(cp!(i1.i1), cp!(i2 -|- id))(e)
}

/// ```
/// distl :: (Either c a, b) -> Either (c, b) (a, b)
/// distl = uncurry (either (curry i1)(curry i2))
/// ```
pub fn distl<A, B, C>(tuple: (Either<C, A>, B)) -> Either<(C, B), (A, B)> {
    // I've been defeated
    // uncurry(either(curry(i1), curry(i2)))(tuple)
    let (e, b) = tuple;
    match e {
        Either::Left(c) => Either::Left((c, b)),
        Either::Right(a) => Either::Right((a, b)),
    }
}

/// ```
/// distr :: (b, Either c a) -> Either (b, c) (b, a)
/// distr = (swap -|- swap) . distl . swap
/// ```
pub fn distr<A, B, C>(tuple: (B, Either<C, A>)) -> Either<(B, C), (B, A)> {
    dot(cp!(swap -|- swap), cp!(distl.swap))(tuple)
}

// (6) Class bifunctor
/// ```
/// class BiFunctor f where
///      bmap :: (a -> b) -> (c -> d) -> (f a c -> f b d)
/// ```
trait BiFunctor<A, C> {
    fn bmap<B, D>(self, f: impl FnOnce(A) -> B, g: impl FnOnce(C) -> D) -> impl BiFunctor<B, D>;
}

/// ```
/// instance BiFunctor Either where
///    bmap f g = f -|- g
/// ```
impl<A, C> BiFunctor<A, C> for Either<A, C> {
    fn bmap<B, D>(self, f: impl FnOnce(A) -> B, g: impl FnOnce(C) -> D) -> impl BiFunctor<B, D> {
        cp!(f -|- g)(self)
    }
}

/// ```
/// instance BiFunctor (,) where
///    bmap f g  = f >< g
/// ```
impl<A, B> BiFunctor<A, B> for (A, B) {
    fn bmap<C, D>(self, f: impl FnOnce(A) -> C, g: impl FnOnce(B) -> D) -> impl BiFunctor<C, D> {
        cp!(f >< g)(self)
    }
}

// (7) Monads:

// (7.1) Kleisli monadic composition

/// ```
/// (.!) :: Monad a => (b -> a c) -> (d -> a b) -> d -> a c
/// (f .! g) a = (g a) >>= f
/// ```
pub const fn monad_dot<B, C, D, M1, M2>(
    f: impl FnMut(B) -> M1::H<C>,
    g: impl FnOnce(D) -> M1,
) -> impl FnOnce(D) -> M1::H<C>
where
    M1: Monad<B>,
{
    |d| g(d).bind(f)
}

/// ```
/// mult :: (Monad m) => m (m b) -> m b
/// -- also known as join
/// mult = (>>= id)
/// ```
// pub fn mult<A, M>(m: M) -> M::Bind<A>
// where
//     M: Monad<MI>,
//     MI: Monad<A, Bind<A> = M>,
// {
//     m.bind(id)
// }
// pub fn mult_vec<A>(m: Vec<Vec<A>>) -> Vec<A> {
//     m.bind(id)
// }

// // (7.2) Monadic binding

/// ```
/// ap' :: (Monad m) => (a -> m b, m a) -> m b
/// ap' = uncurry (flip (>>=))
/// ```
pub fn ap_m<A, B, M>(tuple: (impl FnMut(A) -> M::H<B>, M)) -> M::H<B>
where
    M: Monad<A>,
{
    uncurry(flip_mut(Monad::bind))(tuple)
}

// (7.3) Lists

/// ```
/// singl :: a -> [a]
/// singl = return
/// ```
pub fn singl<A>(a: A) -> Vec<A> {
    Applicative::pure(a)
}

// (7.4)
/// ```
/// class (Functor f, Monad f) => Strong f where
///      rstr :: (f a,b) -> f(a,b)
///      rstr(x,b) = do a <- x ; return (a,b)
///      lstr :: (b,f a) -> f(b,a)
///      lstr(b,x) = do a <- x ; return (b,a)
/// ```
trait Strong<A>: Monad<A> + Sized {
    fn rstr<B>((x, b): (Self, B)) -> Self::H<(A, B)>
    where
        B: Clone,
    {
        x.map(move |a| (a, b.clone()))
    }
    fn lstr<B>((b, x): (B, Self)) -> Self::H<(B, A)>
    where
        B: Clone,
    {
        x.map(move |a| (b.clone(), a))
    }
}

/// ```
/// instance Strong []
/// ```
impl<A> Strong<A> for Vec<A> {}

/// ```
/// instance Strong Maybe
/// ```
impl<A> Strong<A> for Option<A> {}

/// ```
/// dstr :: Strong m => (m a, m b) -> m (a, b)       --- double strength
/// --dstr = mult . fmap rstr . lstr
/// dstr = rstr .! lstr
/// ```
// pub fn dstr<SA, SB, A, B>(ma: SA, mb: SB) -> impl Strong<(A, B)>
// where
//     SA: Strong<A> + Clone,
//     SB: Strong<B> + Clone,
//     SB::H<(SA, B)>: Monad<(SA, B)>,
//     B: Clone,
// {
//     let x = SB::lstr((ma, mb));
//     let y = x.bind(|x| SA::rstr(x));
//     // monad_dot(SA::rstr, SB::lstr)((ma, mb))
//     todo!()
// }
pub fn dstr() {}

/// ```
/// splitm :: Strong ff => ff (a -> b) -> a -> ff b
/// -- Exercise 4.8.13 in Jacobs' "Introduction to Coalgebra" (2012)
/// splitm = curry (fmap ap . rstr)
/// ```
pub fn splitm() {}

#[macro_export]
macro_rules! cp {
    ($f:tt >< $g:tt) => {
        $crate::cp::cross($f, $g)
    };
    ($f:tt -|- $g:tt) => {
        $crate::cp::plus($f, $g)
    };
    ($f:tt . $g:tt) => {
        $crate::haskell::dot($f, $g)
    };
    ($f:tt .! $g:tt) => {
        $crate::monad_dot($f, $g)
    };
}
