use syn::DeriveInput;

use crate::model_mapper;

#[test]
fn comprehensive_derive() {
    let input: DeriveInput = syn::parse_quote! {
        #[mapper(
            from(custom),
            into(custom),
            try_from(custom, err = "CustomError"),
            try_into(custom, err = "CustomError"),
            ty = "Target"
        )]
        pub(crate) struct Source {
            pub x: i32,
            pub y: String,
        }
    };
    let expanded = model_mapper::r#impl(input);
    let file = syn::parse2::<syn::File>(expanded).expect("parsed syntax tree should be valid syn::File");
    let formatted = prettyplease::unparse(&file);
    insta::assert_snapshot!(formatted);
}

#[test]
fn hint_collection() {
    let input: DeriveInput = syn::parse_quote! {
        #[mapper(from, into, ty = "Target")]
        pub(crate) struct Source {
            #[mapper(opt)]
            pub a: Option<i32>,
            #[mapper(iter)]
            pub b: Vec<i32>,
            #[mapper(map)]
            pub c: HashMap<String, i32>,
            #[mapper(boxed)]
            pub d: Box<i32>,
            #[mapper(box)]
            pub e: i32,
            #[mapper(unbox)]
            pub f: Box<i32>,
        }
    };
    let expanded = model_mapper::r#impl(input);
    let file = syn::parse2::<syn::File>(expanded).expect("parsed syntax tree should be valid syn::File");
    let formatted = prettyplease::unparse(&file);
    insta::assert_snapshot!(formatted);
}

#[test]
fn deep_nesting() {
    let input: DeriveInput = syn::parse_quote! {
        #[mapper(from, into, ty = "Target")]
        pub(crate) struct Source {
            #[mapper(opt(iter(map(boxed))))]
            pub x: Option<Vec<HashMap<String, Box<i32>>>>,
        }
    };
    let expanded = model_mapper::r#impl(input);
    let file = syn::parse2::<syn::File>(expanded).expect("parsed syntax tree should be valid syn::File");
    let formatted = prettyplease::unparse(&file);
    insta::assert_snapshot!(formatted);
}

#[test]
fn custom_from_with_skipped_fields() {
    let input: DeriveInput = syn::parse_quote! {
        #[mapper(from(custom = "from_target"), ty = "Target")]
        pub(crate) struct Source {
            pub x: i32,
            #[mapper(skip)]
            pub y: String,
        }
    };
    let expanded = model_mapper::r#impl(input);
    let file = syn::parse2::<syn::File>(expanded).expect("parsed syntax tree should be valid syn::File");
    let formatted = prettyplease::unparse(&file);
    insta::assert_snapshot!(formatted);
}
