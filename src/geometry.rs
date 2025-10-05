use std::fmt;
use windows::Win32::Foundation::{POINT, POINTL, POINTS, RECT, RECTL, SIZE};

#[cfg(feature = "win-display")]
use crate::win::display::{DISPLAYCONFIG_2DREGION, POINTFIX, RECTFX};

use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

mod sealed {
    pub trait Sealed {}
}

pub trait Vec2Kind: sealed::Sealed {}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Vec2<T, K: Vec2Kind>(pub T, pub T, PhantomData<K>);

impl<T: Default, K: Vec2Kind> Default for Vec2<T, K> {
    fn default() -> Self {
        Self(T::default(), T::default(), PhantomData)
    }
}

pub type Pos2<T> = Vec2<T, Absolute>;
pub type Len2<T> = Vec2<T, Relative>;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Absolute {}
impl sealed::Sealed for Absolute {}
impl Vec2Kind for Absolute {}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Relative {}
impl sealed::Sealed for Relative {}
impl Vec2Kind for Relative {}

impl<T: fmt::Debug> fmt::Debug for Pos2<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}, {:?})", self.0, self.1)
    }
}

impl<T: fmt::Debug> fmt::Debug for Len2<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{:?} * {:?}>", self.0, self.1)
    }
}

impl<T, K: Vec2Kind> Vec2<T, K> {
    pub const fn new(a: T, b: T) -> Self {
        Self(a, b, PhantomData)
    }

    pub fn cast_kind<K2: Vec2Kind>(self) -> Vec2<T, K2> {
        Vec2(self.0, self.1, PhantomData)
    }

    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> Vec2<U, K> {
        Vec2(f(self.0), f(self.1), PhantomData)
    }
}

pub trait RectKind: sealed::Sealed {}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Rect<T, U = T, K: RectKind = Ltrb>(pub T, pub T, pub U, pub U, PhantomData<K>);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Ltrb {}
impl sealed::Sealed for Ltrb {}
impl RectKind for Ltrb {}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Xywh {}
impl sealed::Sealed for Xywh {}
impl RectKind for Xywh {}

impl<T: Copy> Rect<T, T, Ltrb> {
    pub const fn from_ltrb(left: T, top: T, right: T, bottom: T) -> Self {
        Self(left, top, right, bottom, PhantomData)
    }

    pub const fn from_corners(top_left: Pos2<T>, bottom_right: Pos2<T>) -> Self {
        Self::from_ltrb(top_left.0, top_left.1, bottom_right.0, bottom_right.1)
    }

    #[must_use]
    pub fn as_xywh(&self) -> Rect<T, T::Output, Xywh>
    where
        T: Sub<Output: Copy>,
    {
        Rect::from_pos_size(self.top_left(), self.size())
    }

    pub fn is_valid(&self) -> bool
    where
        T: PartialOrd,
    {
        self.2 >= self.0 && self.3 >= self.1
    }

    pub fn is_empty(&self) -> bool
    where
        T: PartialOrd,
    {
        self.2 <= self.0 && self.3 <= self.1
    }

    pub fn bottom_right(&self) -> Pos2<T> {
        Vec2::new(self.2, self.3)
    }

    pub fn size(&self) -> Len2<T::Output>
    where
        T: Sub,
    {
        self.bottom_right() - self.top_left()
    }

    #[must_use]
    pub fn with_top_left(mut self, top_left: Pos2<T>) -> Self
    where
        T: Sub + PartialOrd,
        T: AddAssign<T::Output> + SubAssign<T::Output>,
    {
        update_coords(&mut self.0, &mut self.2, top_left.0);
        update_coords(&mut self.1, &mut self.3, top_left.1);
        self
    }

    #[must_use]
    pub fn with_size(mut self, size: Len2<T>) -> Self
    where
        T: Add<Output = T>,
    {
        self.2 = self.0 + size.0;
        self.3 = self.1 + size.1;
        self
    }
}

fn update_coords<T, U>(v0: &mut T, v1: &mut U, new_v0: T)
where
    T: PartialOrd + Copy + Sub,
    U: AddAssign<T::Output> + SubAssign<T::Output>,
{
    if new_v0 > *v0 {
        let delta = new_v0 - *v0;
        *v1 += delta;
    } else if *v0 > new_v0 {
        let delta = *v0 - new_v0;
        *v1 -= delta;
    }
}

