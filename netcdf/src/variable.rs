//! Variables in the netcdf file
#![allow(clippy::similar_names)]

use std::marker::PhantomData;

#[cfg(feature = "ndarray")]
use ndarray::ArrayD;
use netcdf_sys::*;

use super::attribute::{Attribute, AttributeValue};
use super::dimension::Dimension;
use super::error;
use super::extent::Extents;
use crate::types::{NcTypeDescriptor, NcVariableType};
use crate::utils;

#[allow(clippy::doc_markdown)]
/// This struct defines a `netCDF` variable.
///
/// This type is used for retrieving data from a variable.
/// Metadata on the `netCDF`-level can be retrieved using e.g.
/// [`fill_value`](Self::fill_value), [`endinanness`](Self::endianness).
/// Use [`attributes`](Self::attribute) to get additional metadata assigned
/// by the data producer. This crate will not apply any of the transformations
/// given by such attributes (e.g. `add_offset` and `scale_factor` are NOT considered).
///
/// Use the `get*`-functions to retrieve values.
#[derive(Debug, Clone)]
pub struct Variable<'g> {
    /// The variable name
    pub(crate) dimensions: Vec<Dimension<'g>>,
    /// the `netCDF` variable type identifier (from netcdf-sys)
    pub(crate) vartype: nc_type,
    pub(crate) ncid: nc_type,
    pub(crate) varid: nc_type,
    pub(crate) _group: PhantomData<&'g nc_type>,
}

#[derive(Debug)]
/// Mutable access to a variable.
///
/// This type is used for defining and inserting data into a variable.
/// Some properties is required to be set before putting data, such as
/// [`set_chunking`](Self::set_chunking) and [`set_compression`](Self::set_compression).
/// After these are defined one can use the `put*`-functions to insert data into the variable.
///
/// This type derefs to a [`Variable`](Variable), which means [`VariableMut`](Self)
/// can be used where [`Variable`](Variable) is expected.
#[allow(clippy::module_name_repetitions)]
pub struct VariableMut<'g>(
    pub(crate) Variable<'g>,
    pub(crate) PhantomData<&'g mut nc_type>,
);

impl<'g> std::ops::Deref for VariableMut<'g> {
    type Target = Variable<'g>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Enum for variables endianness
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Endianness {
    /// Native endianness, depends on machine architecture (x86_64 is Little)
    Native,
    /// Lille endian
    Little,
    /// Big endian
    Big,
}

#[allow(clippy::len_without_is_empty)]
impl<'g> Variable<'g> {
    pub(crate) fn find_from_name(ncid: nc_type, name: &str) -> error::Result<Option<Variable<'g>>> {
        let cname = super::utils::short_name_to_bytes(name)?;
        let mut varid = 0;
        let e =
            unsafe { utils::with_lock(|| nc_inq_varid(ncid, cname.as_ptr().cast(), &mut varid)) };
        if e == NC_ENOTVAR {
            return Ok(None);
        }
        error::checked(e)?;

        let mut xtype = 0;
        let mut ndims = 0;
        unsafe {
            error::checked(utils::with_lock(|| {
                nc_inq_var(
                    ncid,
                    varid,
                    std::ptr::null_mut(),
                    &mut xtype,
                    &mut ndims,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            }))?;
        }
        let mut dimids = vec![0; ndims.try_into()?];
        unsafe {
            error::checked(utils::with_lock(|| {
                nc_inq_vardimid(ncid, varid, dimids.as_mut_ptr())
            }))?;
        }
        let dimensions = super::dimension::dimensions_from_variable(ncid, varid)?
            .collect::<error::Result<Vec<_>>>()?;

        Ok(Some(Variable {
            dimensions,
            ncid,
            varid,
            vartype: xtype,
            _group: PhantomData,
        }))
    }

