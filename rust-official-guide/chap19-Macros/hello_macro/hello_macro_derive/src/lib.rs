extern crate proc_macro;

use proc_macro::TokenStream; // proc はprocedural macroのこと
use quote::quote; // https://crates.io/crates/quote
use syn; // https://crates.io/crates/syn
// quoteもsynもASTを扱う上で必要なサードパーティ(公式の人がメンテしている)

// Macroを自分で定義するときに必要(proc_macro_derive)
// TokenStreamはASTのことASTを受け取って書き換えて返す→コードを再構成するコードということ
#[proc_macro_derive(HelloMacro)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_hello_macro(&ast)
}

// impl_hello_macro へは以下のようなASTが入力で入る

/*
DeriveInput {
    // --snip--

    ident: Ident {
        ident: "Pancakes",
        span: #0 bytes(95..103)
    },
    data: Struct(
        DataStruct {
            struct_token: Struct,
            fields: Unit,
            semi_token: Some(
                Semi
            )
        }
    )
}
*/

fn impl_hello_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! { // quote!がRustのコードを作る
        impl HelloMacro for #name {
            fn hello_macro() {
                println!("Hello, Macro! My name is {}!", stringify!(#name));
            }
        }
    };
    gen.into()
}
