use crate::{
    parse::{macros::impl_parse, ParseImpl},
    Concat, Error, HardError, Result, SourceBase,
};
use alloc::vec::Vec;
use core::{borrow::Borrow, marker::PhantomData, ops::Range};

pub trait Const {
    type Type;
    const VALUE: Self::Type;
}

pub trait Predicate<T, A> {
    fn is(v: &T, a: &A) -> bool;
}

pub trait Mapper<T, A> {
    type Output;
    fn map(v: T, a: &A) -> Self::Output;
}

pub struct Define<T>(PhantomData<T>);

impl<T, U: Mapper<T, A>, A> Mapper<T, A> for Define<U> {
    type Output = U::Output;
    #[inline(always)]
    fn map(v: T, a: &A) -> Self::Output {
        U::map(v, a)
    }
}

impl<T, U: Predicate<T, A>, A> Predicate<T, A> for Define<U> {
    #[inline(always)]
    fn is(v: &T, arg: &A) -> bool {
        U::is(v, arg)
    }
}

impl<T: Const> Const for Define<T> {
    type Type = T::Type;
    const VALUE: Self::Type = T::VALUE;
}

impl<I, U: ParseImpl<I, A>, A> ParseImpl<I, A> for Define<U> {
    type Output = U::Output;

    #[inline(always)]
    fn parse<S: crate::Source<Item = I>>(input: &mut S, arg: &A) -> Result<Self::Output> {
        U::parse(input, arg)
    }

    #[cfg(feature = "async")]
    #[inline(always)]
    async fn parse_async<S: crate::AsyncSource<Item = I>>(
        input: &mut S,
        arg: &A,
    ) -> Result<Self::Output> {
        U::parse_async(input, arg).await
    }
}

pub struct Is<P>(PhantomData<P>);

impl<I: Clone, P: Predicate<I, A>, A> ParseImpl<I, A> for Is<P> {
    type Output = (I,);
    impl_parse!(parse, _await, |input: I, arg: A| {
        if let Some(item) = _await!(input.read(1))?.first() {
            if P::is(item, arg) {
                let item = item.clone();
                input.consume(1);
                return Ok((item,));
            }
        }

        Err(Error::Mismatch)
    });
}

impl<I, P: Predicate<I, A>, A> Predicate<I, A> for Is<P> {
    #[inline(always)]
    fn is(v: &I, arg: &A) -> bool {
        P::is(v, arg)
    }
}

pub type A<C> = AnyOf<AsMany<C>>;

pub struct Not<T>(PhantomData<T>);

impl<I, T: Predicate<I, A>, A> Predicate<I, A> for Not<T> {
    #[inline(always)]
    fn is(v: &I, arg: &A) -> bool {
        !T::is(v, arg)
    }
}