    /// Get the name of variable
    pub fn name(&self) -> String {
        let mut name = vec![0_u8; NC_MAX_NAME as usize + 1];
        unsafe {
            error::checked(utils::with_lock(|| {
                nc_inq_varname(self.ncid, self.varid, name.as_mut_ptr().cast())
            }))
            .unwrap();
        }
        let zeropos = name.iter().position(|&x| x == 0).unwrap_or(name.len());
        name.resize(zeropos, 0);

        String::from_utf8(name).expect("Variable name contained invalid sequence")
    }
    /// Get an attribute of this variable
    pub fn attribute<'a>(&'a self, name: &str) -> Option<Attribute<'a>> {
        // Need to lock when reading the first attribute (per variable)
        Attribute::find_from_name(self.ncid, Some(self.varid), name)
            .expect("Could not retrieve attribute")
    }
    /// Iterator over all the attributes of this variable
    pub fn attributes(&self) -> impl Iterator<Item = Attribute> {
        // Need to lock when reading the first attribute (per variable)
        crate::attribute::AttributeIterator::new(self.ncid, Some(self.varid))
            .expect("Could not get attributes")
            .map(Result::unwrap)
    }
    /// Get the attribute value
    ///
    /// # Example
    /// ```rust,no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let var: netcdf::Variable = todo!();
    /// let capture_date: String = var.attribute_value("capture_date").transpose()?
    ///                               .expect("no such attribute").try_into()?;
    /// println!("Captured at {capture_date}");
    /// # Ok(())
    /// # }
    /// ```
    pub fn attribute_value(&self, name: &str) -> Option<error::Result<AttributeValue>> {
        self.attribute(name).as_ref().map(Attribute::value)
    }
    /// Dimensions for a variable
    pub fn dimensions(&self) -> &[Dimension] {
        &self.dimensions
    }
    /// Get the type of this variable
    pub fn vartype(&self) -> NcVariableType {
        crate::types::read_type(self.ncid, self.vartype).expect("Unknown type encountered")
    }
    /// Get current length of the variable
    pub fn len(&self) -> usize {
        self.dimensions
            .iter()
            .map(Dimension::len)
            .fold(1_usize, usize::saturating_mul)
    }
    /// Get endianness of the variable.
    ///
    /// # Errors
    ///
    /// Not a `netCDF-4` file
    pub fn endianness(&self) -> error::Result<Endianness> {
        let mut e: nc_type = 0;
        unsafe {
            error::checked(utils::with_lock(|| {
                nc_inq_var_endian(self.ncid, self.varid, &mut e)
            }))?;
        }
        match e {
            NC_ENDIAN_NATIVE => Ok(Endianness::Native),
            NC_ENDIAN_LITTLE => Ok(Endianness::Little),
            NC_ENDIAN_BIG => Ok(Endianness::Big),
            _ => Err(NC_EVARMETA.into()),
        }
    }

    fn access_mode(&self, mode: crate::par::AccessMode) -> error::Result<()> {
        error::checked(super::with_lock(|| unsafe {
            netcdf_sys::par::nc_var_par_access(
                self.ncid,
                self.varid,
                mode as i32 as std::ffi::c_int,
            )
        }))
    }

    /// Access the variable in independent mode
    /// for parallell reading using MPI.
    /// File must have been opened using `open_par`
    ///
    /// This is the default access mode
    pub fn access_independent(&self) -> error::Result<()> {
        crate::par::set_access_mode(self.ncid, self.varid, crate::par::AccessMode::Independent)
    }
    /// Access the variable in collective mode
    /// for parallell reading using MPI.
    /// File must have been opened using `open_par`
    pub fn access_collective(&self) -> error::Result<()> {
        crate::par::set_access_mode(self.ncid, self.varid, crate::par::AccessMode::Collective)
    }
}
impl<'g> VariableMut<'g> {
    /// Sets compression on the variable. Must be set before filling in data.
    ///
    /// `deflate_level` can take a value 0..=9, with 0 being no
    /// compression (good for CPU bound tasks), and 9 providing the
    /// highest compression level (good for memory bound tasks)
    ///
    /// `shuffle` enables a filter to reorder bytes before compressing, which
    /// can improve compression ratios
    ///
    /// # Errors
    ///
    /// Not a `netcdf-4` file or `deflate_level` not valid
    pub fn set_compression(&mut self, deflate_level: nc_type, shuffle: bool) -> error::Result<()> {
        unsafe {
            error::checked(utils::with_lock(|| {
                nc_def_var_deflate(
                    self.ncid,
                    self.varid,
                    shuffle.into(),
                    <_>::from(true),
                    deflate_level,
                )
            }))?;
        }

        Ok(())
    }

