//! Integration tests verifying generic mapping features.

#![allow(dead_code, unused, clippy::restriction, clippy::float_cmp, reason = "test")]

use model_mapper::Mapper;

// ====================================================================================================================
// Same Generic Names
// ====================================================================================================================

#[derive(Debug, PartialEq)]
struct SourceSame<T> {
    value: T,
}

#[derive(Mapper, Debug, PartialEq)]
#[mapper(from, into, ty = "SourceSame<T>")]
struct TargetSame<T> {
    value: T,
}

#[test]
fn test_same_generic_names() {
    let src = SourceSame { value: 42i32 };
    let target = TargetSame::<i64>::from(src);
    assert_eq!(target.value, 42i64);

    let target2 = TargetSame { value: 100i32 };
    let src2 = SourceSame::<i64>::from(target2);
    assert_eq!(src2.value, 100i64);
}

// ====================================================================================================================
// Decoupled Generic Mapping
// ====================================================================================================================

#[derive(Debug, PartialEq)]
struct DecoupledSource<A, Z> {
    field1: A,
    field2: i32,
    field3: Z,
}

#[derive(Mapper, Debug, PartialEq)]
#[mapper(from, into, ty = "DecoupledSource<X, Z>")]
struct DecoupledTarget<T, Z> {
    #[mapper(other_ty = X)]
    field1: i64,
    #[mapper(other_ty = i32)]
    field2: T,
    field3: Z,
}

#[test]
fn test_decoupled_generics() {
    let src = DecoupledSource {
        field1: 10i8,
        field2: 42,
        field3: 3.5f32,
    };
    let target = DecoupledTarget::<i64, f64>::from(src);
    assert_eq!(target.field1, 10i64);
    assert_eq!(target.field2, 42i64);
    assert_eq!(target.field3, 3.5f64);

    let target2 = DecoupledTarget {
        field1: 20i64,
        field2: 50i8,
        field3: 2.5f32,
    };
    let src2 = DecoupledSource::<i64, f64>::from(target2);
    assert_eq!(src2.field1, 20i64);
    assert_eq!(src2.field2, 50);
    assert_eq!(src2.field3, 2.5f64);
}

// ====================================================================================================================
// Simple Generic Enums
// ====================================================================================================================

#[derive(Debug, PartialEq)]
enum SimpleEnumSource<T> {
    Variant(T),
}

#[derive(Mapper, Debug, PartialEq)]
#[mapper(from, into, ty = "SimpleEnumSource<T>")]
enum SimpleEnumTarget<T> {
    Variant(T),
}

#[test]
fn test_simple_generic_enums() {
    let src = SimpleEnumSource::Variant(42i32);
    let target = SimpleEnumTarget::<i64>::from(src);
    assert_eq!(target, SimpleEnumTarget::Variant(42i64));

    let target2 = SimpleEnumTarget::Variant(100i32);
    let src2 = SimpleEnumSource::<i64>::from(target2);
    assert_eq!(src2, SimpleEnumSource::Variant(100i64));
}

// ====================================================================================================================
// Complex Generic Enums
// ====================================================================================================================

#[derive(Debug, PartialEq)]
enum ComplexEnumSource<T, U> {
    Pair(T, U),
    Named { x: T, y: U },
    Unit,
}

#[derive(Mapper, Debug, PartialEq)]
#[mapper(from, into, ty = "ComplexEnumSource<T, U>")]
enum ComplexEnumTarget<A, B> {
    Pair(#[mapper(other_ty = T)] A, #[mapper(other_ty = U)] B),
    Named {
        #[mapper(other_ty = T)]
        x: A,
        #[mapper(other_ty = U)]
        y: B,
    },
    Unit,
}

#[test]
fn test_complex_generic_enums() {
    let src_pair = ComplexEnumSource::Pair(10i32, "hello".to_string());
    let target_pair = ComplexEnumTarget::<i64, String>::from(src_pair);
    assert_eq!(target_pair, ComplexEnumTarget::Pair(10i64, "hello".to_string()));

    let target_pair2 = ComplexEnumTarget::Pair(20i32, "world");
    let src_pair2 = ComplexEnumSource::<i64, String>::from(target_pair2);
    assert_eq!(src_pair2, ComplexEnumSource::Pair(20i64, "world".to_string()));

    let src_named = ComplexEnumSource::Named { x: 30i32, y: 4.5f32 };
    let target_named = ComplexEnumTarget::<i64, f64>::from(src_named);
    assert_eq!(target_named, ComplexEnumTarget::Named { x: 30i64, y: 4.5f64 });

    let target_named2 = ComplexEnumTarget::Named { x: 40i32, y: 5.5f32 };
    let src_named2 = ComplexEnumSource::<i64, f64>::from(target_named2);
    assert_eq!(src_named2, ComplexEnumSource::Named { x: 40i64, y: 5.5f64 });

    let src_unit = ComplexEnumSource::<i32, f32>::Unit;
    let target_unit = ComplexEnumTarget::<i64, f64>::from(src_unit);
    assert_eq!(target_unit, ComplexEnumTarget::Unit);
}

// ====================================================================================================================
// Where Bounds Propagation
// ====================================================================================================================

#[derive(Debug, PartialEq)]
struct BoundedTarget<T> {
    val: T,
}

#[derive(Mapper, Debug, PartialEq)]
#[mapper(into, ty = "BoundedTarget<T>")]
struct BoundedSource<T>
where
    T: Clone + Default,
{
    val: T,
}

#[derive(Mapper, Debug, PartialEq)]
#[mapper(from, ty = "BoundedSource2<T>")]
struct BoundedTarget2<T>
where
    T: Clone + Default,
{
    val: T,
}

#[derive(Debug, PartialEq)]
struct BoundedSource2<T> {
    val: T,
}

#[test]
fn test_where_bounds_propagation() {
    let src = BoundedSource { val: 123i32 };
    let target = BoundedTarget::<i32>::from(src);
    assert_eq!(target.val, 123);

    let src2 = BoundedSource2 { val: 456i32 };
    let target2 = BoundedTarget2::<i32>::from(src2);
    assert_eq!(target2.val, 456);
}

// ====================================================================================================================
// Complex and Quoted Namespaces
// ====================================================================================================================

pub mod external_namespace {
    #[derive(Debug, PartialEq)]
    pub struct NamespaceSource<T> {
        pub data: T,
    }
}

pub mod target_namespace {
    use super::Mapper;

    #[derive(Mapper, Debug, PartialEq)]
    #[mapper(from, into, ty = "super::external_namespace::NamespaceSource<T>")]
    pub struct NamespaceTarget<T> {
        pub data: T,
    }
}

#[test]
fn test_complex_namespaces() {
    let src = external_namespace::NamespaceSource { data: 42i32 };
    let target = target_namespace::NamespaceTarget::<i64>::from(src);
    assert_eq!(target.data, 42i64);
}
