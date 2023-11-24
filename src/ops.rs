use crate::{
    parse::{macros::impl_parse, ParseImpl},
    Concat, Error, HardError, Result, SourceBase,
};
use alloc::vec::Vec;
use core::{borrow::Borrow, marker::PhantomData};

pub trait Const {
    type Type;
    const VALUE: Self::Type;
}

pub trait Predicate<T> {
    fn is(v: &T) -> bool;
}

pub trait Mapper<T> {
    type Output;
    fn map(v: T) -> Self::Output;
}

pub struct Define<T>(PhantomData<T>);

impl<T, U: Mapper<T>> Mapper<T> for Define<U> {
    type Output = U::Output;
    #[inline(always)]
    fn map(v: T) -> Self::Output {
        U::map(v)
    }
}

impl<T, U: Predicate<T>> Predicate<T> for Define<U> {
    #[inline(always)]
    fn is(v: &T) -> bool {
        U::is(v)
    }
}

impl<T: Const> Const for Define<T> {
    type Type = T::Type;
    const VALUE: Self::Type = T::VALUE;
}

impl<I, U: ParseImpl<I>> ParseImpl<I> for Define<U> {
    type Output = U::Output;

    #[inline(always)]
    fn parse<S: crate::Source<Item = I>>(input: &mut S) -> Result<Self::Output> {
        U::parse(input)
    }

    #[cfg(feature = "async")]
    #[inline(always)]
    async fn parse_async<S: crate::AsyncSource<Item = I>>(input: &mut S) -> Result<Self::Output> {
        U::parse_async(input).await
    }
}

pub struct Is<P>(PhantomData<P>);

impl<I: Clone, P: Predicate<I>> ParseImpl<I> for Is<P> {
    type Output = (I,);
    impl_parse!(parse, _await, |input: I| {
        if let Some(item) = _await!(input.read(1))?.first() {
            if P::is(item) {
                let item = item.clone();
                input.consume(1);
                return Ok((item,));
            }
        }

        Err(Error::Mismatch)
    });
}

impl<I, P: Predicate<I>> Predicate<I> for Is<P> {
    #[inline(always)]
    fn is(v: &I) -> bool {
        P::is(v)
    }
}

pub type A<C> = AnyOf<AsMany<C>>;

pub struct Not<T>(PhantomData<T>);

impl<I, T: Predicate<I>> Predicate<I> for Not<T> {
    #[inline(always)]
    fn is(v: &I) -> bool {
        !T::is(v)
    }
}

impl<I: Clone, T: Predicate<I>> ParseImpl<I> for Not<T> {
    type Output = <Is<Self> as ParseImpl<I>>::Output;
    impl_parse!(parse, _await, |input: I| parse!(Is::<Self>, input));
}

pub type AsMany<T> = ConstSome<T>;

pub struct ConstSome<T>(PhantomData<T>);

impl<T: Const> Const for ConstSome<T> {
    type Type = Option<T::Type>;
    const VALUE: Self::Type = Some(T::VALUE);
}

pub struct ConstNone<T>(PhantomData<T>);

impl<T> Const for ConstNone<T> {
    type Type = Option<T>;
    const VALUE: Self::Type = None;
}

pub struct ConstUSize<const VALUE: usize>;

impl<const VALUE: usize> Const for ConstUSize<VALUE> {
    type Type = usize;
    const VALUE: Self::Type = VALUE;
}

pub struct AnyOf<C>(PhantomData<C>);

impl<C: Const<Type = G>, G: IntoIterator<Item = T>, T: Borrow<U>, U: PartialEq> Predicate<U>
    for AnyOf<C>
{
    #[inline(always)]
    fn is(v: &U) -> bool {
        C::VALUE.into_iter().any(|x| v == x.borrow())
    }
}

impl<C: Const<Type = G>, G: IntoIterator<Item = T>, T: Borrow<U>, U: PartialEq + Clone> ParseImpl<U>
    for AnyOf<C>
{
    type Output = <Is<Self> as ParseImpl<U>>::Output;
    impl_parse!(parse, _await, |input: U| parse!(Is::<Self>, input));
}

pub struct Seq<C>(PhantomData<C>);

impl<C: Const<Type = G>, G: IntoIterator<Item = T>, T: Borrow<I>, I: PartialEq + Clone> ParseImpl<I>
    for Seq<C>
{
    type Output = (Vec<I>,);
    impl_parse!(parse, _await, |input: I| {
        let i = C::VALUE.into_iter();
        let mut count = 0;
        for item in i {
            match _await!(input.read(count + 1))?.get(count) {
                Some(read) if read == item.borrow() => {
                    count += 1;
                }
                _ => return Err(Error::Mismatch),
            }
        }
        let result = _await!(input.read(count))?.to_vec();
        input.consume(count);
        Ok((result,))
    });
}

pub struct Discard<T = NoOp>(PhantomData<T>);

pub struct NoOp;

pub struct Never<T>(PhantomData<T>);

impl<I, T: ParseImpl<I>> ParseImpl<I> for Discard<T> {
    type Output = ();
    impl_parse!(parse, _await, |input: I| {
        parse!(T, input)?;
        Ok(())
    });
}

