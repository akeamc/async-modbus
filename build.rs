use quote::quote;
use std::env;
use std::fs;
use std::path::Path;

/// Represents a field in a message structure
#[derive(Debug, Clone)]
struct Field {
    name: String,
    ty: String,
}

/// Represents a Modbus message specification
#[derive(Debug, Clone)]
struct MessageSpec {
    name: String,
    function_code: u8,
    fields: Vec<Field>,
    has_const_generic: bool,
    const_assertion: Option<String>,
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_messages.rs");

    // Define all message specifications for request types
    let request_messages = vec![
        MessageSpec {
            name: "request_WriteHolding".to_string(),
            function_code: 0x06,
            fields: vec![
                Field {
                    name: "register".to_string(),
                    ty: "big_endian::U16".to_string(),
                },
                Field {
                    name: "value".to_string(),
                    ty: "big_endian::U16".to_string(),
                },
            ],
            has_const_generic: false,
            const_assertion: None,
        },
        MessageSpec {
            name: "request_ReadHoldings".to_string(),
            function_code: 0x03,
            fields: vec![
                Field {
                    name: "starting_register".to_string(),
                    ty: "big_endian::U16".to_string(),
                },
                Field {
                    name: "n_registers".to_string(),
                    ty: "big_endian::U16".to_string(),
                },
            ],
            has_const_generic: false,
            const_assertion: None,
        },
        MessageSpec {
            name: "request_WriteHoldings".to_string(),
            function_code: 0x10,
            fields: vec![
                Field {
                    name: "starting_register".to_string(),
                    ty: "big_endian::U16".to_string(),
                },
                Field {
                    name: "n_registers".to_string(),
                    ty: "big_endian::U16".to_string(),
                },
                Field {
                    name: "data_bytes".to_string(),
                    ty: "u8".to_string(),
                },
                Field {
                    name: "data".to_string(),
                    ty: "[big_endian::U16; N]".to_string(),
                },
            ],
            has_const_generic: true,
            const_assertion: Some(
                "const { assert!(N <= 127, \"N must be less than or equal to 127\") }".to_string(),
            ),
        },
        MessageSpec {
            name: "request_ReadInputs".to_string(),
            function_code: 0x04,
            fields: vec![
                Field {
                    name: "starting_register".to_string(),
                    ty: "big_endian::U16".to_string(),
                },
                Field {
                    name: "n_registers".to_string(),
                    ty: "big_endian::U16".to_string(),
                },
            ],
            has_const_generic: false,
            const_assertion: None,
        },
    ];

    // Define all message specifications for response types
    let response_messages = vec![
        MessageSpec {
            name: "response_WriteHolding".to_string(),
            function_code: 0x06,
            fields: vec![
                Field {
                    name: "register".to_string(),
                    ty: "big_endian::U16".to_string(),
                },
                Field {
                    name: "value".to_string(),
                    ty: "big_endian::U16".to_string(),
                },
            ],
            has_const_generic: false,
            const_assertion: None,
        },
        MessageSpec {
            name: "response_ReadHoldings".to_string(),
            function_code: 0x03,
            fields: vec![
                Field {
                    name: "data_bytes".to_string(),
                    ty: "u8".to_string(),
                },
                Field {
                    name: "data".to_string(),
                    ty: "[big_endian::U16; N]".to_string(),
                },
            ],
            has_const_generic: true,
            const_assertion: Some(
                "const { assert!(N <= 127, \"N must be less than or equal to 127\") }".to_string(),
            ),
        },
        MessageSpec {
            name: "response_WriteHoldings".to_string(),
            function_code: 0x10,
            fields: vec![
                Field {
                    name: "starting_register".to_string(),
                    ty: "big_endian::U16".to_string(),
                },
                Field {
                    name: "n_registers".to_string(),
                    ty: "big_endian::U16".to_string(),
                },
            ],
            has_const_generic: false,
            const_assertion: None,
        },
        MessageSpec {
            name: "response_ReadInputs".to_string(),
            function_code: 0x04,
            fields: vec![
                Field {
                    name: "data_bytes".to_string(),
                    ty: "u8".to_string(),
                },
                Field {
                    name: "data".to_string(),
                    ty: "[big_endian::U16; N]".to_string(),
                },
            ],
            has_const_generic: true,
            const_assertion: Some(
                "const { assert!(N <= 127, \"N must be less than or equal to 127\") }".to_string(),
            ),
        },
    ];

    let mut all_messages = request_messages;
    all_messages.extend(response_messages);

    // Generate code for all messages
    let mut generated = quote! {
        use zerocopy::{IntoBytes, big_endian, little_endian};
        use zerocopy_derive::*;
    };

    for msg in all_messages {
        let code = generate_message(&msg);
        generated.extend(code);
    }

    // Format the generated code
    let syntax_tree = syn::parse_file(&generated.to_string()).unwrap();
    let formatted = prettyplease::unparse(&syntax_tree);

    // Write the generated code to the output file
    fs::write(&dest_path, formatted).unwrap();

    // Tell cargo to rerun this build script if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
}

