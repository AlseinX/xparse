use crate::{Result, Source};
#[cfg(feature = "macros")]
pub use xparse_macros::parser;
#[cfg(feature = "async")]
use {crate::AsyncSource, core::future::Future};

pub trait Parse<I> {
    type Output;
    fn parse<S: Source<Item = I>>(input: &mut S) -> Result<Self::Output>;

    #[cfg(feature = "async")]
    fn parse_async<S: AsyncSource<Item = I>>(
        input: &mut S,
    ) -> impl Future<Output = Result<Self::Output>>;
}

pub trait ParseImpl<I, A> {
    type Output;
    fn parse<S: Source<Item = I>>(input: &mut S, arg: A) -> Result<Self::Output>;

    #[cfg(feature = "async")]
    fn parse_async<S: AsyncSource<Item = I>>(
        input: &mut S,
        arg: A,
    ) -> impl Future<Output = Result<Self::Output>>;
}

impl<I, T: ParseImpl<I, ()>, O> Parse<I> for T
where
    T::Output: SingleTuple<Item = O>,
{
    type Output = O;
    #[inline(always)]
    fn parse<S: Source<Item = I>>(input: &mut S) -> Result<Self::Output> {
        Ok(<T as ParseImpl<I, ()>>::parse(input, ())?.into_item())
    }

    #[cfg(feature = "async")]
    #[inline(always)]
    async fn parse_async<S: AsyncSource<Item = I>>(input: &mut S) -> Result<Self::Output> {
        Ok(<T as ParseImpl<I, ()>>::parse_async(input, ())
            .await?
            .into_item())
    }
}

trait SingleTuple {
    type Item;
    fn into_item(self) -> Self::Item;
}

impl<T> SingleTuple for (T,) {
    type Item = T;
    #[inline(always)]
    fn into_item(self) -> Self::Item {
        self.0
    }
}

pub(crate) mod macros {
    macro_rules! impl_parse {
        ($pa:ident,$aw:ident,|$s:ident:$i:ty,$av:ident:$at:ty|$b:expr) => {
            #[inline(always)]
            fn parse<S: $crate::Source<Item = $i>>($s: &mut S, $av: $at) -> Result<Self::Output> {
                #[allow(unused_imports)]
                use $crate::parse::macros::no_await as $aw;
                #[allow(unused_imports)]
                use $crate::parse::macros::parse_sync as $pa;
                $b
            }

            #[cfg(feature = "async")]
            #[inline(always)]
            async fn parse_async<S: $crate::AsyncSource<Item = $i>>(
                $s: &mut S,
                $av: $at,
            ) -> Result<Self::Output> {
                #[allow(unused_imports)]
                use $crate::parse::macros::has_await as $aw;
                #[allow(unused_imports)]
                use $crate::parse::macros::parse_async as $pa;
                $b
            }
        };
    }

    #[cfg(feature = "async")]
    macro_rules! has_await {
        ($v:expr) => {
            $v.await
        };
    }

    macro_rules! no_await {
        ($v:expr) => {
            $v
        };
    }

    #[cfg(feature = "async")]
    macro_rules! parse_async {
        ($t:ty,$s:expr,$a:expr) => {
            <$t as $crate::parse::ParseImpl<_, _>>::parse_async($s, $a).await
        };
    }

    macro_rules! parse_sync {
        ($t:ty,$s:expr,$a:expr) => {
            <$t as $crate::parse::ParseImpl<_, _>>::parse($s, $a)
        };
    }

    pub(crate) use {impl_parse, no_await, parse_sync};

    #[cfg(feature = "async")]
    pub(crate) use {has_await, parse_async};
}
