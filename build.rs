use std::path::Path;

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

enum ConstGeneric {
    None,
    Request,
    Response,
}

fn main() {
    // let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let out_dir = "src";
    let dest_path = Path::new(&out_dir).join("pdu.rs");

    let mut output = quote! {
        use crate::Frame;

        use zerocopy_derive::*;
    };

    define_function(&mut output, 0x01, "ReadCoils", ConstGeneric::Response);
    define_function(
        &mut output,
        0x02,
        "ReadDiscreteInputs",
        ConstGeneric::Response,
    );
    define_function(
        &mut output,
        0x03,
        "ReadHoldingRegisters",
        ConstGeneric::Response,
    );
    define_function(
        &mut output,
        0x04,
        "ReadInputRegisters",
        ConstGeneric::Response,
    );

    std::fs::write(&dest_path, format_rust(output)).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}

fn define_function(
    output: &mut TokenStream,
    code: u8,
    name: &'static str,
    const_generic: ConstGeneric,
) {
    let name = format_ident!("{name}");
    let builder_name = format_ident!("{name}Builder");

    let (generic_decl, generic_use) = match const_generic {
        ConstGeneric::Response => (quote! { <const N: usize> }, quote! { <N> }),
        _ => (quote! {}, quote! {}),
    };

    output.extend(quote! {
        #[derive(Debug, Clone, FromBytes, IntoBytes, Immutable, Unaligned)]
        #[repr(C)]
        pub struct #name #generic_decl {
            function_code: u8,
        }

        impl #generic_decl #name #generic_use {
            pub const FUNCTION_CODE: u8 = #code;

            pub const fn new() -> Self {
                Self { function_code: Self::FUNCTION_CODE }
            }
        }

        pub struct #builder_name #generic_decl (Frame<#name #generic_use>);

        impl #generic_decl #builder_name #generic_use {
            pub const fn new(server_address: u8) -> Self {
                Self(Frame::new(server_address, <#name #generic_use>::new()))
            }
        }
    });
}

fn format_rust(contents: impl ToTokens) -> String {
    let contents = syn::parse2(contents.to_token_stream()).expect("unable to parse tokens");
    prettyplease::unparse(&contents)
}
