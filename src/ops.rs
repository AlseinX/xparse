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

pub trait Predicate<T, A> {
    fn is(v: &T, a: A) -> bool;
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

impl<T, U: Predicate<T, A>, A> Predicate<T, A> for Define<U> {
    #[inline(always)]
    fn is(v: &T, arg: A) -> bool {
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
    fn parse<S: crate::Source<Item = I>>(input: &mut S, arg: A) -> Result<Self::Output> {
        U::parse(input, arg)
    }

    #[cfg(feature = "async")]
    #[inline(always)]
    async fn parse_async<S: crate::AsyncSource<Item = I>>(
        input: &mut S,
        arg: A,
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
    fn is(v: &I, arg: A) -> bool {
        P::is(v, arg)
    }
}

pub type A<C> = AnyOf<AsMany<C>>;

pub struct Not<T>(PhantomData<T>);

impl<I, T: Predicate<I, A>, A> Predicate<I, A> for Not<T> {
    #[inline(always)]
    fn is(v: &I, arg: A) -> bool {
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
    fn is(v: &U, _: A) -> bool {
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
        ()
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
        A: Clone,
    > ParseImpl<I, A>
    for And<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15>
{
    type Output = C15;
    impl_parse!(parse, _await, |input: I, arg: A| {
        let r = parse!(T0, input, arg.clone())?;
        macro_rules! impl_concat {
            ($r:ident, $($t:ty),*$(,)?) => {$(
                let $r = $r.concat(parse!($t, input, arg.clone())?);
            )*};
        }
        impl_concat!(r, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
        let r = r.concat(parse!(T15, input, arg)?);
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
        I,
        O,
        A: Clone,
    > ParseImpl<I, A> for Or<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15>
{
    type Output = O;
    impl_parse!(parse, _await, |input: I, arg: A| {
        macro_rules! impl_or {
            ($i:expr, $($t:ty),*$(,)?) => {$(
                let mut fork = $i.fork();
                match parse!($t, &mut fork, arg.clone()) {
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
        impl_or!(input, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);

        let mut fork = input.fork();
        match parse!(T15, &mut fork, arg) {
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

impl<I, T: ParseImpl<I, A, Output = (O,)>, O, A: Clone, const MIN: usize, const MAX: usize>
    ParseImpl<I, A> for Repeat<T, MIN, MAX>
{
    type Output = (Vec<O>,);
    impl_parse!(parse, _await, |input: I, arg: A| {
        let mut result = Vec::new();
        let mut le = None;
        for _ in 0..MAX {
            match parse!(T, input, arg.clone()) {
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
        A: Clone,
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
            match parse!(T, input, arg.clone()) {
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
                match parse!(P, &mut input, arg.clone()) {
                    Ok((item,)) => puncts.push(item),
                    Err(e @ Error::Hard(_)) => {
                        return Err(e);
                    }
                    Err(e) => {
                        le = Some(e);
                        break;
                    }
                }

                match parse!(T, &mut input, arg.clone()) {
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

impl<I, P: ParseImpl<I, A, Output = T>, M: Mapper<T, Output = U>, T, U, A> ParseImpl<I, A>
    for Map<P, M>
{
    type Output = (U,);
    impl_parse!(parse, _await, |input: I, arg: A| Ok((M::map(parse!(
        P, input, arg
    )?),)));
}

impl<I, P: ParseImpl<I, A, Output = T>, M: Mapper<T, Output = Result<U>>, T, U, A> ParseImpl<I, A>
    for TryMap<P, M>
{
    type Output = (U,);
    impl_parse!(parse, _await, |input: I, arg: A| Ok((M::map(parse!(
        P, input, arg
    )?)?,)));
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

pub struct ArgOut;

impl<I, A> ParseImpl<I, A> for ArgOut {
    type Output = (A,);
    impl_parse!(parse, _await, |_input: I, arg: A| Ok((arg,)));
}

#[allow(clippy::type_complexity)]
pub struct Chain<
    T0 = NoOp<((),)>,
    T1 = ArgOut,
    T2 = ArgOut,
    T3 = ArgOut,
    T4 = ArgOut,
    T5 = ArgOut,
    T6 = ArgOut,
    T7 = ArgOut,
    T8 = ArgOut,
    T9 = ArgOut,
    T10 = ArgOut,
    T11 = ArgOut,
    T12 = ArgOut,
    T13 = ArgOut,
    T14 = ArgOut,
    T15 = ArgOut,
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
        T0: ParseImpl<I, A0, Output = (A1,)>,
        T1: ParseImpl<I, A1, Output = (A2,)>,
        T2: ParseImpl<I, A2, Output = (A3,)>,
        T3: ParseImpl<I, A3, Output = (A4,)>,
        T4: ParseImpl<I, A4, Output = (A5,)>,
        T5: ParseImpl<I, A5, Output = (A6,)>,
        T6: ParseImpl<I, A6, Output = (A7,)>,
        T7: ParseImpl<I, A7, Output = (A8,)>,
        T8: ParseImpl<I, A8, Output = (A9,)>,
        T9: ParseImpl<I, A9, Output = (A10,)>,
        T10: ParseImpl<I, A10, Output = (A11,)>,
        T11: ParseImpl<I, A11, Output = (A12,)>,
        T12: ParseImpl<I, A12, Output = (A13,)>,
        T13: ParseImpl<I, A13, Output = (A14,)>,
        T14: ParseImpl<I, A14, Output = (A15,)>,
        T15: ParseImpl<I, A15, Output = (A16,)>,
        A0,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        A8,
        A9,
        A10,
        A11,
        A12,
        A13,
        A14,
        A15,
        A16,
    > ParseImpl<I, A0>
    for Chain<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15>
{
    type Output = (A16,);
    impl_parse!(parse, _await, |input: I, arg: A0| {
        macro_rules! impl_chain {
            ($r:ident,$($t:ty),*$(,)?) => {$(
                let ($r,) = parse!($t, input, $r)?;
            )*};
        }
        let r = arg;
        impl_chain!(r, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
        Ok((r,))
    });
}
