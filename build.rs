use std::path::Path;

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::Ident;

trait Payload {
    fn const_generic(&self) -> bool {
        false
    }

    fn fields(&self) -> Vec<Field>;

    fn data_field(&self) -> Option<Field> {
        self.fields().into_iter().find(|f| f.is_data)
    }

    fn generic_decl(&self) -> TokenStream {
        if self.const_generic() {
            quote! { <const N: usize> }
        } else {
            quote! {}
        }
    }

    fn generic_use(&self) -> TokenStream {
        if self.const_generic() {
            quote! { <N> }
        } else {
            quote! {}
        }
    }

    fn validate_against_counterpart(&self, _counterpart: Ident) -> TokenStream {
        quote! { true }
    }
}

struct FieldBuilder {
    name: Ident,
    ty: TokenStream,
    default_value: TokenStream,
    mutable: bool,
    is_data: bool,
    value_from_const_generic: bool,
    is_be_u16: bool,
}

impl FieldBuilder {
    fn mutable(mut self) -> Self {
        self.mutable = true;
        self
    }

    fn data(mut self) -> Self {
        self.is_data = true;
        self
    }

    fn value_from_const_generic(mut self) -> Self {
        self.value_from_const_generic = true;
        self
    }

    fn build(self) -> Field {
        Field {
            name: self.name,
            ty: self.ty,
            default_value: self.default_value,
            mutable: self.mutable,
            is_data: self.is_data,
            value_from_const_generic: self.value_from_const_generic,
            is_be_u16: self.is_be_u16,
        }
    }
}

struct Field {
    name: Ident,
    ty: TokenStream,
    default_value: TokenStream,
    mutable: bool,
    is_data: bool,
    value_from_const_generic: bool,
    is_be_u16: bool,
}

impl Field {
    fn builder(name: Ident, ty: TokenStream, default_value: TokenStream) -> FieldBuilder {
        FieldBuilder {
            name,
            ty,
            default_value,
            mutable: false,
            is_data: false,
            value_from_const_generic: false,
            is_be_u16: false,
        }
    }

    fn be_u16(name: Ident) -> FieldBuilder {
        let mut builder = Self::builder(
            name,
            quote! { big_endian::U16 },
            quote! { big_endian::U16::ZERO },
        );
        builder.is_be_u16 = true;
        builder
    }
}

struct MultiReadRequest;

impl Payload for MultiReadRequest {
    fn fields(&self) -> Vec<Field> {
        vec![
            Field::be_u16(format_ident!("starting_register"))
                .mutable()
                .build(),
            Field::be_u16(format_ident!("n_registers"))
                .mutable()
                .value_from_const_generic()
                .build(),
        ]
    }
}

struct MultiReadResponse;

impl Payload for MultiReadResponse {
    fn const_generic(&self) -> bool {
        true
    }

    fn fields(&self) -> Vec<Field> {
        vec![
            Field::builder(
                format_ident!("byte_count"),
                quote! { u8 },
                quote! { 2 * N as u8 },
            )
            .build(),
            Field::builder(
                format_ident!("data"),
                quote! { [big_endian::U16; N] },
                quote! { [big_endian::U16::ZERO; N] },
            )
            .mutable()
            .data()
            .build(),
        ]
    }

    fn validate_against_counterpart(&self, counterpart: Ident) -> TokenStream {
        quote! {
            self.byte_count as u16 == 2 * #counterpart.n_registers.get()
        }
    }
}

fn single_write_fields() -> Vec<Field> {
    vec![
        Field::be_u16(format_ident!("register")).mutable().build(),
        Field::be_u16(format_ident!("value")).mutable().build(),
    ]
}

struct SingleWriteRequest;

impl Payload for SingleWriteRequest {
    fn fields(&self) -> Vec<Field> {
        single_write_fields()
    }
}

struct SingleWriteResponse;

impl Payload for SingleWriteResponse {
    fn fields(&self) -> Vec<Field> {
        single_write_fields()
    }

    fn validate_against_counterpart(&self, counterpart: Ident) -> TokenStream {
        quote! {
            self.register == #counterpart.register
                && self.value == #counterpart.value
        }
    }
}

struct MultiWriteRequest;

impl Payload for MultiWriteRequest {
    fn const_generic(&self) -> bool {
        true
    }

    fn fields(&self) -> Vec<Field> {
        vec![
            Field::be_u16(format_ident!("starting_register"))
                .mutable()
                .build(),
            Field::builder(
                format_ident!("n_registers"),
                quote! { big_endian::U16 },
                quote! { big_endian::U16::new(N as u16) },
            )
            .build(),
            Field::builder(
                format_ident!("data_bytes"),
                quote! { u8 },
                quote! { 2 * N as u8 },
            )
            .build(),
            Field::builder(
                format_ident!("data"),
                quote! { [big_endian::U16; N] },
                quote! { [big_endian::U16::ZERO; N] },
            )
            .mutable()
            .data()
            .build(),
        ]
    }
}