impl<I: Clone, T: Predicate<I, A>, A> ParseImpl<I, A> for Not<T> {
    type Output = <Is<Self> as ParseImpl<I, A>>::Output;
    impl_parse!(parse, _await, |input: I, arg: A| parse!(
        Is::<Self>, input, arg
    ));
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

impl<C: Const<Type = G>, G: IntoIterator<Item = T>, T: Borrow<U>, U: PartialEq, A> Predicate<U, A>
    for AnyOf<C>
{
    #[inline(always)]
    fn is(v: &U, _: &A) -> bool {
        C::VALUE.into_iter().any(|x| v == x.borrow())
    }
}

impl<C: Const<Type = G>, G: IntoIterator<Item = T>, T: Borrow<U>, U: PartialEq + Clone, A>
    ParseImpl<U, A> for AnyOf<C>
{
    type Output = <Is<Self> as ParseImpl<U, A>>::Output;
    impl_parse!(parse, _await, |input: U, _arg: A| parse!(
        Is::<Self>,
        input,
        &()
    ));
}

pub struct Seq<C>(PhantomData<C>);

impl<C: Const<Type = G>, G: IntoIterator<Item = T>, T: Borrow<I>, I: PartialEq + Clone, A>
    ParseImpl<I, A> for Seq<C>
{
    type Output = (Vec<I>,);
    impl_parse!(parse, _await, |input: I, _arg: A| {
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

pub struct NoOp<T = ()>(PhantomData<T>);

pub struct Never<T>(PhantomData<T>);

impl<I, T: ParseImpl<I, A>, A> ParseImpl<I, A> for Discard<T> {
    type Output = ();
    impl_parse!(parse, _await, |input: I, arg: A| {
        parse!(T, input, arg)?;
        Ok(())
    });
}

impl<I, A, T: Default> ParseImpl<I, A> for NoOp<T> {
    type Output = T;
    impl_parse!(parse, _await, |__: I, _arg: A| Ok(T::default()));
}

impl<T, A> Mapper<(T,), A> for NoOp {
    type Output = T;
    #[inline(always)]
    fn map((v,): (T,), _: &A) -> Self::Output {
        v
    }
}

impl<I, O, T: ParseImpl<I, A, Output = O>, A> ParseImpl<I, A> for Never<T> {
    type Output = O;
    impl_parse!(parse, _await, |__: I, _arg: A| Err(Error::Mismatch));
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
    T16 = NoOp,
    T17 = NoOp,
    T18 = NoOp,
    T19 = NoOp,
    T20 = NoOp,
    T21 = NoOp,
    T22 = NoOp,
    T23 = NoOp,
    T24 = NoOp,
    T25 = NoOp,
    T26 = NoOp,
    T27 = NoOp,
    T28 = NoOp,
    T29 = NoOp,
    T30 = NoOp,
    T31 = NoOp,
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
        T16,
        T17,
        T18,
        T19,
        T20,
        T21,
        T22,
        T23,
        T24,
        T25,
        T26,
        T27,
        T28,
        T29,
        T30,
        T31,
    )>,
);

impl<
        I,
        T0: ParseImpl<I, A, Output = C0>,
        T1: ParseImpl<I, A>,
        T2: ParseImpl<I, A>,
        T3: ParseImpl<I, A>,
        T4: ParseImpl<I, A>,
        T5: ParseImpl<I, A>,
        T6: ParseImpl<I, A>,
        T7: ParseImpl<I, A>,
        T8: ParseImpl<I, A>,
        T9: ParseImpl<I, A>,
        T10: ParseImpl<I, A>,
        T11: ParseImpl<I, A>,
        T12: ParseImpl<I, A>,
        T13: ParseImpl<I, A>,
        T14: ParseImpl<I, A>,
        T15: ParseImpl<I, A>,
        T16: ParseImpl<I, A>,
        T17: ParseImpl<I, A>,
        T18: ParseImpl<I, A>,
        T19: ParseImpl<I, A>,
        T20: ParseImpl<I, A>,
        T21: ParseImpl<I, A>,
        T22: ParseImpl<I, A>,
        T23: ParseImpl<I, A>,
        T24: ParseImpl<I, A>,
        T25: ParseImpl<I, A>,
        T26: ParseImpl<I, A>,
        T27: ParseImpl<I, A>,
        T28: ParseImpl<I, A>,
        T29: ParseImpl<I, A>,
        T30: ParseImpl<I, A>,
        T31: ParseImpl<I, A>,
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
        C15: Concat<T16::Output, Output = C16>,
        C16: Concat<T17::Output, Output = C17>,
        C17: Concat<T18::Output, Output = C18>,
        C18: Concat<T19::Output, Output = C19>,
        C19: Concat<T20::Output, Output = C20>,
        C20: Concat<T21::Output, Output = C21>,
        C21: Concat<T22::Output, Output = C22>,
        C22: Concat<T23::Output, Output = C23>,
        C23: Concat<T24::Output, Output = C24>,
        C24: Concat<T25::Output, Output = C25>,
        C25: Concat<T26::Output, Output = C26>,
        C26: Concat<T27::Output, Output = C27>,
        C27: Concat<T28::Output, Output = C28>,
        C28: Concat<T29::Output, Output = C29>,
        C29: Concat<T30::Output, Output = C30>,
        C30: Concat<T31::Output, Output = C31>,
        C31,
        A,
    > ParseImpl<I, A>
    for And<
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
        T16,
        T17,
        T18,
        T19,
        T20,
        T21,
        T22,
        T23,
        T24,
        T25,
        T26,
        T27,
        T28,
        T29,
        T30,
        T31,
    >
{
    type Output = C31;
    impl_parse!(parse, _await, |input: I, arg: A| {
        let r = parse!(T0, input, arg)?;
        macro_rules! impl_concat {
            ($r:ident, $($t:ty),*$(,)?) => {$(
                let $r = $r.concat(parse!($t, input, arg)?);
            )*};
        }
        impl_concat!(
            r, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18,
            T19, T20, T21, T22, T23, T24, T25, T26, T27, T28, T29, T30,
        );
        let r = r.concat(parse!(T31, input, arg)?);
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
    T16 = Never<T0>,
    T17 = Never<T0>,
    T18 = Never<T0>,
    T19 = Never<T0>,
    T20 = Never<T0>,
    T21 = Never<T0>,
    T22 = Never<T0>,
    T23 = Never<T0>,
    T24 = Never<T0>,
    T25 = Never<T0>,
    T26 = Never<T0>,
    T27 = Never<T0>,
    T28 = Never<T0>,
    T29 = Never<T0>,
    T30 = Never<T0>,
    T31 = Never<T0>,
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
        T16,
        T17,
        T18,
        T19,
        T20,
        T21,
        T22,
        T23,
        T24,
        T25,
        T26,
        T27,
        T28,
        T29,
        T30,
        T31,
    )>,
);

impl<
        T0: ParseImpl<I, A, Output = O>,
        T1: ParseImpl<I, A, Output = O>,
        T2: ParseImpl<I, A, Output = O>,
        T3: ParseImpl<I, A, Output = O>,
        T4: ParseImpl<I, A, Output = O>,
        T5: ParseImpl<I, A, Output = O>,
        T6: ParseImpl<I, A, Output = O>,
        T7: ParseImpl<I, A, Output = O>,
        T8: ParseImpl<I, A, Output = O>,
        T9: ParseImpl<I, A, Output = O>,
        T10: ParseImpl<I, A, Output = O>,
        T11: ParseImpl<I, A, Output = O>,
        T12: ParseImpl<I, A, Output = O>,
        T13: ParseImpl<I, A, Output = O>,
        T14: ParseImpl<I, A, Output = O>,
        T15: ParseImpl<I, A, Output = O>,
        T16: ParseImpl<I, A, Output = O>,
        T17: ParseImpl<I, A, Output = O>,
        T18: ParseImpl<I, A, Output = O>,
        T19: ParseImpl<I, A, Output = O>,
        T20: ParseImpl<I, A, Output = O>,
        T21: ParseImpl<I, A, Output = O>,
        T22: ParseImpl<I, A, Output = O>,
        T23: ParseImpl<I, A, Output = O>,
        T24: ParseImpl<I, A, Output = O>,
        T25: ParseImpl<I, A, Output = O>,
        T26: ParseImpl<I, A, Output = O>,
        T27: ParseImpl<I, A, Output = O>,
        T28: ParseImpl<I, A, Output = O>,
        T29: ParseImpl<I, A, Output = O>,
        T30: ParseImpl<I, A, Output = O>,
        T31: ParseImpl<I, A, Output = O>,
        I,
        O,
        A,
    > ParseImpl<I, A>
    for Or<
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
        T16,
        T17,
        T18,
        T19,
        T20,
        T21,
        T22,
        T23,
        T24,
        T25,
        T26,
        T27,
        T28,
        T29,
        T30,
        T31,
    >
{
    type Output = O;
    impl_parse!(parse, _await, |input: I, arg: A| {
        macro_rules! impl_or {
            ($i:expr, $($t:ty),*$(,)?) => {$(
                let mut fork = $i.fork();
                match parse!($t, &mut fork, arg) {
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
        impl_or!(
            input, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17,
            T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28, T29, T30,
        );

        let mut fork = input.fork();
        match parse!(T31, &mut fork, arg) {
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

        Err(Error::Mismatch)
    });
}

pub struct Repeat<T, const MIN: usize = 0, const MAX: usize = { usize::MAX }>(PhantomData<T>);

impl<I, T: ParseImpl<I, A, Output = (O,)>, O, A, const MIN: usize, const MAX: usize> ParseImpl<I, A>
    for Repeat<T, MIN, MAX>
{
    type Output = (Vec<O>,);
    impl_parse!(parse, _await, |input: I, arg: A| {
        let mut result = Vec::new();
        let mut le = None;
        for _ in 0..MAX {
            match parse!(T, input, arg) {
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

impl<I, T: ParseImpl<I, A, Output = (O,)>, O, A> ParseImpl<I, A> for Optional<T> {
    type Output = (Option<O>,);
    impl_parse!(parse, _await, |input: I, arg: A| {
        match parse!(T, input, arg) {
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
        T: ParseImpl<I, A, Output = (O,)>,
        O,
        P: ParseImpl<I, A, Output = (PO,)>,
        PO,
        A,
        const MIN: usize,
        const MAX: usize,
    > ParseImpl<I, A> for Punctuated<T, P, MIN, MAX>
{
    type Output = (Vec<O>, Vec<PO>);
    impl_parse!(parse, _await, |input: I, arg: A| {
        let mut result = Vec::new();
        let mut puncts = Vec::new();
        let mut le = None;

        'matching: {
            match parse!(T, input, arg) {
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
                match parse!(P, &mut input, arg) {
                    Ok((item,)) => puncts.push(item),
                    Err(e @ Error::Hard(_)) => {
                        return Err(e);
                    }
                    Err(e) => {
                        le = Some(e);
                        break;
                    }
                }

                match parse!(T, &mut input, arg) {
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

pub struct Map<P, M = NoOp>(PhantomData<(P, M)>);

pub struct TryMap<P, M>(PhantomData<(P, M)>);

pub struct IsMap<P, M>(PhantomData<(P, M)>);

impl<I, P: ParseImpl<I, A, Output = T>, M: Mapper<T, A, Output = U>, T, U, A> ParseImpl<I, A>
    for Map<P, M>
{
    type Output = (U,);
    impl_parse!(parse, _await, |input: I, arg: A| Ok((M::map(
        parse!(P, input, arg)?,
        arg
    ),)));
}

impl<I, P: ParseImpl<I, A, Output = T>, M: Mapper<T, A, Output = Result<U>>, T, U, A>
    ParseImpl<I, A> for TryMap<P, M>
{
    type Output = (U,);
    impl_parse!(parse, _await, |input: I, arg: A| Ok((M::map(
        parse!(P, input, arg)?,
        arg
    )?,)));
}

impl<I, P: ParseImpl<I, A, Output = T>, M: Mapper<T, A, Output = Option<U>>, T, U, A>
    ParseImpl<I, A> for IsMap<P, M>
{
    type Output = (U,);
    impl_parse!(parse, _await, |input: I, arg: A| Ok((M::map(
        parse!(P, input, arg)?,
        arg
    )
    .ok_or(Error::Mismatch)?,)));
}

pub struct Expected<T, N>(PhantomData<(T, N)>);

impl<I, T: ParseImpl<I, A>, N: Const<Type = &'static str>, A> ParseImpl<I, A> for Expected<T, N> {
    type Output = T::Output;
    impl_parse!(parse, _await, |input: I, arg: A| parse!(T, input, arg)
        .map_err(|e| {
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

impl<I, T: ParseImpl<I, A>, N: Const<Type = &'static str>, A> ParseImpl<I, A> for Name<T, N> {
    type Output = T::Output;
    impl_parse!(parse, _await, |input: I, arg: A| parse!(T, input, arg)
        .map_err(|e| {
            if let Error::Mismatch = e {
                Error::NamedMismatch(N::VALUE)
            } else {
                e
            }
        }));
}

pub struct Peek<T>(PhantomData<T>);

impl<T: ParseImpl<I, A>, I, A> ParseImpl<I, A> for Peek<T> {
    type Output = T::Output;
    impl_parse!(parse, _await, |input: I, arg: A| parse!(
        T,
        &mut input.fork(),
        arg
    ));
}

pub struct AndWithArg<T0 = NoOp, T1 = NoOp>(PhantomData<(T0, T1)>);

impl<I, T0: ParseImpl<I, A0, Output = (A1,)>, T1: ParseImpl<I, A1>, A0, A1: Clone> ParseImpl<I, A0>
    for AndWithArg<T0, T1>
where
    (A1,): Concat<T1::Output>,
{
    type Output = <(A1,) as Concat<T1::Output>>::Output;

    impl_parse!(parse, _await, |input: I, arg: A0| {
        let r = parse!(T0, input, arg)?;
        Ok(r.clone().concat(parse!(T1, input, &r.0)?))
    });
}

pub struct MapRange<T, M = ConcatArg>(PhantomData<(T, M)>);

impl<I, A, T: ParseImpl<I, A>, M: Mapper<T::Output, Range<usize>>> ParseImpl<I, A>
    for MapRange<T, M>
{
    type Output = (M::Output,);
    impl_parse!(parse, _await, |input: I, arg: A| {
        let start = input.position();
        let result = parse!(T, input, arg)?;
        let end = input.position();
        Ok((M::map(result, &(start..end)),))
    });
}

pub struct ConcatArg;

impl<T: Concat<(A,)>, A: Clone> Mapper<T, A> for ConcatArg {
    type Output = T::Output;
    #[inline(always)]
    fn map(v: T, a: &A) -> Self::Output {
        v.concat((a.clone(),))
    }
}

pub struct Start;

impl<I, A> ParseImpl<I, A> for Start {
    type Output = ();
    impl_parse!(parse, _await, |input: I, _arg: A| {
        if input.position() == 0 {
            Ok(())
        } else {
            Err(Error::Mismatch)
        }
    });
}

pub struct End;

impl<I, A> ParseImpl<I, A> for End {
    type Output = ();
    impl_parse!(parse, _await, |input: I, _arg: A| {
        if _await!(input.read(1))?.is_empty() {
            Ok(())
        } else {
            Err(Error::Mismatch)
        }
    });
}
