//! Integration tests verifying type mapping transform hints.

#![allow(dead_code, unused, clippy::restriction, clippy::struct_field_names, reason = "test")]

use std::collections::{HashMap, HashSet, VecDeque};

use model_mapper::Mapper;

// ====================================================================================================================
// Simple Transform Hints: opt, iter, map, boxed, box, unbox
// ====================================================================================================================

#[derive(Debug, PartialEq)]
struct SimpleSource {
    opt_field: Option<i32>,
    iter_field: Vec<i32>,
    map_field: HashMap<String, i32>,
    boxed_field: Box<i32>,
    box_field: Box<i32>, // target is boxed
    unbox_field: i32,    // target is unboxed
}

#[derive(Mapper, Debug, PartialEq)]
#[mapper(into, from, ty = SimpleSource)]
struct SimpleTarget {
    #[mapper(opt)]
    opt_field: Option<i32>,

    #[mapper(iter)]
    iter_field: VecDeque<i32>,

    #[mapper(map)]
    map_field: HashMap<String, i32>,

    #[mapper(boxed)]
    boxed_field: Box<i32>,

    #[mapper(box)]
    box_field: i32, // source is unboxed

    #[mapper(unbox)]
    unbox_field: Box<i32>, // source is boxed
}

#[test]
fn test_simple_transform_hints() {
    let src = SimpleSource {
        opt_field: Some(42),
        iter_field: vec![1, 2, 3],
        map_field: {
            let mut m = HashMap::new();
            m.insert("hello".to_string(), 100);
            m
        },
        boxed_field: Box::new(200),
        box_field: Box::new(300),
        unbox_field: 400,
    };

    // Test conversion from SimpleSource to SimpleTarget (into / from)
    let target = SimpleTarget::from(src);
    assert_eq!(target.opt_field, Some(42));

    let mut expected_iter = VecDeque::new();
    expected_iter.push_back(1);
    expected_iter.push_back(2);
    expected_iter.push_back(3);
    assert_eq!(target.iter_field, expected_iter);

    assert_eq!(target.map_field.get("hello"), Some(&100));
    assert_eq!(target.boxed_field, Box::new(200));
    assert_eq!(target.box_field, 300);
    assert_eq!(target.unbox_field, Box::new(400));

    // Test round-trip (SimpleTarget back to SimpleSource)
    let src2 = SimpleSource::from(target);
    assert_eq!(src2.opt_field, Some(42));
    assert_eq!(src2.iter_field, vec![1, 2, 3]);
    assert_eq!(src2.map_field.get("hello"), Some(&100));
    assert_eq!(src2.boxed_field, Box::new(200));
    assert_eq!(src2.box_field, Box::new(300));
    assert_eq!(src2.unbox_field, 400);
}

// ====================================================================================================================
// Nested Hints Combinations
// ====================================================================================================================

#[derive(Debug, PartialEq)]
struct NestedSource {
    opt_iter: Option<Vec<i32>>,
    iter_opt: Vec<Option<i32>>,
    opt_map_opt_boxed: Option<HashMap<String, Option<Box<i32>>>>,
    boxed_opt_iter: Box<Option<Vec<i32>>>,
}

#[derive(Mapper, Debug, PartialEq)]
#[mapper(into, from, ty = NestedSource)]
struct NestedTarget {
    #[mapper(opt(iter))]
    opt_iter: Option<VecDeque<i32>>,

    #[mapper(iter(opt))]
    iter_opt: Vec<Option<i32>>,

    #[mapper(opt(map(opt(boxed))))]
    opt_map_opt_boxed: Option<HashMap<String, Option<Box<i32>>>>,

    #[mapper(boxed(opt(iter)))]
    boxed_opt_iter: Box<Option<VecDeque<i32>>>,
}

#[test]
fn test_nested_transform_hints() {
    let src = NestedSource {
        opt_iter: Some(vec![1, 2]),
        iter_opt: vec![Some(10), None, Some(20)],
        opt_map_opt_boxed: {
            let mut inner_map = HashMap::new();
            inner_map.insert("key".to_string(), Some(Box::new(500)));
            inner_map.insert("none_key".to_string(), None);
            Some(inner_map)
        },
        boxed_opt_iter: Box::new(Some(vec![1000])),
    };

    let target = NestedTarget::from(src);

    let mut expected_opt_iter = VecDeque::new();
    expected_opt_iter.push_back(1);
    expected_opt_iter.push_back(2);
    assert_eq!(target.opt_iter, Some(expected_opt_iter));

    assert_eq!(target.iter_opt, vec![Some(10), None, Some(20)]);

    let map = target.opt_map_opt_boxed.as_ref().unwrap();
    assert_eq!(map.get("key").unwrap().as_ref().unwrap(), &Box::new(500));
    assert!(map.get("none_key").unwrap().is_none());

    let mut expected_boxed_opt_iter = VecDeque::new();
    expected_boxed_opt_iter.push_back(1000);
    assert_eq!(target.boxed_opt_iter, Box::new(Some(expected_boxed_opt_iter)));

    // Round-trip back to NestedSource
    let src2 = NestedSource::from(target);
    assert_eq!(src2.opt_iter, Some(vec![1, 2]));
    assert_eq!(src2.iter_opt, vec![Some(10), None, Some(20)]);

    let map2 = src2.opt_map_opt_boxed.as_ref().unwrap();
    assert_eq!(map2.get("key").unwrap().as_ref().unwrap(), &Box::new(500));
    assert!(map2.get("none_key").unwrap().is_none());

    assert_eq!(src2.boxed_opt_iter, Box::new(Some(vec![1000])));
}

// ====================================================================================================================
// Hints combined with Custom Expressions
// ====================================================================================================================

#[derive(Debug, PartialEq)]
struct CustomSource {
    opt_iter_with: Option<Vec<i32>>,
}

#[derive(Mapper, Debug, PartialEq)]
#[mapper(into, ty = CustomSource)]
struct CustomTarget {
    #[mapper(opt(iter(with = opt_iter_with.parse().unwrap())))]
    opt_iter_with: Option<Vec<String>>,
}

#[test]
fn test_hints_with_custom_expressions() {
    let target = CustomTarget {
        opt_iter_with: Some(vec!["123".to_string(), "456".to_string()]),
    };
    let src = CustomSource::from(target);
    assert_eq!(src.opt_iter_with, Some(vec![123, 456]));
}
