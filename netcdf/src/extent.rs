//! Extents used for putting and getting data
//! from a variable

use std::convert::Infallible;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::iter::StepBy;
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

use crate::dimension::Dimension;
use crate::error;

#[derive(Debug, Clone, Copy)]
/// An extent of a dimension
pub enum Extent {
    /// A slice
    Slice {
        /// Start of slice
        start: usize,
        /// Stride of slice
        stride: isize,
    },
    /// A slice with an end
    SliceEnd {
        /// Start of slice
        start: usize,
        /// End of slice
        end: usize,
        /// Stride of slice
        stride: isize,
    },
    /// A slice with a count
    SliceCount {
        /// Start of slice
        start: usize,
        /// Number of elements in slice
        count: usize,
        /// Stride of slice
        stride: isize,
    },
    /// A slice which is just an index
    Index(usize),
}

macro_rules! impl_for_ref {
    ($from: ty : $item: ty) => {
        impl From<&$from> for $item {
            fn from(e: &$from) -> Self {
                Self::from(e.clone())
            }
        }
    };
    (TryFrom $from: ty : $item: ty) => {
        impl TryFrom<&$from> for $item {
            type Error = error::Error;
            fn try_from(e: &$from) -> Result<Self, Self::Error> {
                Self::try_from(e.clone())
            }
        }
    };
}

impl From<usize> for Extent {
    fn from(start: usize) -> Self {
        Self::Index(start)
    }
}
impl_for_ref!(usize: Extent);

impl From<RangeFrom<usize>> for Extent {
    fn from(range: RangeFrom<usize>) -> Self {
        Self::Slice {
            start: range.start,
            stride: 1,
        }
    }
}
impl_for_ref!(RangeFrom<usize> : Extent);

impl From<StepBy<RangeFrom<usize>>> for Extent {
    fn from(mut range: StepBy<RangeFrom<usize>>) -> Self {
        let first = range
            .next()
            .expect("Iterator must contain at least two items");
        let second = range
            .next()
            .expect("Iterator must contain at least two items");

        let stride = isize::try_from(second - first).unwrap_or(isize::MAX);
        Self::Slice {
            start: first,
            stride,
        }
    }
}
impl_for_ref!(StepBy<RangeFrom<usize>> : Extent);

impl From<Range<usize>> for Extent {
    fn from(range: Range<usize>) -> Self {
        Self::SliceEnd {
            start: range.start,
            end: range.end,
            stride: 1,
        }
    }
}
impl_for_ref!(Range<usize> : Extent);

impl From<StepBy<Range<usize>>> for Extent {
    fn from(mut range: StepBy<Range<usize>>) -> Self {
        match (range.next(), range.next()) {
            (None, _) => Self::SliceCount {
                start: 0,
                count: 0,
                stride: 1,
            },
            (Some(first), Some(second)) => {
                let stride = isize::try_from(second - first).unwrap_or(isize::MAX);
                let last = range.last().unwrap_or(second);

                Self::SliceEnd {
                    start: first,
                    end: last,
                    stride,
                }
            }
            (Some(first), None) => Self::SliceCount {
                start: first,
                count: 1,
                stride: 1,
            },
        }
    }
}
impl_for_ref!(StepBy<Range<usize>> : Extent);

impl From<RangeTo<usize>> for Extent {
    fn from(range: RangeTo<usize>) -> Self {
        Self::SliceEnd {
            start: 0,
            end: range.end,
            stride: 1,
        }
    }
}
impl_for_ref!(RangeTo<usize> : Extent);

impl From<RangeToInclusive<usize>> for Extent {
    fn from(range: RangeToInclusive<usize>) -> Self {
        Self::SliceEnd {
            start: 0,
            end: range.end + 1,
            stride: 1,
        }
    }
}
impl_for_ref!(RangeToInclusive<usize> : Extent);

impl From<RangeInclusive<usize>> for Extent {
    fn from(range: RangeInclusive<usize>) -> Self {
        Self::SliceEnd {
            start: *range.start(),
            end: range.end() + 1,
            stride: 1,
        }
    }
}
impl_for_ref!(RangeInclusive<usize> : Extent);