impl<I> ParseImpl<I> for NoOp {
    type Output = ();
    impl_parse!(parse, _await, |__: I| Ok(()));
}

impl<I, O, T: ParseImpl<I, Output = O>> ParseImpl<I> for Never<T> {
    type Output = O;
    impl_parse!(parse, _await, |__: I| Err(Error::Mismatch));
}

#[allow(clippy::type_complexity)]
pub struct And<
    T0 = NoOp,
    T1 = NoOp,
    T2 = NoOp,
    T3 = NoOp,
    T4 = NoOp,
    T5 = NoOp,
    T6 = NoOp,
    T7 = NoOp,
    T8 = NoOp,
    T9 = NoOp,
    T10 = NoOp,
    T11 = NoOp,
    T12 = NoOp,
    T13 = NoOp,
    T14 = NoOp,
    T15 = NoOp,
>(
    PhantomData<(
        T0,
        T1,
        T2,
        T3,
        T4,
        T5,
        T6,
        T7,
        T8,
        T9,
        T10,
        T11,
        T12,
        T13,
        T14,
        T15,
    )>,
);

impl<
        I,
        T0: ParseImpl<I, Output = C0>,
        T1: ParseImpl<I>,
        T2: ParseImpl<I>,
        T3: ParseImpl<I>,
        T4: ParseImpl<I>,
        T5: ParseImpl<I>,
        T6: ParseImpl<I>,
        T7: ParseImpl<I>,
        T8: ParseImpl<I>,
        T9: ParseImpl<I>,
        T10: ParseImpl<I>,
        T11: ParseImpl<I>,
        T12: ParseImpl<I>,
        T13: ParseImpl<I>,
        T14: ParseImpl<I>,
        T15: ParseImpl<I>,
        C0: Concat<T1::Output, Output = C1>,
        C1: Concat<T2::Output, Output = C2>,
        C2: Concat<T3::Output, Output = C3>,
        C3: Concat<T4::Output, Output = C4>,
        C4: Concat<T5::Output, Output = C5>,
        C5: Concat<T6::Output, Output = C6>,
        C6: Concat<T7::Output, Output = C7>,
        C7: Concat<T8::Output, Output = C8>,
        C8: Concat<T9::Output, Output = C9>,
        C9: Concat<T10::Output, Output = C10>,
        C10: Concat<T11::Output, Output = C11>,
        C11: Concat<T12::Output, Output = C12>,
        C12: Concat<T13::Output, Output = C13>,
        C13: Concat<T14::Output, Output = C14>,
        C14: Concat<T15::Output, Output = C15>,
        C15,
    > ParseImpl<I> for And<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15>
{
    type Output = C15;
    impl_parse!(parse, _await, |input: I| {
        let r = parse!(T0, input)?;
        macro_rules! impl_concat {
            ($r:ident, $($t:ty),*$(,)?) => {$(
                let $r = $r.concat(parse!($t, input)?);
            )*};
        }
        impl_concat!(r, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
        Ok(r)
    });
}

#[allow(clippy::type_complexity)]
pub struct Or<
    T0 = Never<NoOp>,
    T1 = Never<T0>,
    T2 = Never<T0>,
    T3 = Never<T0>,
    T4 = Never<T0>,
    T5 = Never<T0>,
    T6 = Never<T0>,
    T7 = Never<T0>,
    T8 = Never<T0>,
    T9 = Never<T0>,
    T10 = Never<T0>,
    T11 = Never<T0>,
    T12 = Never<T0>,
    T13 = Never<T0>,
    T14 = Never<T0>,
    T15 = Never<T0>,
>(
    PhantomData<(
        T0,
        T1,
        T2,
        T3,
        T4,
        T5,
        T6,
        T7,
        T8,
        T9,
        T10,
        T11,
        T12,
        T13,
        T14,
        T15,
    )>,
);

impl<
        T0: ParseImpl<I, Output = O>,
        T1: ParseImpl<I, Output = O>,
        T2: ParseImpl<I, Output = O>,
        T3: ParseImpl<I, Output = O>,
        T4: ParseImpl<I, Output = O>,
        T5: ParseImpl<I, Output = O>,
        T6: ParseImpl<I, Output = O>,
        T7: ParseImpl<I, Output = O>,
        T8: ParseImpl<I, Output = O>,
        T9: ParseImpl<I, Output = O>,
        T10: ParseImpl<I, Output = O>,
        T11: ParseImpl<I, Output = O>,
        T12: ParseImpl<I, Output = O>,
        T13: ParseImpl<I, Output = O>,
        T14: ParseImpl<I, Output = O>,
        T15: ParseImpl<I, Output = O>,
        I,
        O,
    > ParseImpl<I> for Or<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15>
{
    type Output = O;
    impl_parse!(parse, _await, |input: I| {
        macro_rules! impl_or {
            ($i:expr, $($t:ty),*$(,)?) => {$(
                let mut fork = $i.fork();
                match parse!($t, &mut fork) {
                    Ok(item) => {
                        fork.join();
                        return Ok(item);
                    }
                    Err(e @ Error::Hard(_)) => {
                        fork.join();
                        return Err(e);
                    }
                    _ => drop(fork),
                }
            )*};
        }
        impl_or!(input, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);

        Err(Error::Mismatch)
    });
}

pub struct Repeat<T, const MIN: usize = 0, const MAX: usize = { usize::MAX }>(PhantomData<T>);

impl<I, T: ParseImpl<I, Output = (O,)>, O, const MIN: usize, const MAX: usize> ParseImpl<I>
    for Repeat<T, MIN, MAX>
{
    type Output = (Vec<O>,);
    impl_parse!(parse, _await, |input: I| {
        let mut result = Vec::new();
        let mut le = None;
        for _ in 0..MAX {
            match parse!(T, input) {
                Ok((item,)) => result.push(item),
                Err(e @ Error::Hard(_)) => {
                    return Err(e);
                }
                Err(e) => {
                    le = Some(e);
                    break;
                }
            }
        }
        if result.len() < MIN {
            return Err(le.unwrap_or(Error::Mismatch));
        }
        Ok((result,))
    });
}

pub struct Optional<T>(PhantomData<T>);

impl<I, T: ParseImpl<I, Output = (O,)>, O> ParseImpl<I> for Optional<T> {
    type Output = (Option<O>,);
    impl_parse!(parse, _await, |input: I| {
        match parse!(T, input) {
            Ok((item,)) => Ok((Some(item),)),
            Err(e @ Error::Hard(_)) => Err(e),
            Err(_) => Ok((None,)),
        }
    });
}

pub struct Punctuated<T, P, const MIN: usize = 0, const MAX: usize = { usize::MAX }>(
    PhantomData<(T, P)>,
);

impl<
        I: core::fmt::Debug,
        T: ParseImpl<I, Output = (O,)>,
        O,
        P: ParseImpl<I, Output = (PO,)>,
        PO,
        const MIN: usize,
        const MAX: usize,
    > ParseImpl<I> for Punctuated<T, P, MIN, MAX>
{
    type Output = (Vec<O>, Vec<PO>);
    impl_parse!(parse, _await, |input: I| {
        let mut result = Vec::new();
        let mut puncts = Vec::new();
        let mut le = None;

        'matching: {
            match parse!(T, input) {
                Ok((item,)) => result.push(item),
                Err(e @ Error::Hard(_)) => {
                    return Err(e);
                }
                Err(e) => {
                    le = Some(e);
                    break 'matching;
                }
            }

            for _ in 1..MAX {
                let mut input = input.fork();
                match parse!(P, &mut input) {
                    Ok((item,)) => puncts.push(item),
                    Err(e @ Error::Hard(_)) => {
                        return Err(e);
                    }
                    Err(e) => {
                        le = Some(e);
                        break;
                    }
                }

                match parse!(T, &mut input) {
                    Ok((item,)) => {
                        result.push(item);
                        input.join();
                    }
                    Err(e @ Error::Hard(_)) => {
                        return Err(e);
                    }
                    Err(e) => {
                        puncts.pop();
                        le = Some(e);
                        break;
                    }
                }
            }
        }
        if result.len() < MIN {
            return Err(le.unwrap_or(Error::Mismatch));
        }
        Ok((result, puncts))
    });
}