    /// Set chunking for variable. Must be set before inserting data
    ///
    /// Use this when reading or writing smaller units of the hypercube than
    /// the full dimensions lengths, to change how the variable is stored in
    /// the file. This has no effect on the memory order when reading/putting
    /// a buffer.
    ///
    /// # Errors
    ///
    /// Not a `netCDF-4` file or invalid chunksize
    pub fn set_chunking(&mut self, chunksize: &[usize]) -> error::Result<()> {
        if self.dimensions.is_empty() {
            // Can't really set chunking, would lead to segfault
            return Ok(());
        }
        if chunksize.len() != self.dimensions.len() {
            return Err(error::Error::SliceLen);
        }
        let len = chunksize
            .iter()
            .copied()
            .fold(1_usize, usize::saturating_mul);
        if len == usize::MAX {
            return Err(error::Error::Overflow);
        }
        unsafe {
            error::checked(utils::with_lock(|| {
                nc_def_var_chunking(self.ncid, self.varid, NC_CHUNKED, chunksize.as_ptr())
            }))?;
        }

        Ok(())
    }
}

impl<'g> VariableMut<'g> {
    /// Adds an attribute to the variable
    pub fn put_attribute<T>(&mut self, name: &str, val: T) -> error::Result<Attribute>
    where
        T: Into<AttributeValue>,
    {
        Attribute::put(self.ncid, self.varid, name, val.into())
    }
}

impl<'g> Variable<'g> {
    fn get_values_mono<T: NcTypeDescriptor>(&self, extents: &Extents) -> error::Result<Vec<T>> {
        let dims = self.dimensions();
        let (start, count, stride) = extents.get_start_count_stride(dims)?;

        let number_of_elements = count.iter().copied().fold(1_usize, usize::saturating_mul);
        let mut values = Vec::with_capacity(number_of_elements);

        unsafe {
            super::putget::get_vars(
                self,
                &T::type_descriptor(),
                &start,
                &count,
                &stride,
                values.as_mut_ptr(),
            )?;
            values.set_len(number_of_elements);
        };
        Ok(values)
    }

