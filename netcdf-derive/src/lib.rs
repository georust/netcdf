use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{
    parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, LitStr, Type,
    Variant,
};

#[proc_macro_derive(NcType, attributes(netcdf))]
/// Derives `NcTypeDescriptor` for user defined types.
///
/// See the documentation under `netcdf::TypeDescriptor` for examples
///
/// Use `#[netcdf(rename = "name")]` to
/// rename field names or enum member names, or the name
/// of the compound/enum.
///
/// Types one derives `NcType` for must have some properties to
/// ensure correctness:
/// * Structs must have `repr(C)` to ensure layout compatibility
/// * Structs must be packed (no padding allowed)
/// * Enums must have `repr(T)` where `T` is an int type (`{i/u}{8/16/32/64}`)
#[proc_macro_error]
pub fn derive(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut renamed = None;
    let mut repr_c = false;
    for attr in &input.attrs {
        if attr.path().is_ident("netcdf") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    renamed = Some(meta.value()?.parse::<LitStr>()?.value());
                } else {
                    abort!(meta.path, "NcType encountered an unknown attribute");
                }
                Ok(())
            })
            .unwrap();
        } else if attr.path().is_ident("repr") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("C") {
                    repr_c = true;
                }
                Ok(())
            })
            .unwrap();
        }
    }
    let ncname = renamed.unwrap_or_else(|| name.to_string());

    let body = match input.data {
        Data::Struct(DataStruct {
            struct_token: _,
            ref fields,
            semi_token: _,
        }) => {
            if !repr_c {
                abort!(
                    input,
                    "Can not derive NcType for struct without fixed layout";
                    help = "struct must have attribute #[repr(C)]"
                );
            }
            match fields {
                Fields::Named(fields) => impl_compound(name, &ncname, fields.clone()),
                Fields::Unnamed(f) => {
                    abort!(f, "Can not derive NcType for struct with unnamed field"; note="#[derive(NcType)]")
                }
                Fields::Unit => abort!(input, "Can not derive NcType for unit struct"),
            }
        }
        Data::Enum(DataEnum {
            enum_token: _,
            brace_token: _,
            ref variants,
        }) => {
            let mut basetyp = None;
            for attr in &input.attrs {
                if attr.path().is_ident("repr") {
                    attr.parse_nested_meta(|meta| {
                        for item in ["u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64"] {
                            if meta.path.is_ident(item) {
                                basetyp = Some(meta.path.get_ident().unwrap().clone());
                            }
                        }
                        Ok(())
                    })
                    .unwrap();
                }
            }
            let Some(basetyp) = basetyp else {
                abort!(
                    input,
                    "Can not derive NcType for enum without suitable repr";
                    help="Add #[repr(i32)] (or another integer type) as an attribute to the enum"
                );
            };
            impl_enum(/*&name,*/ &ncname, &basetyp, variants.iter())
        }
        Data::Union(_) => abort!(
            input,
            "Can not derive NcType for union";
            note = "netCDF has no concept of Union type"
        ),
    };

    let expanded = quote! {
        const _: () = {
            use netcdf::types::*;

            #[automatically_derived]
            unsafe impl #impl_generics NcTypeDescriptor for #name #ty_generics #where_clause {
                fn type_descriptor() -> NcVariableType {
                    #body
                }
            }
        };
    };
    proc_macro::TokenStream::from(expanded)
}

fn impl_compound(ty: &Ident, ncname: &str, fields: FieldsNamed) -> TokenStream {
    struct FieldInfo {
        name: String,
        typ: Type,
    }
    let mut items: Vec<FieldInfo> = vec![];

    for field in fields.named {
        let ident = field.ident.expect("Field must have a name").clone();
        let mut rename = None;
        for attr in field.attrs {
            if attr.path().is_ident("netcdf") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("rename") {
                        rename = Some(meta.value()?.parse::<LitStr>()?.value());
                    } else {
                        abort!(meta.path, "NcType encountered an unknown attribute")
                    }
                    Ok(())
                })
                .unwrap();
            }
        }
        let name = rename.unwrap_or_else(|| ident.to_string());
        items.push(FieldInfo {
            name,
            typ: field.ty,
        });
    }

    let fieldnames = items
        .iter()
        .map(|item| item.name.clone())
        .collect::<Vec<_>>();
    let typeids = items
        .iter()
        .map(|item| item.typ.clone())
        .collect::<Vec<_>>();
    let fieldinfo = quote!(vec![#(
            (
                (#fieldnames).to_owned(),
                <#typeids as NcTypeDescriptor>::type_descriptor(),
                (<#typeids as NcTypeDescriptor>::ARRAY_ELEMENTS).as_dims().map(Vec::from),
            )
            ),*]);

    quote! {
        let mut fields = vec![];
        let mut offset = 0;
        for (name, basetype, arraydims) in #fieldinfo {
            let nelems = arraydims.as_ref().map_or(1, |x| x.iter().copied().product());
            let thissize = basetype.size() * nelems;
            fields.push(CompoundTypeField {
                name,
                offset,
                basetype,
                arraydims,
            });
            offset += thissize;
        }
        let compound = NcVariableType::Compound(CompoundType {
            name: (#ncname).to_owned(),
            size: offset,
            fields,
        });
        assert_eq!(compound.size(), std::mem::size_of::<#ty>(), "Compound must be packed");
        compound
    }
}

fn impl_enum<'a>(
    // ty: &Ident,
    ncname: &str,
    basetyp: &Ident,
    fields: impl Iterator<Item = &'a Variant>,
) -> TokenStream {
    let mut fieldnames = vec![];
    let mut fieldvalues = vec![];

    for field in fields {
        let ident = field.ident.clone();
        let mut rename = None;
        for attr in &field.attrs {
            if attr.path().is_ident("netcdf") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("rename") {
                        rename = Some(meta.value()?.parse::<LitStr>()?.value());
                    } else {
                        abort!(meta.path, "NcType encountered an unknown attribute")
                    }
                    Ok(())
                })
                .unwrap();
            }
        }
        let name = rename.unwrap_or_else(|| ident.to_string());
        fieldnames.push(name);

        let variant = match field.discriminant.clone() {
            Some((_, x)) => quote!(#x),
            None => fieldvalues
                .last()
                .map(|e| quote!(#e + 1))
                .unwrap_or(quote!(0)),
        };

        fieldvalues.push(variant);
    }

    let fieldnames = quote!(vec![#(#fieldnames),*]);
    let fieldvalues = quote!(vec![#(#fieldvalues),*]);

    quote! {
        NcVariableType::Enum(EnumType {
            name: (#ncname).to_owned(),
            fieldnames: (#fieldnames).iter().map(|x| x.to_string()).collect(),
            fieldvalues: ((#fieldvalues).into_iter().collect::<Vec::<#basetyp>>()).into(),
        })
    }
}