pub struct Map<P, M>(PhantomData<(P, M)>);

pub struct TryMap<P, M>(PhantomData<(P, M)>);

impl<I, P: ParseImpl<I, Output = T>, M: Mapper<T, Output = U>, T, U> ParseImpl<I> for Map<P, M> {
    type Output = (U,);
    impl_parse!(parse, _await, |input: I| Ok((M::map(parse!(P, input)?),)));
}

impl<I, P: ParseImpl<I, Output = T>, M: Mapper<T, Output = Result<U>>, T, U> ParseImpl<I>
    for TryMap<P, M>
{
    type Output = (U,);
    impl_parse!(parse, _await, |input: I| Ok((M::map(parse!(P, input)?)?,)));
}

pub struct Expected<T, N>(PhantomData<(T, N)>);

impl<I, T: ParseImpl<I>, N: Const<Type = &'static str>> ParseImpl<I> for Expected<T, N> {
    type Output = T::Output;
    impl_parse!(parse, _await, |input: I| parse!(T, input).map_err(|e| {
        match e {
            Error::Mismatch => HardError::Incomplete {
                position: input.position(),
                name: N::VALUE,
            }
            .into(),
            Error::NamedMismatch(component_name) => HardError::NamedIncomplete {
                position: input.position(),
                name: N::VALUE,
                component_name,
            }
            .into(),
            _ => e,
        }
    }));
}

pub struct Name<T, N>(PhantomData<(T, N)>);

impl<I, T: ParseImpl<I>, N: Const<Type = &'static str>> ParseImpl<I> for Name<T, N> {
    type Output = T::Output;
    impl_parse!(parse, _await, |input: I| parse!(T, input).map_err(|e| {
        if let Error::Mismatch = e {
            Error::NamedMismatch(N::VALUE)
        } else {
            e
        }
    }));
}