    /// Get multiple values from a variable
    ///
    /// Take notice:
    /// `scale_factor` and `offset_factor` and other attributes are not
    /// automatically applied. To take such into account, you can use code like below
    /// ```rust,no_run
    /// # use netcdf::AttributeValue;
    /// # let f = netcdf::create("file.nc")?;
    /// # let var = f.variable("stuff").unwrap();
    /// // let var = ...
    /// // let values = ...
    /// if let Some(scale_offset) = var.attribute_value("scale_offset").transpose()? {
    ///     let scale_offset: f64 = scale_offset.try_into()?;
    ///     // values += scale_offset
    /// }
    /// # Result::<(), netcdf::Error>::Ok(())
    /// ```
    /// where `Option::transpose` is used to bubble up any read errors
    pub fn get_values<T: NcTypeDescriptor + Copy, E>(&self, extents: E) -> error::Result<Vec<T>>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        let extents: Extents = extents.try_into().map_err(Into::into)?;
        self.get_values_mono(&extents)
    }

    /// Get a single value
    pub fn get_value<T: NcTypeDescriptor + Copy, E>(&self, extents: E) -> error::Result<T>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        let mut elems = self.get_values::<T, _>(extents)?;
        if elems.is_empty() {
            return Err("No elements returned".into());
        }
        if elems.len() > 1 {
            return Err("Too many elements returned".into());
        }
        Ok(elems.pop().unwrap())
    }

    /// Get a string from this variable
    pub fn get_string<E>(&self, extents: E) -> error::Result<String>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        let extents = extents.try_into().map_err(Into::into)?;
        let mut elems = self.get_values_mono::<super::types::NcString>(&extents)?;
        if elems.is_empty() {
            return Err("No elements returned".into());
        }
        if elems.len() > 1 {
            super::utils::checked_with_lock(|| unsafe {
                netcdf_sys::nc_free_string(elems.len(), elems.as_mut_ptr().cast())
            })?;
            return Err("Too many elements returned".into());
        }
        let cstr = unsafe { std::ffi::CStr::from_ptr(elems[0].0) };
        let s = cstr.to_string_lossy().to_string();
        super::utils::checked_with_lock(|| unsafe {
            netcdf_sys::nc_free_string(elems.len(), elems.as_mut_ptr().cast())
        })?;
        Ok(s)
    }

    #[cfg(feature = "ndarray")]
    /// Fetches variable
    fn values_arr_mono<T: NcTypeDescriptor>(&self, extents: &Extents) -> error::Result<ArrayD<T>> {
        let dims = self.dimensions();
        let mut start = vec![];
        let mut count = vec![];
        let mut stride = vec![];
        let mut shape = vec![];

        for item in extents.iter_with_dims(dims)? {
            start.push(item.start);
            count.push(item.count);
            stride.push(item.stride);
            if !item.is_an_index {
                shape.push(item.count);
            }
        }

        let number_of_elements = count.iter().copied().fold(1_usize, usize::saturating_mul);
        let mut values = Vec::with_capacity(number_of_elements);
        super::putget::get_vars(
            self,
            &T::type_descriptor(),
            &start,
            &count,
            &stride,
            values.as_mut_ptr(),
        )?;
        unsafe {
            values.set_len(number_of_elements);
        };

        Ok(ArrayD::from_shape_vec(shape, values).unwrap())
    }

    #[cfg(feature = "ndarray")]
    /// Get values from a variable
    pub fn get<T: NcTypeDescriptor + Copy, E>(&self, extents: E) -> error::Result<ArrayD<T>>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        let extents: Extents = extents.try_into().map_err(Into::into)?;
        self.values_arr_mono(&extents)
    }

    #[cfg(feature = "ndarray")]
    /// Get values from a variable directly into an ndarray
    pub fn get_into<T: NcTypeDescriptor + Copy, E, D>(
        &self,
        extents: E,
        mut out: ndarray::ArrayViewMut<T, D>,
    ) -> error::Result<()>
    where
        D: ndarray::Dimension,
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        let extents = extents.try_into().map_err(Into::into)?;

        let dims = self.dimensions();
        let mut start = Vec::with_capacity(dims.len());
        let mut count = Vec::with_capacity(dims.len());
        let mut stride = Vec::with_capacity(dims.len());

        let mut rem_outshape = out.shape();

        for (pos, item) in extents.iter_with_dims(dims)?.enumerate() {
            start.push(item.start);
            count.push(item.count);
            stride.push(item.stride);
            if !item.is_an_index {
                let cur_dim_len = if let Some((&head, rest)) = rem_outshape.split_first() {
                    rem_outshape = rest;
                    head
                } else {
                    return Err(("Output array dimensionality is less than extents").into());
                };
                if item.count != cur_dim_len {
                    return Err(format!("Item count (position {pos}) as {} but expected in output was {cur_dim_len}", item.count).into());
                }
            }
        }
        if !rem_outshape.is_empty() {
            return Err(("Output array dimensionality is larger than extents").into());
        }

        let Some(slice) = out.as_slice_mut() else {
            return Err("Output array must be in standard layout".into());
        };

        assert_eq!(
            slice.len(),
            count.iter().copied().fold(1, usize::saturating_mul),
            "Output size and number of elements to get are not compatible"
        );

        // Safety:
        // start, count, stride are correct length
        // slice is valid pointer, with enough space to hold all elements
        super::putget::get_vars(
            self,
            &T::type_descriptor(),
            &start,
            &count,
            &stride,
            slice.as_mut_ptr(),
        )
    }

    /// Get the fill value of a variable
    pub fn fill_value<T: NcTypeDescriptor + Copy>(&self) -> error::Result<Option<T>> {
        if T::type_descriptor() != super::types::read_type(self.ncid, self.vartype)? {
            return Err(error::Error::TypeMismatch);
        }
        let mut location = std::mem::MaybeUninit::uninit();
        let mut nofill: nc_type = 0;
        unsafe {
            error::checked(utils::with_lock(|| {
                nc_inq_var_fill(
                    self.ncid,
                    self.varid,
                    &mut nofill,
                    std::ptr::addr_of_mut!(location).cast(),
                )
            }))?;
        }
        if nofill == 1 {
            return Ok(None);
        }

        Ok(Some(unsafe { location.assume_init() }))
    }

    fn values_to_mono<T: NcTypeDescriptor>(
        &self,
        buffer: &mut [T],
        extents: &Extents,
    ) -> error::Result<()> {
        let dims = self.dimensions();
        let (start, count, stride) = extents.get_start_count_stride(dims)?;

        let number_of_elements = count.iter().copied().fold(1_usize, usize::saturating_mul);
        if number_of_elements != buffer.len() {
            return Err(error::Error::BufferLen {
                wanted: number_of_elements,
                actual: buffer.len(),
            });
        }
        super::putget::get_vars(
            self,
            &T::type_descriptor(),
            &start,
            &count,
            &stride,
            buffer.as_mut_ptr(),
        )
    }
    /// Fetches variable into slice
    /// buffer must be able to hold all the requested elements
    pub fn get_values_into<T: NcTypeDescriptor + Copy, E>(
        &self,
        buffer: &mut [T],
        extents: E,
    ) -> error::Result<()>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        let extents: Extents = extents.try_into().map_err(Into::into)?;
        self.values_to_mono(buffer, &extents)
    }
}