fn generate_message(spec: &MessageSpec) -> proc_macro2::TokenStream {
    let name_ident = syn::Ident::new(&spec.name, proc_macro2::Span::call_site());
    let function_code = spec.function_code;

    // Generate field definitions
    let field_defs: Vec<_> = spec
        .fields
        .iter()
        .map(|f| {
            let field_name = syn::Ident::new(&f.name, proc_macro2::Span::call_site());
            let field_type: proc_macro2::TokenStream = f.ty.parse().unwrap();
            quote! {
                pub(crate) #field_name: #field_type
            }
        })
        .collect();

    // Generate field names for method parameters
    let field_names: Vec<_> = spec
        .fields
        .iter()
        .map(|f| syn::Ident::new(&f.name, proc_macro2::Span::call_site()))
        .collect();

    // Generate field types for method parameters
    let field_types: Vec<_> = spec
        .fields
        .iter()
        .map(|f| {
            let ty: proc_macro2::TokenStream = f.ty.parse().unwrap();
            ty
        })
        .collect();

    // Generate prologue (const assertion if needed)
    let prologue = if let Some(assertion) = &spec.const_assertion {
        let assertion_tokens: proc_macro2::TokenStream = assertion.parse().unwrap();
        quote! { #assertion_tokens; }
    } else {
        quote! {}
    };

    if spec.has_const_generic {
        // Generate struct with const generic
        quote! {
            #[derive(IntoBytes, Immutable, FromBytes, KnownLayout)]
            #[repr(C)]
            pub struct #name_ident<const N: usize> {
                pub(crate) addr: u8,
                pub(crate) function: u8,
                #(#field_defs,)*
                pub(crate) crc: little_endian::U16,
            }

            impl<const N: usize> #name_ident<N> {
                #[allow(dead_code)]
                pub(crate) const FUNCTION: u8 = #function_code;

                #[allow(dead_code)]
                pub(crate) fn new_inner(addr: u8, #(#field_names: #field_types),*) -> Self {
                    #prologue

                    let mut message = Self {
                        addr,
                        function: #function_code,
                        #(#field_names,)*
                        crc: Default::default(),
                    };

                    message.crc = message.calculate_crc().into();
                    message
                }

                #[allow(dead_code)]
                pub(crate) fn new_with_inner(addr: u8, f: impl FnOnce(&mut Self)) -> Self {
                    #prologue

                    let mut message = <Self as zerocopy::FromZeros>::new_zeroed();
                    message.addr = addr;
                    message.function = #function_code;
                    f(&mut message);
                    message.crc = message.calculate_crc().into();
                    message
                }

                pub(crate) fn calculate_crc(&self) -> u16 {
                    let bytes = self.as_bytes();
                    crate::crc(&bytes[..bytes.len() - 2])
                }

                pub fn validate_crc(&self) -> Result<(), crate::CrcError> {
                    if self.crc.get() == self.calculate_crc() {
                        Ok(())
                    } else {
                        Err(crate::CrcError)
                    }
                }

                pub fn update_crc(&mut self) {
                    self.crc = self.calculate_crc().into();
                }

                pub fn address(&self) -> u8 {
                    self.addr
                }

                pub(crate) fn function(&self) -> u8 {
                    self.function
                }
            }
        }
    } else {
        // Generate struct without const generic
        quote! {
            #[derive(IntoBytes, Immutable, FromBytes, KnownLayout)]
            #[repr(C)]
            pub struct #name_ident {
                pub(crate) addr: u8,
                pub(crate) function: u8,
                #(#field_defs,)*
                pub(crate) crc: little_endian::U16,
            }

            impl #name_ident {
                #[allow(dead_code)]
                pub(crate) const FUNCTION: u8 = #function_code;

                #[allow(dead_code)]
                pub(crate) fn new_inner(addr: u8, #(#field_names: #field_types),*) -> Self {
                    #prologue

                    let mut message = Self {
                        addr,
                        function: #function_code,
                        #(#field_names,)*
                        crc: Default::default(),
                    };

                    message.crc = message.calculate_crc().into();
                    message
                }

                #[allow(dead_code)]
                pub(crate) fn new_with_inner(addr: u8, f: impl FnOnce(&mut Self)) -> Self {
                    #prologue

                    let mut message = <Self as zerocopy::FromZeros>::new_zeroed();
                    message.addr = addr;
                    message.function = #function_code;
                    f(&mut message);
                    message.crc = message.calculate_crc().into();
                    message
                }

                pub(crate) fn calculate_crc(&self) -> u16 {
                    let bytes = self.as_bytes();
                    crate::crc(&bytes[..bytes.len() - 2])
                }

                pub fn validate_crc(&self) -> Result<(), crate::CrcError> {
                    if self.crc.get() == self.calculate_crc() {
                        Ok(())
                    } else {
                        Err(crate::CrcError)
                    }
                }

                pub fn update_crc(&mut self) {
                    self.crc = self.calculate_crc().into();
                }

                pub fn address(&self) -> u8 {
                    self.addr
                }

                pub(crate) fn function(&self) -> u8 {
                    self.function
                }
            }
        }
    }
}
