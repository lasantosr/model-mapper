//! Integration tests verifying custom mapping functions and expressions.

#![allow(dead_code, unused, clippy::restriction, reason = "test")]

use model_mapper::Mapper;

// ====================================================================================================================
// Custom mapping properties: `with`, `from_with`, `into_with` expressions
// ====================================================================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct Foo {
    field1: i32,
    field2: String,
    field3: i32,
}

#[derive(Mapper, Debug, PartialEq)]
#[mapper(from, ty = Foo)]
struct Bar {
    #[mapper(with = field1 + 1)]
    field1: i32,
    #[mapper(with = String::len)]
    field2: usize,
    #[mapper(with = i32::abs)]
    field3: i32,
}

#[derive(Mapper, Debug, PartialEq)]
#[mapper(from, into, ty = Foo)]
struct BarMultiple {
    #[mapper(from_with = field1 + 1, into_with = field1 - 1)]
    field1: i32,
    field2: String,
    #[mapper(with = 0 - field3)]
    field3: i32,
}

#[test]
fn test_custom_properties_basic() {
    let foo = Foo {
        field1: 1,
        field2: "hello".to_string(),
        field3: -10,
    };

    let bar = Bar::from(foo.clone());
    assert_eq!(bar.field1, 2);
    assert_eq!(bar.field2, 5);
    assert_eq!(bar.field3, 10);

    let multiple = BarMultiple::from(foo.clone());
    assert_eq!(multiple.field1, 2);
    assert_eq!(multiple.field2, "hello");
    assert_eq!(multiple.field3, 10);

    let back_to_foo: Foo = multiple.into();
    assert_eq!(back_to_foo, foo);
}

// ====================================================================================================================
// Reference vs Value trait dispatch
// ====================================================================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct DispatchSource {
    text1: String,
    text2: String,
    text3: String,
    number: i32,
}

fn own_string(s: String) -> String {
    s.to_uppercase()
}

#[expect(clippy::ptr_arg, reason = "signature expected by macro")]
fn ref_string(s: &String) -> usize {
    s.len()
}

#[derive(Mapper, Debug, PartialEq, Eq)]
#[mapper(from, ty = DispatchSource)]
struct DispatchTarget {
    #[mapper(with = String::len)]
    text1: usize,

    #[mapper(with = ref_string)]
    text2: usize,

    #[mapper(with = own_string)]
    text3: String,

    #[mapper(with = i32::abs)]
    number: i32,
}

#[test]
fn test_ref_vs_value_dispatch() {
    let source = DispatchSource {
        text1: "hello".to_string(),
        text2: "world".to_string(),
        text3: "rust".to_string(),
        number: -5,
    };
    let target = DispatchTarget::from(source);
    assert_eq!(target.text1, 5);
    assert_eq!(target.text2, 5);
    assert_eq!(target.text3, "RUST");
    assert_eq!(target.number, 5);
}

// ====================================================================================================================
// Cross-field expression scope
// ====================================================================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct SimplePerson {
    name: String,
    age: i32,
}

#[derive(Mapper, Debug, PartialEq, Eq)]
#[mapper(from, into, ty = SimplePerson)]
struct ComplexPerson {
    // The variables in scope for custom mapping expressions correspond directly
    // to the fields of the source/input struct of the mapping (i.e. `name` and `age`
    // for `from`, and `description` and `age` for `into`).
    #[mapper(
        rename = name,
        from_with = format!("{name} (age {age})"),
        into_with = description.strip_suffix(&format!(" (age {age})")).unwrap_or(&description).to_string()
    )]
    description: String,
    age: i32,
}

#[test]
fn test_cross_field_expression() {
    // Test a perfect bidirectional roundtrip
    let source = SimplePerson {
        name: "Alice".to_string(),
        age: 25,
    };

    let target = ComplexPerson::from(source.clone());
    assert_eq!(target.description, "Alice (age 25)");
    assert_eq!(target.age, 25);

    let source_back: SimplePerson = target.into();
    assert_eq!(source_back, source);
}

// ====================================================================================================================
// Custom function name generation
// ====================================================================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct SimpleType {
    x: i64,
}

pub mod service {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct UpdateUserInput {
        pub user_id: i64,
        pub name: String,
    }
}

#[derive(Mapper, Debug, PartialEq, Eq)]
#[mapper(from(custom), ty = SimpleType)]
struct CustomFromName {
    x: i64,
}

#[derive(Mapper, Debug, PartialEq, Eq)]
#[mapper(into(custom), ty = SimpleType)]
struct CustomIntoName {
    x: i64,
}

#[derive(Debug, thiserror::Error)]
enum TestError {
    #[error("error")]
    Err(#[from] std::num::TryFromIntError),
    #[error("infallible")]
    Infallible,
}

impl From<std::convert::Infallible> for TestError {
    fn from(value: std::convert::Infallible) -> Self {
        match value {}
    }
}

#[derive(Mapper, Debug, PartialEq, Eq)]
#[mapper(try_from(custom, err = TestError), ty = SimpleType)]
struct CustomTryFromName {
    x: i32,
}

#[derive(Mapper, Debug, PartialEq, Eq)]
#[mapper(try_into(custom, err = TestError), ty = SimpleType)]
struct CustomTryIntoName {
    x: i32,
}

#[derive(Mapper, Debug, PartialEq, Eq)]
#[mapper(into(custom), ty = "service::UpdateUserInput", add(field = user_id, ty = i64))]
struct CustomNamespacedName {
    #[mapper(rename = name)]
    username: String,
}

#[test]
fn test_custom_function_name_generation() {
    let src = SimpleType { x: 42 };
    let target = CustomFromName::from_simple_type(src.clone());
    assert_eq!(target.x, 42);

    let target_into = CustomIntoName { x: 100 };
    let src_out = target_into.into_simple_type();
    assert_eq!(src_out.x, 100);

    let target_try_from =
        CustomTryFromName::try_from_simple_type(src.clone()).expect("custom try_from conversion should succeed");
    assert_eq!(target_try_from.x, 42);

    let target_try_into = CustomTryIntoName { x: 200 };
    let src_try_out = target_try_into
        .try_into_simple_type()
        .expect("custom try_into conversion should succeed");
    assert_eq!(src_try_out.x, 200);

    let namespaced = CustomNamespacedName {
        username: "bob".to_string(),
    };
    let user_input = namespaced.into_service_update_user_input(999);
    assert_eq!(user_input.user_id, 999);
    assert_eq!(user_input.name, "bob");
}
