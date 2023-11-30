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
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16, R17));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16, R17, R18));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R20));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R20, R21));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R20, R21, R22));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R20, R21, R22, R23));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R20, R21, R22, R23, R24));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R20, R21, R22, R23, R24, R25));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R20, R21, R22, R23, R24, R25, R26));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R20, R21, R22, R23, R24, R25, R26, R27));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R20, R21, R22, R23, R24, R25, R26, R27, R28));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R20, R21, R22, R23, R24, R25, R26, R27, R28, R29));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R20, R21, R22, R23, R24, R25, R26, R27, R28, R29, R30));
        impl_tuples!(($($l),*),(R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R20, R21, R22, R23, R24, R25, R26, R27, R28, R29, R30, R31));
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
impl_tuples!((L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11, L12, L13, L14, L15, L16));
impl_tuples!((L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11, L12, L13, L14, L15, L16, L17));
impl_tuples!((L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11, L12, L13, L14, L15, L16, L17, L18));
impl_tuples!((
    L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11, L12, L13, L14, L15, L16, L17, L18, L19
));
impl_tuples!((
    L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11, L12, L13, L14, L15, L16, L17, L18, L19, L20
));
impl_tuples!((
    L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11, L12, L13, L14, L15, L16, L17, L18, L19, L20,
    L21
));
impl_tuples!((
    L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11, L12, L13, L14, L15, L16, L17, L18, L19, L20,
    L21, L22
));
impl_tuples!((
    L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11, L12, L13, L14, L15, L16, L17, L18, L19, L20,
    L21, L22, L23
));
impl_tuples!((
    L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11, L12, L13, L14, L15, L16, L17, L18, L19, L20,
    L21, L22, L23, L24
));
impl_tuples!((
    L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11, L12, L13, L14, L15, L16, L17, L18, L19, L20,
    L21, L22, L23, L24, L25
));
impl_tuples!((
    L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11, L12, L13, L14, L15, L16, L17, L18, L19, L20,
    L21, L22, L23, L24, L25, L26
));
impl_tuples!((
    L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11, L12, L13, L14, L15, L16, L17, L18, L19, L20,
    L21, L22, L23, L24, L25, L26, L27
));
impl_tuples!((
    L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11, L12, L13, L14, L15, L16, L17, L18, L19, L20,
    L21, L22, L23, L24, L25, L26, L27, L28
));
impl_tuples!((
    L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11, L12, L13, L14, L15, L16, L17, L18, L19, L20,
    L21, L22, L23, L24, L25, L26, L27, L28, L29
));
impl_tuples!((
    L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11, L12, L13, L14, L15, L16, L17, L18, L19, L20,
    L21, L22, L23, L24, L25, L26, L27, L28, L29, L30
));
impl_tuples!((
    L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, L10, L11, L12, L13, L14, L15, L16, L17, L18, L19, L20,
    L21, L22, L23, L24, L25, L26, L27, L28, L29, L30, L31
));
