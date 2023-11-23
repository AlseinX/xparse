#[cfg(feature = "async")]
use {
    crate::Error,
    alloc::collections::VecDeque,
    core::{
        future::{poll_fn, Future},
        pin::Pin,
        task::{Context, Poll},
    },
    futures_core::Stream,
};

use crate::Result;

#[cfg(not(feature = "async"))]
pub fn from_slice<T>(slice: &[T]) -> impl Source<Item = T> + '_ {
    OwnedSource {
        position: 0,
        r#impl: slice,
    }
}

#[cfg(feature = "async")]
pub fn from_slice<T>(slice: &[T]) -> impl Source<Item = T> + AsyncSource<Item = T> + '_ {
    OwnedSource {
        position: 0,
        r#impl: slice,
    }
}

#[cfg(feature = "async")]
pub fn form_stream<S: Stream + Unpin>(stream: S) -> impl AsyncSource<Item = S::Item> {
    OwnedSource {
        position: 0,
        r#impl: BufferedStream {
            buffer: VecDeque::new(),
            stream: AsResult(stream),
        },
    }
}

#[cfg(feature = "async")]
struct AsResult<T>(T);

#[cfg(feature = "async")]
impl<T: Stream> Stream for AsResult<T> {
    type Item = Result<T::Item>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().0) }
            .poll_next(cx)
            .map(|x| x.map(Ok))
    }
}

#[cfg(feature = "async")]
pub fn form_try_stream<S: Stream<Item = Result<I, E>> + Unpin, I, E: Into<Error>>(
    stream: S,
) -> impl AsyncSource<Item = I> {
    OwnedSource {
        position: 0,
        r#impl: BufferedStream {
            buffer: VecDeque::new(),
            stream,
        },
    }
}

pub trait SourceBase {
    type Item;
    fn consume(&mut self, len: usize);
    fn position(&self) -> usize;
    fn join(self);
}

pub trait Source: SourceBase {
    fn fork(&mut self) -> impl Source<Item = Self::Item>;
    fn read(&mut self, len: usize) -> Result<&[Self::Item]>;
}

#[cfg(feature = "async")]
pub trait AsyncSource: SourceBase {
    fn fork(&mut self) -> impl AsyncSource<Item = Self::Item>;
    fn read(&mut self, len: usize) -> impl Future<Output = Result<&[Self::Item]>>;
}

struct OwnedSource<T> {
    r#impl: T,
    position: usize,
}

impl<T: SourceImplBase> SourceBase for OwnedSource<T> {
    type Item = T::Item;

    #[inline(always)]
    fn consume(&mut self, len: usize) {
        debug_assert!(len <= self.r#impl.available());
        self.position += len;
        self.r#impl.consume(len);
    }

    #[inline(always)]
    fn position(&self) -> usize {
        self.position
    }

    #[inline(always)]
    fn join(self) {}
}

impl<T: SourceImpl> Source for OwnedSource<T> {
    #[inline(always)]
    fn fork(&mut self) -> impl Source<Item = Self::Item> {
        SourceRef {
            target: self,
            parent: None,
            offset: 0,
        }
    }

    #[inline(always)]
    fn read(&mut self, len: usize) -> Result<&[Self::Item]> {
        self.r#impl.read(len)
    }
}

#[cfg(feature = "async")]
impl<T: AsyncSourceImpl> AsyncSource for OwnedSource<T> {
    #[inline(always)]
    fn fork(&mut self) -> impl AsyncSource<Item = Self::Item> {
        SourceRef {
            target: self,
            parent: None,
            offset: 0,
        }
    }

    #[inline(always)]
    fn read(&mut self, len: usize) -> impl Future<Output = Result<&[Self::Item]>> {
        self.r#impl.read(len)
    }
}

struct SourceRef<'a, T> {
    target: &'a mut OwnedSource<T>,
    parent: Option<&'a mut usize>,
    offset: usize,
}

impl<T: SourceImplBase> SourceBase for SourceRef<'_, T> {
    type Item = T::Item;

    #[inline(always)]
    fn consume(&mut self, len: usize) {
        debug_assert!(self.offset + len <= self.target.r#impl.available());
        self.offset += len;
    }

    #[inline(always)]
    fn position(&self) -> usize {
        self.target.position + self.offset
    }

    #[inline(always)]
    fn join(self) {
        if let Some(parent) = self.parent {
            *parent = self.offset
        } else {
            self.target.consume(self.offset);
        }
    }
}

impl<T: SourceImpl> Source for SourceRef<'_, T> {
    #[inline(always)]
    fn fork(&mut self) -> impl Source<Item = Self::Item> {
        SourceRef {
            target: self.target,
            offset: self.offset,
            parent: Some(&mut self.offset),
        }
    }

    #[inline(always)]
    fn read(&mut self, len: usize) -> Result<&[Self::Item]> {
        Ok(&self.target.r#impl.read(self.offset + len)?[self.offset..])
    }
}