impl From<StepBy<RangeInclusive<usize>>> for Extent {
    fn from(mut range: StepBy<RangeInclusive<usize>>) -> Self {
        match (range.next(), range.next()) {
            (None, _) => Self::SliceCount {
                start: 0,
                count: 0,
                stride: 1,
            },
            (Some(first), Some(second)) => {
                let stride = isize::try_from(second - first).unwrap_or(isize::MAX);
                let last = range.last().unwrap_or(second);

                Self::SliceEnd {
                    start: first,
                    end: last + 1,
                    stride,
                }
            }
            (Some(first), None) => Self::SliceCount {
                start: first,
                count: 1,
                stride: 1,
            },
        }
    }
}
impl_for_ref!(StepBy<RangeInclusive<usize>> : Extent);

impl From<RangeFull> for Extent {
    fn from(_: RangeFull) -> Self {
        Self::Slice {
            start: 0,
            stride: 1,
        }
    }
}
impl_for_ref!(RangeFull: Extent);

impl Extent {
    const fn stride(&self) -> Option<isize> {
        match *self {
            Self::Slice { start: _, stride }
            | Self::SliceEnd {
                start: _,
                stride,
                end: _,
            }
            | Self::SliceCount {
                start: _,
                stride,
                count: _,
            } => Some(stride),
            Self::Index(_start) => None,
        }
    }
    /// Set stride of the slice
    pub fn set_stride(&mut self, stride: isize) {
        let s = stride;
        match self {
            Self::Slice { start: _, stride }
            | Self::SliceEnd {
                start: _,
                stride,
                end: _,
            }
            | Self::SliceCount {
                start: _,
                stride,
                count: _,
            } => *stride = s,
            Self::Index(_start) => {}
        }
    }
}

#[derive(Debug, Clone)]
/// A selector for putting and getting data along a dataset
///
/// This type can be constructed in many ways
/// ```rust,no_run
/// use netcdf::extent::{Extent, Extents};
/// // Get all values
/// let _: Extents = (..).into();
/// // Get array with only first 10 of the first dimension
/// // and the first 2 of the second dimension
/// let _: Extents = [..10, ..2].into();
/// // Get values after some index
/// let _: Extents = [1.., 2..].into();
/// // The above syntax (using arrays) does not allow arbitrary dimensions,
/// // for this use tuples
/// let _: Extents = (
///     1..10,
///     (2..=100).step_by(3),
///     4,
/// ).try_into().unwrap();
/// // Or specify counts using slices of `Extent`
/// let _: Extents = [
///     Extent::SliceCount { start: 0, count: 10, stride: 1 },
///     (5..).into(),
/// ].into();
/// // Use two arrays to specify start and count
/// let _: Extents = (&[1, 2, 3], &[3, 2, 1]).try_into().unwrap();
/// // Use three arrays to specify start, count and stride
/// let _: Extents = (&[1, 2, 3], &[3, 2, 1], &[4, 5, 6]).try_into().unwrap();
/// // The `ndarray::s!` macro can also be used if `ndarray` feature is activated
/// ```
pub enum Extents {
    /// The full variable
    All,
    /// A selection along each dimension
    Extent(Vec<Extent>),
}

impl Default for Extents {
    fn default() -> Self {
        Self::All
    }
}

impl From<std::ops::RangeFull> for Extents {
    fn from(_: std::ops::RangeFull) -> Self {
        Self::All
    }
}

impl From<Vec<Extent>> for Extents {
    fn from(slice: Vec<Extent>) -> Self {
        Self::Extent(slice)
    }
}

impl From<&'_ [Extent]> for Extents {
    fn from(slice: &[Extent]) -> Self {
        Self::Extent(slice.to_owned())
    }
}

impl<const N: usize> From<[Extent; N]> for Extents {
    fn from(slice: [Extent; N]) -> Self {
        Self::Extent(slice.to_vec())
    }
}

macro_rules! impl_extent_as_extents {
    ($item: ty) => {
        impl From<$item> for Extents {
            fn from(item: $item) -> Self {
                Self::from(&item)
            }
        }

        impl From<&$item> for Extents {
            fn from(item: &$item) -> Self {
                Self::Extent(vec![item.into()])
            }
        }
    };
    (TryFrom $item: ty) => {
        impl TryFrom<$item> for Extents {
            type Error = error::Error;
            fn try_from(item: $item) -> Result<Self, Self::Error> {
                Ok(Self::Extent(vec![item.try_into()?]))
            }
        }
        impl TryFrom<&$item> for Extents {
            type Error = error::Error;
            fn try_from(item: &$item) -> Result<Self, Self::Error> {
                Ok(Self::Extent(vec![item.clone().try_into()?]))
            }
        }
    };
}