struct MultiWriteResponse;

impl Payload for MultiWriteResponse {
    fn fields(&self) -> Vec<Field> {
        vec![
            Field::be_u16(format_ident!("starting_register"))
                .mutable()
                .build(),
            Field::be_u16(format_ident!("quantity")).mutable().build(),
        ]
    }

    fn validate_against_counterpart(&self, counterpart: Ident) -> TokenStream {
        quote! {
            self.starting_register == #counterpart.starting_register
                && self.quantity.get() == N as u16
        }
    }
}

struct Modules {
    pdu_req: TokenStream,
    pdu_res: TokenStream,
    client: TokenStream,
}

impl Modules {
    fn new() -> Self {
        let pdu_scaffold = quote! {
            use super::Response;

            use zerocopy_derive::*;
            use zerocopy::big_endian;
        };

        Self {
            pdu_req: pdu_scaffold.clone(),
            pdu_res: pdu_scaffold.clone(),
            client: quote! {
                use zerocopy::big_endian;
                use embedded_io_async::{Read, Write};

                use super::*;
                use crate::{pdu::{request, response}, Frame};
            },
        }
    }

    fn define_function(&mut self, code: u8, name: &str, req: &impl Payload, res: &impl Payload) {
        let name = format_ident!("{name}");

        self.pdu_req.extend(define_pdu(code, name.clone(), req));
        self.pdu_res.extend(define_pdu(code, name.clone(), res));
        self.pdu_res.extend(impl_response(name.clone(), req, res));
        self.client.extend(define_client_impl(code, name, req, res));
    }

    fn write_to_dir(&self, dir: &Path) {
        write_rust(dir.join("pdu_req.rs"), &self.pdu_req);
        write_rust(dir.join("pdu_res.rs"), &self.pdu_res);
        write_rust(dir.join("client.rs"), &self.client);
    }
}

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();

    let mut modules = Modules::new();

    modules.define_function(0x01, "ReadCoils", &MultiReadRequest, &MultiReadResponse);
    modules.define_function(
        0x02,
        "ReadDiscreteInputs",
        &MultiReadRequest,
        &MultiReadResponse,
    );
    modules.define_function(0x03, "ReadHoldings", &MultiReadRequest, &MultiReadResponse);
    modules.define_function(0x04, "ReadInputs", &MultiReadRequest, &MultiReadResponse);
    modules.define_function(
        0x06,
        "WriteHolding",
        &SingleWriteRequest,
        &SingleWriteResponse,
    );
    modules.define_function(
        0x10,
        "WriteHoldings",
        &MultiWriteRequest,
        &MultiWriteResponse,
    );

    modules.write_to_dir(Path::new(&out_dir));

    println!("cargo:rerun-if-changed=build.rs");
}

