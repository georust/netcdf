use netcdf::types::*;
use netcdf_derive::NcType;

#[repr(C, packed)]
#[derive(NcType)]
pub struct Bar {
    arr: [u8; 4],
    arr2: [[u16; 1]; 3],
    arr3: [[[f32; 4]; 5]; 10],
}

// mod other {
//     use super::*;

//     #[repr(C, packed)]
//     #[derive(NcType)]
//     #[netcdf(opaque)]
//     pub struct Opaque {
//         a: i32,
//         b: i64,
//         c: [u8; 32],
//     }

//     // #[repr(C)]
//     // #[derive(NcType)]
//     // pub struct WithOpaque {
//     //     #[netcdf(opaque = true)]
//     //     blob: [u8; 1024],
//     // }
// }

// use other::*;

#[test]
fn test_impl_foo() {
    #[repr(C, packed)]
    #[derive(NcType)]
    pub struct Foo {
        a: i32,
        b: i32,
        c: i64,
        dsdfjkj: f64,
    }

    let auto_tp = Foo::type_descriptor();
    let manual_tp = NcVariableType::Compound(CompoundType {
        name: "Foo".to_owned(),
        size: 24,
        fields: vec![
            CompoundTypeField {
                name: "a".to_owned(),
                basetype: i32::type_descriptor(),
                arraydims: None,
                offset: 0,
            },
            CompoundTypeField {
                name: "b".to_owned(),
                basetype: i32::type_descriptor(),
                arraydims: None,
                offset: 4,
            },
            CompoundTypeField {
                name: "c".to_owned(),
                basetype: i64::type_descriptor(),
                arraydims: None,
                offset: 8,
            },
            CompoundTypeField {
                name: "dsdfjkj".to_owned(),
                basetype: f64::type_descriptor(),
                arraydims: None,
                offset: 16,
            },
        ],
    });
    assert_eq!(auto_tp, manual_tp);
}

#[test]
fn test_impl_array() {
    #[repr(C, packed)]
    #[derive(NcType)]
    pub struct Foo {
        a: i32,
        b: [[u8; 4]; 5],
    }

    let auto_tp = Foo::type_descriptor();
    let manual_tp = NcVariableType::Compound(CompoundType {
        name: "Foo".to_owned(),
        size: 4 + 1 * (4 * 5),
        fields: vec![
            CompoundTypeField {
                name: "a".to_owned(),
                basetype: i32::type_descriptor(),
                arraydims: None,
                offset: 0,
            },
            CompoundTypeField {
                name: "b".to_owned(),
                basetype: u8::type_descriptor(),
                arraydims: Some(vec![4, 5]),
                offset: 4,
            },
        ],
    });
    assert_eq!(auto_tp, manual_tp);
}

#[test]
fn test_renamed() {
    #[repr(C)]
    #[derive(NcType)]
    #[netcdf(rename = "renamed")]
    pub struct Renamed {
        #[netcdf(rename = "other")]
        item: u8,
    }

    let manual_tp = NcVariableType::Compound(CompoundType {
        name: "renamed".to_owned(),
        size: 1,
        fields: vec![CompoundTypeField {
            name: "other".to_owned(),
            basetype: u8::type_descriptor(),
            arraydims: None,
            offset: 0,
        }],
    });
    assert_eq!(Renamed::type_descriptor(), manual_tp);
}

#[test]
fn test_impl_nested() {
    #[repr(C, packed)]
    #[derive(NcType)]
    pub struct Foo {
        a: i32,
        b: [[u8; 4]; 5],
    }

    #[repr(C, packed)]
    #[derive(NcType)]
    pub struct FooBar {
        foo: Foo,
    }

    let manual_tp = NcVariableType::Compound(CompoundType {
        name: "FooBar".to_owned(),
        size: 4 + 1 * (4 * 5),
        fields: vec![CompoundTypeField {
            name: "foo".to_owned(),
            basetype: Foo::type_descriptor(),
            arraydims: None,
            offset: 0,
        }],
    });
    assert_eq!(FooBar::type_descriptor(), manual_tp);
}

#[test]
fn test_impl_enum() {
    #[repr(i8)]
    #[derive(NcType)]
    pub enum EnumOne {
        A = 4,
        B = 5,
        C = 7,
        D = 2,
        E = Self::A as i8 - 50,
    }

    let manual_tp = NcVariableType::Enum(EnumType {
        name: "EnumOne".to_owned(),
        fieldnames: vec![
            "A".to_owned(),
            "B".to_owned(),
            "C".to_owned(),
            "D".to_owned(),
            "E".to_owned(),
        ],
        fieldvalues: vec![
            EnumOne::A as i8,
            EnumOne::B as _,
            EnumOne::C as _,
            EnumOne::D as _,
            EnumOne::E as _,
        ]
        .into(),
    });

    assert_eq!(EnumOne::type_descriptor(), manual_tp);

    const VALUE: i64 = 405;

    #[repr(i64)]
    #[derive(NcType)]
    #[allow(unused)]
    pub enum EnumTwo {
        A,
        B,
        C,
        D = 6,
        F = i64::max_value(),
        G = VALUE,
    }
}