impl_extent_as_extents!(usize);
impl_extent_as_extents!(RangeFrom<usize>);
impl_extent_as_extents!(Range<usize>);
impl_extent_as_extents!(RangeTo<usize>);
impl_extent_as_extents!(RangeToInclusive<usize>);
impl_extent_as_extents!(RangeInclusive<usize>);

impl_extent_as_extents!(StepBy<RangeFrom<usize>>);
impl_extent_as_extents!(StepBy<Range<usize>>);
impl_extent_as_extents!(StepBy<RangeInclusive<usize>>);

macro_rules! impl_extent_arrlike {
    ($item: ty) => {
        impl From<&'_ [$item]> for Extents {
            fn from(slice: &[$item]) -> Self {
                Self::Extent(slice.iter().map(|s| s.into()).collect())
            }
        }
        impl From<Vec<$item>> for Extents {
            fn from(slice: Vec<$item>) -> Self {
                Self::from(slice.as_slice())
            }
        }

        impl<const N: usize> From<[$item; N]> for Extents {
            fn from(slice: [$item; N]) -> Self {
                Self::from(slice.as_slice())
            }
        }
        impl<const N: usize> From<&[$item; N]> for Extents {
            fn from(slice: &[$item; N]) -> Self {
                Self::from(slice.as_slice())
            }
        }
    };
    (TryFrom $item: ty) => {
        impl TryFrom<&'_ [$item]> for Extents
        //where <$item as TryInto<Extent>>::Error: Into<error::Error>,
        {
            type Error = error::Error;
            fn try_from(slice: &[$item]) -> Result<Self, Self::Error> {
                Ok(Self::Extent(
                    slice
                        .iter()
                        .map(|s| {
                            let extent: Extent = s.try_into()?;
                            Ok(extent)
                        })
                        .collect::<Result<Vec<Extent>, error::Error>>()?,
                ))
            }
        }
        impl TryFrom<Vec<$item>> for Extents {
            type Error = error::Error;
            fn try_from(slice: Vec<$item>) -> Result<Self, Self::Error> {
                Self::try_from(slice.as_slice())
            }
        }

        impl<const N: usize> TryFrom<[$item; N]> for Extents {
            type Error = error::Error;
            fn try_from(slice: [$item; N]) -> Result<Self, Self::Error> {
                Self::try_from(slice.as_slice())
            }
        }
        impl<const N: usize> TryFrom<&[$item; N]> for Extents {
            type Error = error::Error;
            fn try_from(slice: &[$item; N]) -> Result<Self, Self::Error> {
                Self::try_from(slice.as_slice())
            }
        }
    };
}

impl_extent_arrlike!(usize);
impl_extent_arrlike!(RangeFrom<usize>);
impl_extent_arrlike!(Range<usize>);
impl_extent_arrlike!(RangeTo<usize>);
impl_extent_arrlike!(RangeToInclusive<usize>);
impl_extent_arrlike!(RangeInclusive<usize>);
impl_extent_arrlike!(RangeFull);
impl_extent_arrlike!(StepBy<RangeFrom<usize>>);
impl_extent_arrlike!(StepBy<Range<usize>>);
impl_extent_arrlike!(StepBy<RangeInclusive<usize>>);

macro_rules! impl_tuple {
    () => ();

    ($head:ident, $($tail:ident,)*) => (
        #[allow(non_snake_case)]
        impl<$head, $($tail,)*> TryFrom<($head, $($tail,)*)> for Extents
            where
                $head: TryInto<Extent>,
                $head::Error: Into<error::Error>,
                $(
                    $tail: TryInto<Extent>,
                    $tail::Error: Into<error::Error>,
                )*
        {
            type Error = error::Error;
            fn try_from(slice: ($head, $($tail,)*)) -> Result<Self, Self::Error> {
                let ($head, $($tail,)*) = slice;
                Ok(vec![($head).try_into().map_err(|e| e.into())?, $(($tail).try_into().map_err(|e| e.into())?,)*].into())
            }
        }

        impl_tuple! { $($tail,)* }
    )
}

