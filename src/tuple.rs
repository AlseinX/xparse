pub trait Concat<Rhs> {
    type Output;
    fn concat(self, r: Rhs) -> Self::Output;
}

macro_rules! impl_tuples {
    (($($l:ident),*$(,)?),($($r:ident),*$(,)?)) => {
        impl<$($l,)*$($r),*> Concat<($($r,)*)> for ($($l,)*) {
            type Output = ($($l,)*$($r,)*);
            #[allow(non_snake_case)]
            #[allow(clippy::unused_unit)]
            #[inline(always)]
            fn concat(self, ($($r,)*): ($($r,)*)) -> Self::Output {
                let ($($l,)*) = self;
                ($($l,)*$($r,)*)
            }
        }
    };
    (($($l:ident),*$(,)?)) => {
        impl_tuples!(($($l),*),());
        impl_tuples!(($($l),*),(R0));
        impl_tuples!(($($l),*),(R0, R1));
        impl_tuples!(($($l),*),(R0, R1, R2));
        impl_tuples!(($($l),*),(R0, R1, R2, R3));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15));
    }
}

impl_tuples!(());
impl_tuples!((L0));
impl_tuples!((L0, L1));
impl_tuples!((L0, L1, L2));
impl_tuples!((L0, L1, L2, L3));
impl_tuples!((L0, L1, L2, L3, L4));
impl_tuples!((L0, L1, L2, L3, L4, L5));
impl_tuples!((L0, L1, L2, L3, L4, L5, L6));
impl_tuples!((L0, L1, L2, L3, L4, L5, L6, L7));
impl_tuples!((L0, L1, L2, L3, L4, L5, L6, L7, L8));
impl_tuples!((L0, L1, L2, L3, L4, L5, L6, L7, L8, L9));
impl_tuples!((L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10));
impl_tuples!((L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11));
impl_tuples!((L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11, L12));
impl_tuples!((L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11, L12, L13));
impl_tuples!((L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11, L12, L13, L14));
impl_tuples!((L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11, L12, L13, L14, L15));