#[cfg(feature = "async")]
impl<T: AsyncSourceImpl> AsyncSource for SourceRef<'_, T> {
    #[inline(always)]
    fn fork(&mut self) -> impl AsyncSource<Item = Self::Item> {
        SourceRef {
            target: self.target,
            offset: self.offset,
            parent: Some(&mut self.offset),
        }
    }

    #[inline(always)]
    async fn read(&mut self, len: usize) -> Result<&[Self::Item]> {
        Ok(&self.target.r#impl.read(self.offset + len).await?[self.offset..])
    }
}

trait SourceImplBase {
    type Item;
    fn consume(&mut self, len: usize);
    fn available(&self) -> usize;
}

trait SourceImpl: SourceImplBase {
    fn read(&mut self, len: usize) -> Result<&[Self::Item]>;
}

#[cfg(feature = "async")]
trait AsyncSourceImpl: SourceImplBase {
    fn read(&mut self, len: usize) -> impl Future<Output = Result<&[Self::Item]>>;
}

#[cfg(feature = "async")]
impl<T: SourceImpl + ?Sized> AsyncSourceImpl for T {
    #[inline(always)]
    async fn read(&mut self, len: usize) -> Result<&[Self::Item]> {
        SourceImpl::read(self, len)
    }
}

#[cfg(feature = "async")]
struct BufferedStream<S: Stream<Item = Result<I, E>> + ?Sized, I, E: Into<Error>> {
    buffer: VecDeque<I>,
    stream: S,
}

#[cfg(feature = "async")]
impl<S: Stream<Item = Result<I, E>> + Unpin + ?Sized, I, E: Into<Error>> SourceImplBase
    for BufferedStream<S, I, E>
{
    type Item = I;

    #[inline]
    fn consume(&mut self, len: usize) {
        debug_assert!(
            self.buffer.len() >= len,
            "consume failed, the current buffer length is lower than {len}"
        );
        for _ in 0..len {
            self.buffer.pop_front();
        }
    }

    #[inline]
    fn available(&self) -> usize {
        self.buffer.len()
    }
}

#[cfg(feature = "async")]
impl<S: Stream<Item = Result<I, E>> + Unpin + ?Sized, I, E: Into<Error>> AsyncSourceImpl
    for BufferedStream<S, I, E>
{
    fn read(&mut self, len: usize) -> impl Future<Output = Result<&[I]>> {
        #[cfg(debug_assertions)]
        let mut is_complete = false;
        poll_fn(move |cx| {
            #[cfg(debug_assertions)]
            if is_complete {
                unreachable!("read future polled after completion")
            }

            while self.buffer.len() < len {
                match Pin::new(&mut self.stream).poll_next(cx) {
                    Poll::Ready(Some(Ok(item))) => self.buffer.push_back(item),
                    Poll::Ready(Some(Err(e))) => return Poll::Ready(Err(e.into())),
                    Poll::Ready(None) => {
                        #[cfg(debug_assertions)]
                        {
                            is_complete = true;
                        }
                        return Poll::Ready(Ok(unsafe {
                            &*(self.buffer.make_contiguous() as *const _)
                        }));
                    }
                    Poll::Pending => return Poll::Pending,
                };
            }

            #[cfg(debug_assertions)]
            {
                is_complete = true;
            }
            Poll::Ready(Ok(unsafe {
                &*((&self.buffer.make_contiguous()[..len]) as *const _)
            }))
        })
    }
}

impl<T> SourceImplBase for &[T] {
    type Item = T;

    fn consume(&mut self, len: usize) {
        *self = &self[len..]
    }

    fn available(&self) -> usize {
        self.len()
    }
}

impl<T> SourceImpl for &[T] {
    fn read(&mut self, len: usize) -> Result<&[Self::Item]> {
        Ok(&self[..len.min(self.len())])
    }
}

#[cfg(test)]
mod test {
    use super::{from_slice, Source, SourceBase};

    #[test]
    fn read_test() {
        let mut source = from_slice(b"01234567");
        let mut fork0 = source.fork();
        assert_eq!(fork0.read(9).unwrap(), b"01234567");
        let mut fork1 = fork0.fork();
        fork1.consume(3);
        fork1.join();
        let mut fork = fork0.fork();
        assert_eq!(fork.read(3).unwrap(), b"345");
        fork.consume(4);
        drop(fork);
        fork0.consume(3);
        assert_eq!(fork0.read(3).unwrap(), b"67");
        fork0.join();
        assert_eq!(source.read(3).unwrap(), b"67");
        source.consume(2);
        assert_eq!(source.read(5).unwrap(), []);
    }
}