impl_tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, }

impl From<()> for Extents {
    fn from(_: ()) -> Self {
        Self::Extent(vec![])
    }
}

pub(crate) type StartCountStride = (Vec<usize>, Vec<usize>, Vec<isize>);

impl Extents {
    pub(crate) fn get_start_count_stride(
        &self,
        dims: &[Dimension],
    ) -> Result<StartCountStride, error::Error> {
        let (start, count, stride): StartCountStride = match self {
            Self::All => {
                let start = dims.iter().map(|_| 0).collect();
                let counts = dims.iter().map(Dimension::len).collect();
                let stride = dims.iter().map(|_| 1).collect();

                (start, counts, stride)
            }
            Self::Extent(extents) => {
                if extents.len() != dims.len() {
                    return Err(error::Error::DimensionMismatch {
                        wanted: dims.len(),
                        actual: extents.len(),
                    });
                }
                let (start, count) = dims
                    .iter()
                    .zip(extents)
                    .map(|(d, &e)| match e {
                        Extent::Index(start) => (start, 1),
                        Extent::Slice { start, stride } => usize::try_from(stride).map_or_else(
                            |_| (start, 0),
                            |stride| (start, (start..d.len()).step_by(stride).count()),
                        ),
                        Extent::SliceCount {
                            start,
                            count,
                            stride: _,
                        } => (start, count),
                        Extent::SliceEnd { start, end, stride } => usize::try_from(stride)
                            .map_or_else(
                                |_| (start, 0),
                                |stride| (start, (start..end).step_by(stride).count()),
                            ),
                    })
                    .unzip();
                let stride = extents.iter().map(|e| e.stride().unwrap_or(1)).collect();
                (start, count, stride)
            }
        };
        Ok((start, count, stride))
    }
}

#[cfg(feature = "ndarray")]
mod ndarray_impl {
    use super::*;
    use ndarray::{Dimension, SliceInfo, SliceInfoElem};

    impl<T, Din: Dimension, Dout: Dimension> TryFrom<&'_ SliceInfo<T, Din, Dout>> for Extents
    where
        T: AsRef<[SliceInfoElem]>,
    {
        type Error = error::Error;
        fn try_from(slice: &SliceInfo<T, Din, Dout>) -> Result<Self, Self::Error> {
            let slice: &[SliceInfoElem] = slice.as_ref();

            Ok(slice
                .iter()
                .map(|&s| match s {
                    SliceInfoElem::Slice { start, end, step } => {
                        let start = usize::try_from(start).map_err(|_| "Invalid start")?;
                        let stride = step;

                        if let Some(end) = end {
                            let end = usize::try_from(end).map_err(|_| "Invalid end")?;
                            Ok(Extent::SliceEnd { start, end, stride })
                        } else {
                            Ok(Extent::Slice { start, stride })
                        }
                    }
                    SliceInfoElem::Index(index) => {
                        let index = usize::try_from(index).map_err(|_| "Invalid index")?;
                        Ok(Extent::Index(index))
                    }
                    SliceInfoElem::NewAxis => Err("Can't add new axis in this context".into()),
                })
                .collect::<Result<Vec<Extent>, Self::Error>>()?
                .into())
        }
    }

    impl<T, Din: Dimension, Dout: Dimension> TryFrom<SliceInfo<T, Din, Dout>> for Extents
    where
        T: AsRef<[SliceInfoElem]>,
    {
        type Error = error::Error;
        fn try_from(slice: SliceInfo<T, Din, Dout>) -> Result<Self, Self::Error> {
            Self::try_from(&slice)
        }
    }
}

impl TryFrom<(&[usize], &[usize])> for Extents {
    type Error = error::Error;
    fn try_from((start, count): (&[usize], &[usize])) -> Result<Self, Self::Error> {
        if start.len() == count.len() {
            Ok(Self::Extent(
                start
                    .iter()
                    .zip(count)
                    .map(|(&start, &count)| Extent::SliceCount {
                        start,
                        count,
                        stride: 1,
                    })
                    .collect(),
            ))
        } else {
            Err("Indices and count does not have the same length".into())
        }
    }
}