impl<T: Copy, U: Copy> Rect<T, U, Xywh> {
    pub const fn from_xywh(x: T, y: T, width: U, height: U) -> Self {
        Self(x, y, width, height, PhantomData)
    }

    pub const fn from_pos_size(top_left: Pos2<T>, size: Len2<U>) -> Self {
        Self::from_xywh(top_left.0, top_left.1, size.0, size.1)
    }

    #[must_use]
    pub fn as_ltrb(&self) -> Rect<T, T, Ltrb>
    where
        T: Add<U, Output = T>,
    {
        Rect::from_corners(self.top_left(), self.bottom_right())
    }

    pub fn is_valid(&self) -> bool
    where
        U: PartialOrd + Default,
    {
        let zero = U::default();
        self.2 >= zero && self.3 >= zero
    }

    pub fn is_empty(&self) -> bool
    where
        U: PartialOrd + Default,
    {
        let zero = U::default();
        self.2 <= zero && self.3 <= zero
    }

    pub fn bottom_right(&self) -> Pos2<T::Output>
    where
        T: Add<U>,
    {
        self.top_left() + self.size()
    }

    pub const fn size(&self) -> Len2<U> {
        Vec2::new(self.2, self.3)
    }

    #[must_use]
    pub fn with_top_left(mut self, top_left: Pos2<T>) -> Self {
        self.0 = top_left.0;
        self.1 = top_left.1;
        self
    }

    #[must_use]
    pub fn with_size(mut self, size: Len2<U>) -> Self {
        self.2 = size.0;
        self.3 = size.1;
        self
    }
}

impl<T: Copy, U, K: RectKind> Rect<T, U, K> {
    pub const fn top_left(&self) -> Pos2<T> {
        Vec2::new(self.0, self.1)
    }
}

impl<T: Copy, U: Copy, K: RectKind> Rect<T, U, K> {
    pub const fn cast_kind<K2: RectKind>(self) -> Rect<T, U, K2> {
        Rect(self.0, self.1, self.2, self.3, PhantomData)
    }
}

impl<T: Default, U: Default, K: RectKind> Default for Rect<T, U, K> {
    fn default() -> Self {
        let (a, b, c, d) = Default::default();
        Self(a, b, c, d, PhantomData)
    }
}

impl<T: fmt::Debug, U: fmt::Debug> fmt::Debug for Rect<T, U, Ltrb> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Rect")
            .field("top_left", &Pos2::new(&self.0, &self.1))
            .field("bottom_right", &Pos2::new(&self.2, &self.3))
            .finish()
    }
}

impl<T: fmt::Debug, U: fmt::Debug> fmt::Debug for Rect<T, U, Xywh> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Rect")
            .field("pos", &Pos2::new(&self.0, &self.1))
            .field("size", &Len2::new(&self.2, &self.3))
            .finish()
    }
}

type Tup2<T> = (T, T);
type Arr2<T> = [T; 2];

trait Pair {
    type Item;
    fn into_pair(self) -> Tup2<Self::Item>;
}

impl<T, K: Vec2Kind> Pair for Vec2<T, K> {
    type Item = T;
    fn into_pair(self) -> (T, T) {
        (self.0, self.1)
    }
}

impl<T> Pair for (T, T) {
    type Item = T;
    fn into_pair(self) -> (T, T) {
        (self.0, self.1)
    }
}

impl<T> Pair for [T; 2] {
    type Item = T;
    fn into_pair(self) -> (T, T) {
        let [a, b] = self;
        (a, b)
    }
}