fn define_pdu(code: u8, name: Ident, payload: &impl Payload) -> TokenStream {
    let generic_decl = payload.generic_decl();
    let generic_use = payload.generic_use();

    let struct_doc = format!("`{name}` PDU (function code `0x{code:02X}`).");
    let new_doc = format!("Creates a new [`{name}`] with default field values.");

    let mut payload_fields = TokenStream::new();
    let mut default_payload_fields = TokenStream::new();
    let mut payload_methods = TokenStream::new();

    for field in payload.fields() {
        let name = field.name;
        let ty = field.ty;
        let default = field.default_value;

        payload_fields.extend(quote! { pub(crate) #name: #ty, });
        default_payload_fields.extend(quote! { #name: #default, });

        let get_doc = format!("Returns a reference to `{name}`.");
        payload_methods.extend(quote! {
            #[doc = #get_doc]
            pub const fn #name(&self) -> &#ty { &self.#name }
        });

        if field.mutable {
            let set_name = format_ident!("set_{name}");
            let with_name = format_ident!("with_{name}");
            let name_mut = format_ident!("{name}_mut");

            let set_doc = format!("Sets `{name}`.");
            let with_doc = format!("Sets `{name}`, returning `self`.");
            let mut_doc = format!("Returns a mutable reference to `{name}`.");

            let (param_ty, convert) = if field.is_be_u16 {
                (quote! { u16 }, quote! { big_endian::U16::new(new) })
            } else {
                (quote! { #ty }, quote! { new })
            };

            payload_methods.extend(quote! {
                #[doc = #set_doc]
                pub const fn #set_name(&mut self, new: #param_ty) -> &mut Self { self.#name = #convert; self }

                #[doc = #with_doc]
                pub const fn #with_name(mut self, new: #param_ty) -> Self { self.#name = #convert; self }

                #[doc = #mut_doc]
                pub const fn #name_mut(&mut self) -> &mut #ty { &mut self.#name }
            });
        }
    }

    quote! {
        #[doc = #struct_doc]
        #[derive(Debug, Clone, FromBytes, IntoBytes, Immutable, Unaligned)]
        #[repr(C)]
        pub struct #name #generic_decl {
            function_code: u8,
            #payload_fields
        }

        impl #generic_decl #name #generic_use {
            #[doc = #new_doc]
            pub const fn new() -> Self {
                Self {
                    function_code: <Self as crate::Pdu>::FUNCTION_CODE,
                    #default_payload_fields
                }
            }

            #payload_methods
        }

        impl #generic_decl crate::Pdu for #name #generic_use {
            const FUNCTION_CODE: u8 = #code;

            const DEFAULT: Self = Self::new();
        }

        impl #generic_decl Default for #name #generic_use {
            fn default() -> Self {
                crate::Pdu::DEFAULT
            }
        }
    }
}

fn impl_response(name: Ident, req: &impl Payload, res: &impl Payload) -> TokenStream {
    let generics_decl = if req.const_generic() || res.const_generic() {
        quote! { <const N: usize> }
    } else {
        quote! {}
    };

    let req_gu = req.generic_use();
    let res_gu = res.generic_use();
    let req_name = quote! { #name #req_gu };
    let res_name = quote! { #name #res_gu };

    let (data_ty, data_access) = if let Some(f) = res.data_field() {
        let name = f.name;
        (f.ty, quote! { self.#name })
    } else {
        (quote! { () }, quote! {})
    };

    let fn_body = res.validate_against_counterpart(format_ident!("req"));

    quote! {
        impl #generics_decl Response<super::request::#req_name> for #res_name {
            type Data = #data_ty;

            fn matches_request(&self, req: &super::request::#req_name) -> bool {
                #fn_body
            }

            fn into_data(self) -> Self::Data {
                #data_access
            }
        }
    }
}

fn define_client_impl(
    code: u8,
    name: Ident,
    req: &impl Payload,
    res: &impl Payload,
) -> TokenStream {
    let snake_name = format_ident!("{}", pascal_to_snake(&name.to_string()));
    let fn_doc = format!("Executes a Modbus `{name}` request (function code `0x{code:02X}`).");
    let name = format_ident!("{}", name);

    let generics_decl = if req.const_generic() || res.const_generic() {
        quote! { <const N: usize, E> }
    } else {
        quote! { <E> }
    };

    let req_gu = req.generic_use();
    let res_gu = res.generic_use();
    let req_pdu = quote! { request::#name #req_gu };
    let res_pdu = quote! { response::#name #res_gu };

    let res_ty = res.data_field().map_or(quote! { () }, |f| f.ty);
    let mut args = quote! {
        mut serial: impl Read<Error = E> + Write<Error = E>,
        address: u8,
    };
    let mut set_calls = quote! {};

    for field in req.fields() {
        if field.mutable {
            let name = field.name;
            let ty = field.ty;
            let set_name = format_ident!("set_{name}");
            let value = if field.value_from_const_generic {
                quote! { N as u16 }
            } else {
                quote! { #name }
            };

            set_calls.extend(quote! { req.pdu_mut().#set_name(#value); });

            if !field.value_from_const_generic {
                let arg_ty = if field.is_be_u16 {
                    quote! { u16 }
                } else {
                    ty
                };
                args.extend(quote! { #name: #arg_ty, });
            }
        }
    }

    quote! {
        #[doc = #fn_doc]
        pub async fn #snake_name #generics_decl (#args) -> Result<#res_ty, Error<E>> {
            let mut req = Frame::<#req_pdu>::builder(address);
            #set_calls
            let req = req.build_ref();
            write_frame(&mut serial, req).await.map_err(Error::Io)?;

            let res: Frame::<#res_pdu> = read_frame(&mut serial).await?;
            Ok(res.into_data(req)?)
        }
    }
}

fn pascal_to_snake(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for (i, c) in s.char_indices() {
        if c.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

fn write_rust(path: impl AsRef<Path>, contents: impl ToTokens) {
    let contents = syn::parse2(contents.to_token_stream()).expect("unable to parse tokens");
    let formatted = prettyplease::unparse(&contents);

    std::fs::write(path, formatted).expect("unable to write file");
}