impl TryFrom<(Vec<usize>, Vec<usize>)> for Extents {
    type Error = error::Error;
    fn try_from((start, count): (Vec<usize>, Vec<usize>)) -> Result<Self, Self::Error> {
        Self::try_from((start.as_slice(), count.as_slice()))
    }
}

impl TryFrom<(&[usize], &[usize], &[isize])> for Extents {
    type Error = error::Error;
    fn try_from(
        (start, count, stride): (&[usize], &[usize], &[isize]),
    ) -> Result<Self, Self::Error> {
        if start.len() != count.len() || start.len() != stride.len() {
            Err("Indices or count or stride does not have the same length".into())
        } else {
            Ok(Self::Extent(
                start
                    .iter()
                    .zip(count)
                    .zip(stride)
                    .map(|((&start, &count), &stride)| Extent::SliceCount {
                        start,
                        count,
                        stride,
                    })
                    .collect(),
            ))
        }
    }
}

impl TryFrom<(Vec<usize>, Vec<usize>, Vec<isize>)> for Extents {
    type Error = error::Error;
    fn try_from(
        (start, count, stride): (Vec<usize>, Vec<usize>, Vec<isize>),
    ) -> Result<Self, Self::Error> {
        Self::try_from((start.as_slice(), count.as_slice(), stride.as_slice()))
    }
}

macro_rules! impl_extents_for_arrays {
    ($N: expr) => {
        impl TryFrom<([usize; $N], [usize; $N])> for Extents {
            type Error = Infallible;
            fn try_from((start, count): ([usize; $N], [usize; $N])) -> Result<Self, Self::Error> {
                    Self::try_from((&start, &count))
            }
        }

        impl TryFrom<(&[usize; $N], &[usize; $N])> for Extents {
            type Error = Infallible;
            fn try_from((start, count): (&[usize; $N], &[usize; $N])) -> Result<Self, Self::Error> {
                    Ok(Self::Extent(
                        start
                            .iter()
                            .zip(count)
                            .map(|(&start, &count)| Extent::SliceCount {
                                start,
                                count,
                                stride: 1,
                            })
                            .collect(),
                    ))
            }
        }

        impl TryFrom<([usize; $N], [usize; $N], [isize; $N])> for Extents {
            type Error = Infallible;
            fn try_from((start, count, stride): ([usize; $N], [usize; $N], [isize; $N])) -> Result<Self, Self::Error> {
                    Self::try_from((&start, &count, &stride))
            }
        }

        impl TryFrom<(&[usize; $N], &[usize; $N], &[isize; $N])> for Extents {
            type Error = Infallible;
            fn try_from((start, count, stride): (&[usize; $N], &[usize; $N], &[isize; $N])) -> Result<Self, Self::Error> {
                    Ok(Self::Extent(
                        start
                            .iter()
                            .zip(count)
                            .zip(stride)
                            .map(|((&start, &count), &stride)| Extent::SliceCount {
                                start,
                                count,
                                stride,
                            })
                            .collect(),
                    ))
            }
        }
    };
    ($($N: expr,)*) => {
        $(impl_extents_for_arrays! { $N })*
    };
}
impl_extents_for_arrays! { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, }

impl From<&Self> for Extents {
    fn from(extents: &Self) -> Self {
        extents.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    fn take_extent<E>(e: E) -> error::Result<Extent>
    where
        E: TryInto<Extent>,
        E::Error: Into<error::Error>,
    {
        e.try_into().map_err(|e| e.into())
    }

    fn take_extents<E>(e: E) -> error::Result<Extents>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        e.try_into().map_err(|e| e.into())
    }

    #[test]
    fn test_extent() -> error::Result<()> {
        let _ = take_extent(1)?;
        let _ = take_extent(1..)?;
        let _ = take_extent(1..5)?;
        let _ = take_extent(..5)?;
        let _ = take_extent(..=5)?;
        let _ = take_extent(4..=5)?;

        let _ = take_extent((1..).step_by(2))?;
        let _ = take_extent((1..5).step_by(2))?;
        // let _ = take_extent((..5).step_by(2))?;
        // let _ = take_extent((..=5).step_by(2))?;
        let _ = take_extent((4..=9).step_by(2))?;

        // Empty slice
        let _ = take_extent(1..0)?;
        let _ = take_extent(1..=1)?;
        let _ = take_extent(1..=2)?;

        Ok(())
    }