impl<T: PartialEq, K: Vec2Kind> PartialEq<(T, T)> for Vec2<T, K> {
    fn eq(&self, other: &(T, T)) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl<T: PartialEq, K: Vec2Kind> PartialEq<[T; 2]> for Vec2<T, K> {
    fn eq(&self, other: &[T; 2]) -> bool {
        self.0 == other[0] && self.1 == other[1]
    }
}

macro_rules! binary_operators {
    (
        $(
            $Trait:ident :: $func:ident {
                $(
                    $lhs:ident $op:tt $rhs:ident -> $output:ident;
                )*
            }
        )*
    ) => {$($(
        impl<T: $Trait<U>, U> $Trait<$rhs<U>> for $lhs<T> {
            type Output = $output<T::Output>;
            fn $func(self, rhs: $rhs<U>) -> Self::Output {
                let (a0, b0) = self.into_pair();
                let (a1, b1) = rhs.into_pair();
                $output::new(a0 $op a1, b0 $op b1)
            }
        }
    )*)*}
}

binary_operators! {
    Add::add {
        Pos2 + Len2 -> Pos2;
        Len2 + Len2 -> Len2;
        Pos2 + Tup2 -> Pos2;
        Len2 + Tup2 -> Len2;
        Pos2 + Arr2 -> Pos2;
        Len2 + Arr2 -> Len2;
    }
    Sub::sub {
        Pos2 - Pos2 -> Len2;
        Pos2 - Len2 -> Pos2;
        Len2 - Len2 -> Len2;
        Pos2 - Tup2 -> Pos2;
        Len2 - Tup2 -> Len2;
        Pos2 - Arr2 -> Pos2;
        Len2 - Arr2 -> Len2;
    }
}

impl<T: Mul<U>, U: Copy> Mul<U> for Len2<T> {
    type Output = Len2<T::Output>;
    fn mul(self, rhs: U) -> Self::Output {
        Len2::new(self.0 * rhs, self.1 * rhs)
    }
}

impl<T: Div<U>, U: Copy> Div<U> for Len2<T> {
    type Output = Len2<T::Output>;
    fn div(self, rhs: U) -> Self::Output {
        Len2::new(self.0 / rhs, self.1 / rhs)
    }
}
impl<T, K: Vec2Kind, R> AddAssign<R> for Vec2<T, K>
where
    Self: Add<R, Output = Self>,
    T: AddAssign<R::Item>,
    R: Pair<Item = T>,
{
    fn add_assign(&mut self, rhs: R) {
        let (a, b) = rhs.into_pair();
        self.0 += a;
        self.1 += b;
    }
}

impl<T, K: Vec2Kind, R> SubAssign<R> for Vec2<T, K>
where
    Self: Sub<R, Output = Self>,
    T: SubAssign<R::Item>,
    R: Pair<Item = T>,
{
    fn sub_assign(&mut self, rhs: R) {
        let (a, b) = rhs.into_pair();
        self.0 -= a;
        self.1 -= b;
    }
}

impl<T: Neg, K: Vec2Kind> Neg for Vec2<T, K> {
    type Output = Vec2<T::Output, K>;
    fn neg(self) -> Self::Output {
        self.map(T::neg)
    }
}

impl<T: Add<Output = T> + Copy> Add<Len2<T>> for Rect<T, T, Ltrb> {
    type Output = Self;
    fn add(self, rhs: Len2<T>) -> Self::Output {
        let Rect(l, t, r, b, ..) = self;
        let Vec2(dx, dy, ..) = rhs;
        Self::from_ltrb(l + dx, t + dy, r + dx, b + dy)
    }
}

impl<T: Sub<Output = T> + Copy> Sub<Len2<T>> for Rect<T, T, Ltrb> {
    type Output = Self;
    fn sub(self, rhs: Len2<T>) -> Self::Output {
        let Rect(l, t, r, b, ..) = self;
        let Vec2(dx, dy, ..) = rhs;
        Self::from_ltrb(l - dx, t - dy, r - dx, b - dy)
    }
}

impl<T: AddAssign + Copy> AddAssign<Len2<T>> for Rect<T, T, Ltrb> {
    fn add_assign(&mut self, rhs: Len2<T>) {
        let Vec2(dx, dy, ..) = rhs;
        self.0 += dx;
        self.1 += dy;
        self.2 += dx;
        self.3 += dy;
    }
}

impl<T: SubAssign + Copy> SubAssign<Len2<T>> for Rect<T, T, Ltrb> {
    fn sub_assign(&mut self, rhs: Len2<T>) {
        let Vec2(dx, dy, ..) = rhs;
        self.0 -= dx;
        self.1 -= dy;
        self.2 -= dx;
        self.3 -= dx;
    }
}

impl<T: Add<Output = T> + Copy, U: Copy> Add<Len2<T>> for Rect<T, U, Xywh> {
    type Output = Self;
    fn add(self, rhs: Len2<T>) -> Self::Output {
        let Rect(x, y, w, h, ..) = self;
        let Vec2(dx, dy, ..) = rhs;
        Self::from_xywh(x + dx, y + dy, w, h)
    }
}

impl<T: Sub<Output = T> + Copy, U: Copy> Sub<Len2<T>> for Rect<T, U, Xywh> {
    type Output = Self;
    fn sub(self, rhs: Len2<T>) -> Self::Output {
        let Rect(x, y, w, h, ..) = self;
        let Vec2(dx, dy, ..) = rhs;
        Self::from_xywh(x - dx, y - dy, w, h)
    }
}

impl<T: AddAssign, U> AddAssign<Len2<T>> for Rect<T, U, Xywh> {
    fn add_assign(&mut self, rhs: Len2<T>) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl<T: SubAssign, U> SubAssign<Len2<T>> for Rect<T, U, Xywh> {
    fn sub_assign(&mut self, rhs: Len2<T>) {
        self.0 -= rhs.0;
        self.1 -= rhs.1;
    }
}

macro_rules! impl_from {
    ($(
        fn
        $(<$(
            $T:ident $(: $bound:ident)?
        ),*>)?
        ($from_arg:ident : $from:ty) -> $into:ty = $body:expr;
    )*) => {$(
        impl
        $(<$(
            $T $(: $bound)?
        ),*>)?
        From<$from> for $into {
            fn from($from_arg: $from) -> $into { $body }
        }
    )*}
}

impl_from! {
    // From Win32
    fn(p: POINT) -> Pos2<i32> = Self::new(p.x, p.y);
    fn(p: POINTL) -> Pos2<i32> = Self::new(p.x, p.y);
    fn(p: POINTS) -> Pos2<i16> = Self::new(p.x, p.y);
    fn(s: SIZE) -> Len2<i32> = Self::new(s.cx, s.cy);

    fn(r: RECT) -> Rect<i32> = Self::from_ltrb(r.left, r.top, r.right, r.bottom);
    fn(r: RECTL) -> Rect<i32> = Self::from_ltrb(r.left, r.top, r.right, r.bottom);

    // Into Win32
    fn(p: Pos2<i32>) -> POINT = Self { x: p.0, y: p.1 };
    fn(p: Pos2<i32>) -> POINTL = Self { x: p.0, y: p.1 };
    fn(p: Pos2<i16>) -> POINTS = Self { x: p.0, y: p.1 };
    fn(l: Len2<i32>) -> SIZE = Self { cx: l.0, cy: l.1 };

    fn(r: Rect<i32>) -> RECT = Self { left: r.0, top: r.1, right: r.2, bottom: r.3 };
    fn(r: Rect<i32>) -> RECTL = Self { left: r.0, top: r.1, right: r.2, bottom: r.3 };

    // From array/tuple
    fn<T, K: Vec2Kind>(v: (T, T)) -> Vec2<T, K> = Self::new(v.0, v.1);
    fn<T, K: Vec2Kind>(v: [T; 2]) -> Vec2<T, K> = { let [a, b] = v; Self::new(a, b) };

    // Into array/tuple
    fn<T, K: Vec2Kind>(v: Vec2<T, K>) -> (T, T) = (v.0, v.1);
    fn<T, K: Vec2Kind>(v: Vec2<T, K>) -> [T; 2] = [v.0, v.1];
}

#[cfg(feature = "win-display")]
impl_from! {
    fn(l: Len2<u32>) -> DISPLAYCONFIG_2DREGION = Self { cx: l.0, cy: l.1 };
    fn(p: Pos2<i32>) -> POINTFIX = Self { x: p.0, y: p.1 };
    fn(r: Rect<i32>) -> RECTFX = Self { xLeft: r.0, yTop: r.1, xRight: r.2, yBottom: r.3 };

    fn(p: POINTFIX) -> Pos2<i32> = Self::new(p.x, p.y);
    fn(r: DISPLAYCONFIG_2DREGION) -> Len2<u32> = Self::new(r.cx, r.cy);
    fn(r: RECTFX) -> Rect<i32> = Self::from_ltrb(r.xLeft, r.yTop, r.xRight, r.yBottom);
}
