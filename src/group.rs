use super::attribute::Attribute;
use super::dimension::Dimension;
use super::error;
use super::variable::{Numeric, Variable};
use super::LOCK;
use netcdf_sys::*;
use std::collections::HashMap;
use std::ffi;

#[derive(Debug)]
pub struct Group {
    pub(crate) name: String,
    pub(crate) id: nc_type,
    pub(crate) variables: HashMap<String, Variable>,
    pub(crate) attributes: HashMap<String, Attribute>,
    pub(crate) dimensions: HashMap<String, Dimension>,
    pub(crate) sub_groups: HashMap<String, Group>,
}

impl Group {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn variables(&self) -> &HashMap<String, Variable> {
        &self.variables
    }
    pub fn variable_mut(&mut self, name: &str) -> Option<&mut Variable> {
        self.variables.get_mut(name)
    }
    pub fn attributes(&self) -> &HashMap<String, Attribute> {
        &self.attributes
    }
    pub fn attribute_mut(&mut self, name: &str) -> Option<&mut Attribute> {
        self.attributes.get_mut(name)
    }
    pub fn dimensions(&self) -> &HashMap<String, Dimension> {
        &self.dimensions
    }
    pub fn sub_groups(&self) -> &HashMap<String, Group> {
        &self.sub_groups
    }
    pub fn sub_groups_mut(&mut self, name: &str) -> Option<&mut Group> {
        self.sub_groups.get_mut(name)
    }
}

// Write support for all variable types
pub trait PutVar {
    const NCTYPE: nc_type;
    fn put(&self, ncid: nc_type, varid: nc_type) -> error::Result<()>;
}

// This macro implements the trait PutVar for &[$type]
// It just avoid code repetition for all numeric types
// (the only difference between each type beeing the
// netCDF funtion to call and the numeric identifier
// of the type used by the libnetCDF library)
macro_rules! impl_putvar {
    ($type: ty, $nc_type: ident, $nc_put_var: ident) => {
        impl PutVar for &[$type] {
            const NCTYPE: nc_type = $nc_type;
            fn put(&self, ncid: nc_type, varid: nc_type) -> error::Result<()> {
                let err;
                unsafe {
                    let _g = LOCK.lock().unwrap();
                    err = $nc_put_var(ncid, varid, self.as_ptr());
                }
                if err != NC_NOERR {
                    return Err(err.into());
                }
                Ok(())
            }
        }
    };
}
impl_putvar!(i8, NC_BYTE, nc_put_var_schar);
impl_putvar!(i16, NC_SHORT, nc_put_var_short);
impl_putvar!(u16, NC_USHORT, nc_put_var_ushort);
impl_putvar!(i32, NC_INT, nc_put_var_int);
impl_putvar!(u32, NC_UINT, nc_put_var_uint);
impl_putvar!(i64, NC_INT64, nc_put_var_longlong);
impl_putvar!(u64, NC_UINT64, nc_put_var_ulonglong);
impl_putvar!(f32, NC_FLOAT, nc_put_var_float);
impl_putvar!(f64, NC_DOUBLE, nc_put_var_double);

// Write support for all attribute types
pub trait PutAttr {
    fn get_nc_type(&self) -> nc_type;
    fn put(&self, ncid: nc_type, varid: nc_type, name: &str) -> error::Result<()>;
}

// This macro implements the trait PutAttr for $type
// It just avoid code repetition for all numeric types
// (the only difference between each type beeing the
// netCDF funtion to call and the numeric identifier
// of the type used by the libnetCDF library)
macro_rules! impl_putattr {
    ($type: ty, $nc_type: ident, $nc_put_att: ident) => {
        impl PutAttr for $type {
            fn get_nc_type(&self) -> nc_type {
                $nc_type
            }
            fn put(&self, ncid: nc_type, varid: nc_type, name: &str) -> error::Result<()> {
                let name_c: ffi::CString = ffi::CString::new(name.clone()).unwrap();
                let err;
                unsafe {
                    let _g = LOCK.lock().unwrap();
                    err = $nc_put_att(ncid, varid, name_c.as_ptr(), $nc_type, 1, self);
                }
                if err != NC_NOERR {
                    return Err(err.into());
                }
                Ok(())
            }
        }
    };
}
impl_putattr!(i8, NC_BYTE, nc_put_att_schar);
impl_putattr!(i16, NC_SHORT, nc_put_att_short);
impl_putattr!(u16, NC_USHORT, nc_put_att_ushort);
impl_putattr!(i32, NC_INT, nc_put_att_int);
impl_putattr!(u32, NC_UINT, nc_put_att_uint);
impl_putattr!(i64, NC_INT64, nc_put_att_longlong);
impl_putattr!(u64, NC_UINT64, nc_put_att_ulonglong);
impl_putattr!(f32, NC_FLOAT, nc_put_att_float);
impl_putattr!(f64, NC_DOUBLE, nc_put_att_double);

impl PutAttr for String {
    fn get_nc_type(&self) -> nc_type {
        NC_CHAR
    }
    fn put(&self, ncid: nc_type, varid: nc_type, name: &str) -> error::Result<()> {
        let name_c: ffi::CString = ffi::CString::new(name.clone()).unwrap();
        let attr_c: ffi::CString = ffi::CString::new(self.clone()).unwrap();
        let err;
        unsafe {
            let _g = LOCK.lock().unwrap();
            err = nc_put_att_text(
                ncid,
                varid,
                name_c.as_ptr(),
                attr_c.to_bytes().len(),
                attr_c.as_ptr(),
            );
        }
        if err != NC_NOERR {
            return Err(err.into());
        }
        Ok(())
    }
}

impl Group {
    pub fn add_attribute<T: PutAttr>(&mut self, name: &str, val: T) -> error::Result<()> {
        val.put(self.id, NC_GLOBAL, name)?;
        self.attributes.insert(
            name.to_string().clone(),
            Attribute {
                name: name.to_string().clone(),
                attrtype: val.get_nc_type(),
                id: 0, // XXX Should Attribute even keep track of an id?
                var_id: NC_GLOBAL,
                file_id: self.id,
            },
        );
        Ok(())
    }

    pub fn add_dimension(&mut self, name: &str, len: usize) -> error::Result<&mut Dimension> {
        if self.dimensions.contains_key(name) {
            return Err(format!("Dimension {} already exists", name).into());
        }

        self.dimensions
            .insert(name.into(), Dimension::new(self.id, name, len)?);

        Ok(self.dimensions.get_mut(name).unwrap())
    }

    /// Create a Variable into the dataset, without writting any data into it.
    pub fn add_variable(
        &mut self,
        name: &str,
        dims: &[&str],
        nctype: nc_type,
    ) -> error::Result<&mut Variable> {
        if let Some(_) = self.variables.get(name) {
            return Err(format!("variable {} already exists", name).into());
        }

        // Assert all dimensions exists, and get &[&Dimension]
        let (d, e): (Vec<_>, Vec<_>) = dims
            .iter()
            .map(|x| self.dimensions.get(*x).ok_or(*x))
            .partition(Result::is_ok);

        if e.len() != 0 {
            return Err(format!(
                "Dimensions not found: {:?}",
                e.into_iter().map(Result::unwrap_err).collect::<Vec<_>>()
            )
            .into());
        }

        let d = d.into_iter().map(Result::unwrap).collect::<Vec<_>>();
        let var = Variable::new(self.id, name, &d, nctype)?;

        self.variables.insert(name.into(), var);

        Ok(self.variables.get_mut(name).unwrap())
    }
}