    #[test]
    fn test_extents() -> error::Result<()> {
        // This is the "All" type
        let extent = take_extents(..)?;
        match extent {
            Extents::All => {}
            _ => panic!(),
        }

        // These are for 1D specs
        let _ = take_extents(1)?;
        let _ = take_extents(1..)?;
        let _ = take_extents(1..5)?;
        let _ = take_extents(..5)?;
        let _ = take_extents(..=5)?;
        let _ = take_extents(4..=5)?;
        let _ = take_extents((1..).step_by(2))?;
        let _ = take_extents((1..5).step_by(2))?;
        let _ = take_extents((4..=9).step_by(2))?;

        // These are multidimensional

        // Array
        let _ = take_extents([.., ..])?;
        let _ = take_extents([1, 2])?;
        let _ = take_extents([1.., 2..])?;
        let _ = take_extents([1..5, 2..6])?;
        let _ = take_extents([..5, ..6])?;
        let _ = take_extents([..=5, ..=6])?;
        let _ = take_extents([4..=50, 5..=8])?;
        let _ = take_extents([(1..).step_by(2), (2..).step_by(3)])?;
        let _ = take_extents([(1..5).step_by(2), (2..89).step_by(3)])?;
        let _ = take_extents([(4..=9).step_by(2), (5..=20).step_by(3)])?;

        // Slice
        let _ = take_extents([.., ..].as_slice())?;
        let _ = take_extents([1, 2].as_slice())?;
        let _ = take_extents([1.., 2..].as_slice())?;
        let _ = take_extents([1..5, 2..6].as_slice())?;
        let _ = take_extents([..5, ..6].as_slice())?;
        let _ = take_extents([..=5, ..=6].as_slice())?;
        let _ = take_extents([4..=5, 5..=6].as_slice())?;
        let _ = take_extents([(1..).step_by(2), (2..).step_by(3)].as_slice())?;
        let _ = take_extents([(1..5).step_by(2), (2..89).step_by(3)].as_slice())?;
        let _ = take_extents([(4..=9).step_by(2), (5..=20).step_by(3)].as_slice())?;

        // Vec
        let _ = take_extents(vec![.., ..])?;
        let _ = take_extents(vec![1, 2])?;
        let _ = take_extents(vec![1.., 2..])?;
        let _ = take_extents(vec![1..5, 2..6])?;
        let _ = take_extents(vec![..5, ..6])?;
        let _ = take_extents(vec![..=5, ..=6])?;
        let _ = take_extents(vec![4..=5, 5..=6])?;
        let _ = take_extents(vec![(1..).step_by(2), (2..).step_by(3)])?;
        let _ = take_extents(vec![(1..5).step_by(2), (2..89).step_by(3)])?;
        let _ = take_extents(vec![(4..=9).step_by(2), (5..=20).step_by(3)])?;

        // Tuple
        let _ = take_extents((1_usize.., 2_usize))?;
        let _ = take_extents((1.., 2))?;
        let _ = take_extents((2, (1..10).step_by(3)))?;

        #[cfg(feature = "ndarray")]
        {
            let _ = take_extents(ndarray::s![2..;4, 4])?;
        }

        // (start, count)
        let _ = take_extents(([1, 2], [3, 4]))?;
        let _ = take_extents(([1, 2].as_slice(), [3, 4].as_slice()))?;
        let _ = take_extents((&[1, 2], &[3, 4]))?;
        let _ = take_extents((vec![1, 2], vec![3, 4]))?;

        // (start, count, stride)
        let _ = take_extents(([1, 2], [3, 4], [4, 5]))?;
        let _ = take_extents(([1, 2].as_slice(), [3, 4].as_slice(), [4, 5].as_slice()))?;
        let _ = take_extents((&[1, 2], &[3, 4], &[4, 5]))?;
        let _ = take_extents((vec![1, 2], vec![3, 4], vec![4, 5]))?;

        // Use of borrowed Extents
        let e: Extents = (..).into();
        let _ = take_extents(&e)?;
        let _ = take_extents(e)?;

        Ok(())
    }
}