impl<'g> VariableMut<'g> {
    fn put_values_mono<T: NcTypeDescriptor>(
        &mut self,
        values: &[T],
        extents: &Extents,
    ) -> error::Result<()> {
        let dims = self.dimensions();
        let (start, mut count, stride) = extents.get_start_count_stride(dims)?;

        let number_of_elements_to_put = values.len();
        let number_of_elements = count.iter().copied().fold(1_usize, usize::saturating_mul);
        if number_of_elements != number_of_elements_to_put {
            if dims.len() == 1 {
                count[0] = values.len();
            } else {
                return Err(error::Error::BufferLen {
                    wanted: number_of_elements,
                    actual: number_of_elements_to_put,
                });
            }
        }

        crate::putget::put_vars(
            self,
            &T::type_descriptor(),
            &start,
            &count,
            &stride,
            values.as_ptr(),
        )?;
        Ok(())
    }
    /// Put a slice of values at `indices`
    pub fn put_values<T: NcTypeDescriptor, E>(
        &mut self,
        values: &[T],
        extents: E,
    ) -> error::Result<()>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        let extents: Extents = extents.try_into().map_err(Into::into)?;
        self.put_values_mono(values, &extents)
    }
    /// Put a value at the specified indices
    pub fn put_value<T: NcTypeDescriptor, E>(&mut self, value: T, extents: E) -> error::Result<()>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        self.put_values(&[value], extents)
    }
    /// Put a string at the specified indices
    pub fn put_string<E>(&mut self, value: &str, extents: E) -> error::Result<()>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
    {
        let cstr = std::ffi::CString::new(value)?;
        let item = super::types::NcString(cstr.as_ptr().cast_mut().cast());
        self.put_value(item, extents)
    }

    /// Set a Fill Value
    ///
    /// # Errors
    ///
    /// Not a `netCDF-4` file, late define, `fill_value` has the wrong type
    #[allow(clippy::needless_pass_by_value)] // All values will be small
    pub fn set_fill_value<T>(&mut self, fill_value: T) -> error::Result<()>
    where
        T: NcTypeDescriptor,
    {
        if T::type_descriptor() != super::types::read_type(self.ncid, self.vartype)? {
            return Err(error::Error::TypeMismatch);
        }
        unsafe {
            error::checked(utils::with_lock(|| {
                nc_def_var_fill(
                    self.ncid,
                    self.varid,
                    NC_FILL,
                    std::ptr::addr_of!(fill_value).cast(),
                )
            }))?;
        }
        Ok(())
    }

    /// Set the fill value to no value. Use this when wanting to avoid
    /// duplicate writes into empty variables.
    ///
    /// # Errors
    ///
    /// Not a `netCDF-4` file
    ///
    /// # Safety
    ///
    /// Reading from this variable after having defined nofill
    /// will read potentially uninitialized data. Normally
    /// one will expect to find some filler value
    pub unsafe fn set_nofill(&mut self) -> error::Result<()> {
        error::checked(utils::with_lock(|| {
            nc_def_var_fill(self.ncid, self.varid, NC_NOFILL, std::ptr::null_mut())
        }))
    }

    /// Set endianness of the variable. Must be set before inserting data
    ///
    /// `endian` can take a `Endianness` value with Native being `NC_ENDIAN_NATIVE` (0),
    /// Little `NC_ENDIAN_LITTLE` (1), Big `NC_ENDIAN_BIG` (2)
    ///
    /// # Errors
    ///
    /// Not a `netCDF-4` file, late define
    pub fn set_endianness(&mut self, e: Endianness) -> error::Result<()> {
        let endianness = match e {
            Endianness::Native => NC_ENDIAN_NATIVE,
            Endianness::Little => NC_ENDIAN_LITTLE,
            Endianness::Big => NC_ENDIAN_BIG,
        };
        unsafe {
            error::checked(utils::with_lock(|| {
                nc_def_var_endian(self.ncid, self.varid, endianness)
            }))?;
        }
        Ok(())
    }

    #[cfg(feature = "ndarray")]
    /// Put values in an ndarray into the variable
    #[allow(clippy::needless_pass_by_value)]
    pub fn put<T: NcTypeDescriptor, E, D>(
        &mut self,
        extent: E,
        arr: ndarray::ArrayView<T, D>,
    ) -> error::Result<()>
    where
        E: TryInto<Extents>,
        E::Error: Into<error::Error>,
        D: ndarray::Dimension,
    {
        let extent = extent.try_into().map_err(Into::into)?;

        let Some(slice) = arr.as_slice() else {
            return Err(
                "Slice is not contiguous or in c-order, you might want to use `as_standard_layout`"
                    .into(),
            );
        };

        let dimlen = self.dimensions.len();
        let mut start = Vec::with_capacity(dimlen);
        let mut count = Vec::with_capacity(dimlen);
        let mut stride = Vec::with_capacity(dimlen);

        let mut remaining_arrshape = arr.shape();
        for (pos, item) in extent.iter_with_dims(self.dimensions())?.enumerate() {
            if item.is_an_index {
                start.push(item.start);
                count.push(item.count);
                stride.push(item.stride);
                continue;
            }
            let arr_len = if let Some((&head, rest)) = remaining_arrshape.split_first() {
                remaining_arrshape = rest;
                head
            } else {
                return Err("Extents have greater dimensionality than the input array".into());
            };

            start.push(item.start);
            if arr_len != item.count {
                if arr_len > item.count && item.is_growable && !item.is_upwards_limited {
                    // Item is allowed to grow to accomodate the
                    // extra values in the array
                } else {
                    return Err(format!(
                        "Variable dimension (at position {pos}) has length {}, but input array has a size of {arr_len}",
                        item.count,
                    )
                    .into());
                }
            }
            count.push(arr_len);
            stride.push(item.stride);
        }
        if !remaining_arrshape.is_empty() {
            return Err("Extents have lesser dimensionality than the input array".into());
        }

        assert_eq!(
            arr.len(),
            count.iter().copied().fold(1, usize::saturating_mul),
            "Mismatch between the number of elements in array and the calculated `count`s"
        );

        // Safety:
        // Dimensionality matches (always pushing in for loop)
        // slice is valid pointer since we assert the size above
        // slice is valid pointer since memory order is standard_layout (C)
        super::putget::put_vars::<T>(
            self,
            &self.vartype(),
            &start,
            &count,
            &stride,
            slice.as_ptr(),
        )
    }
}

