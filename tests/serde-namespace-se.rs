use quick_xml::se::to_string;
use quick_xml::se::serl::expand_derive_serialize;
use serde::{Serialize, Deserialize};
use regex::Regex;
use std::borrow::Cow;
use pretty_assertions::assert_eq;
use std::str::FromStr;
use proc_macro2::{TokenStream, TokenTree, Literal, Ident};
use quote::{format_ident, quote, quote_spanned};
use syn::{ItemStruct, DeriveInput};
use itertools::Itertools;

#[test]
fn test_nested() {
    let foo_struct_tokens = quote! {
        #[derive(Serialize, Deserialize)]
        #[xmlns:B="foourn"]
        #[xmlns="asdfasdf"]
        #[xmlns:C="http://asdfasdfasdf.com/dddddd"]
        #[xmlns=""]
        struct Foo {
            #[xmlpre:B]
            id: String,
            #[serde(flatten)]
            bar: Bar
        }
    };
    let bar_struct_tokens = quote! {
        #[xmlns:F="barurn"]
        struct Bar {
            #[xmlpre:F]
            name: String,
            #[xmlpre:F]
            desc: String
        }
    };
    let ast: ItemStruct = syn::parse2(foo_struct_tokens).unwrap();
    let attrs = &ast.attrs;
    for attr in attrs.iter().filter(|a| a.path.is_ident("xmlns")) {
        let mut attr_token_trees = attr.clone().tokens.into_iter();
        let first_o: Option<TokenTree> = attr_token_trees.next();
        let second_o: Option<TokenTree> = first_o.as_ref().and_then(|_| attr_token_trees.next());
        let third_o: Option<TokenTree> = second_o.as_ref().and_then(|_| attr_token_trees.next());
        let fourth_o: Option<TokenTree> = third_o.as_ref().and_then(|_| attr_token_trees.next());
        if attr_token_trees.next().is_some() {
            panic!("xmlns improprely formatted");
        }

        let first = match first_o {
            Some(TokenTree::Punct(first)) => first,
            _ => {
                panic!("xmlns improperly formatted");
            }
        };
        
        let mut prefix: Option<String> = None;
        let mut uri: Option<String> = None;

        if first.to_string() == ":" {

            let second = match second_o {
                Some(TokenTree::Ident(ref second)) => second,
                _ => {
                    panic!("xmlns improperly formatted");
                }
            };
            prefix = Some(second.to_string());
            let third = match third_o {
                Some(TokenTree::Punct(ref third)) => third,
                _ => {
                    panic!("xmlns improperly formatted");
                }
            };
            if third.to_string() != "=" {
                panic!("xmlns improperly formatted");
            }
            let fourth = match fourth_o {
                Some(TokenTree::Literal(ref fourth)) => fourth,
                _ => {
                    panic!("xmlns improperly formatted");
                }
            };
            uri = Some(fourth.to_string());
        }
        
        if first.to_string() == "=" {
            if third_o.is_some() {
                panic!("xmlns improperly formatted");
            }
            let second = match second_o {
                Some(TokenTree::Literal(ref second)) => second,
                _ => {
                    panic!("xmlns improperly formatted");
                }
            };
            uri = Some(second.to_string());
        }

        println!("{:?} {:?}", prefix, uri);
    }
}
#[derive(Serialize, Deserialize)]
struct FakeRoot {
    root: Root
}
#[derive(Serialize, Deserialize)]
struct Root {
    foo: Foo
}

#[derive(Serialize, Deserialize)]
struct Foo {
  id: String,
  #[serde(flatten)]
  bar: Bar,
  bar2: UnflattenedBar
}

#[derive(Serialize, Deserialize)]
struct Bar {
  name: String,
  desc: String
}

#[derive(Serialize, Deserialize)]
struct UnflattenedBar {
    thing1: String,
    thing2: u32
}

#[test]
fn test_nested2() {
    let doc = FakeRoot {
        root: Root {
            foo: Foo {
                id: "123".to_string(),
                bar: Bar { 
                    name: "asdf".to_string(), 
                    desc: "foobar".to_string() 
                },
                bar2: UnflattenedBar { 
                    thing1: "asdf".to_string(), 
                    thing2: 43
                }
            }
        }
    };
    let xml = quick_xml::se::to_string(&doc).unwrap();
    let str = r#"
<F:foo xmlns:B="foourn" xmlns:F="barurn">
      <F:id>123</F:id>
      <B:name>asdf</B:name>
      <B:desc>foobar </B:desc>
</F:foo>
"#;
    assert_eq!(xml, inline(str));
}

fn inline(str: &str) -> Cow<str> {
    let regex = Regex::new(r">\s+<").unwrap();
    regex.replace_all(str, "><")
}

#[test]
fn test_fake_serde() {
    let foo_struct_tokens = quote! {
        #[derive(Serialize, Deserialize)]
        struct Foo {
            id: String,
            #[serde(flatten)]
            bar: Bar
        }
    };
    let bar_struct_tokens = quote! {
        struct Bar {
            name: String,
            desc: String
        }
    };
    let mut input: DeriveInput = syn::parse2(foo_struct_tokens).unwrap();
    let result = expand_derive_serialize(&mut input)
        .unwrap_or_else(to_compile_errors);
    println!("{}", result.to_string());
    println!("hello world!");
}


fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}