impl<'g> VariableMut<'g> {
    pub(crate) fn add_from_str(
        ncid: nc_type,
        xtype: &NcVariableType,
        name: &str,
        dims: &[&str],
    ) -> error::Result<Self> {
        let dimensions = dims
            .iter()
            .map(
                |dimname| match super::dimension::from_name_toid(ncid, dimname) {
                    Ok(Some(id)) => Ok(id),
                    Ok(None) => Err(error::Error::NotFound(format!("dimensions {dimname}"))),
                    Err(e) => Err(e),
                },
            )
            .collect::<error::Result<Vec<_>>>()?;

        let cname = super::utils::short_name_to_bytes(name)?;
        let mut varid = 0;
        let xtype = crate::types::find_type(ncid, xtype)?.expect("Type not found");
        unsafe {
            let dimlen = dimensions.len().try_into()?;
            error::checked(utils::with_lock(|| {
                nc_def_var(
                    ncid,
                    cname.as_ptr().cast(),
                    xtype,
                    dimlen,
                    dimensions.as_ptr(),
                    &mut varid,
                )
            }))?;
        }

        let dimensions = dims
            .iter()
            .map(|dimname| match super::dimension::from_name(ncid, dimname) {
                Ok(None) => Err(error::Error::NotFound(format!("dimensions {dimname}"))),
                Ok(Some(dim)) => Ok(dim),
                Err(e) => Err(e),
            })
            .collect::<error::Result<Vec<_>>>()?;

        Ok(VariableMut(
            Variable {
                ncid,
                varid,
                vartype: xtype,
                dimensions,
                _group: PhantomData,
            },
            PhantomData,
        ))
    }
}

pub(crate) fn variables_at_ncid<'g>(
    ncid: nc_type,
) -> error::Result<impl Iterator<Item = error::Result<Variable<'g>>>> {
    let mut nvars = 0;
    unsafe {
        error::checked(utils::with_lock(|| {
            nc_inq_varids(ncid, &mut nvars, std::ptr::null_mut())
        }))?;
    }
    let mut varids = vec![0; nvars.try_into()?];
    unsafe {
        error::checked(utils::with_lock(|| {
            nc_inq_varids(ncid, std::ptr::null_mut(), varids.as_mut_ptr())
        }))?;
    }
    Ok(varids.into_iter().map(move |varid| {
        let mut xtype = 0;
        unsafe {
            error::checked(utils::with_lock(|| nc_inq_vartype(ncid, varid, &mut xtype)))?;
        }
        let dimensions = super::dimension::dimensions_from_variable(ncid, varid)?
            .collect::<error::Result<Vec<_>>>()?;
        Ok(Variable {
            ncid,
            varid,
            dimensions,
            vartype: xtype,
            _group: PhantomData,
        })
    }))
}

pub(crate) fn add_variable_from_identifiers<'g>(
    ncid: nc_type,
    name: &str,
    dims: &[super::dimension::DimensionIdentifier],
    xtype: nc_type,
) -> error::Result<VariableMut<'g>> {
    let cname = super::utils::short_name_to_bytes(name)?;

    let dimensions = dims
        .iter()
        .map(move |&id| {
            // Internal netcdf detail, the top 16 bits gives the corresponding
            // file handle. This to ensure dimensions are not added from another
            // file which is unrelated to self
            if id.ncid >> 16 != ncid >> 16 {
                return Err(error::Error::WrongDataset);
            }
            let mut dimlen = 0;
            unsafe {
                error::checked(utils::with_lock(|| {
                    nc_inq_dimlen(id.ncid, id.dimid, &mut dimlen)
                }))?;
            }
            Ok(Dimension {
                len: core::num::NonZeroUsize::new(dimlen),
                id,
                _group: PhantomData,
            })
        })
        .collect::<error::Result<Vec<_>>>()?;
    let dims = dims.iter().map(|x| x.dimid).collect::<Vec<_>>();

    let mut varid = 0;
    unsafe {
        let dimlen = dims.len().try_into()?;
        error::checked(utils::with_lock(|| {
            nc_def_var(
                ncid,
                cname.as_ptr().cast(),
                xtype,
                dimlen,
                dims.as_ptr(),
                &mut varid,
            )
        }))?;
    }

    Ok(VariableMut(
        Variable {
            ncid,
            dimensions,
            varid,
            vartype: xtype,
            _group: PhantomData,
        },
        PhantomData,
    ))
}
