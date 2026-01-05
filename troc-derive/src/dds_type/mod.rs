use proc_macro::TokenStream;
use quote::*;
use syn::{
    Data::Struct, DataStruct, DeriveInput, Field, Fields::Named, FieldsNamed, Type, TypePath,
    parse_macro_input,
};

pub fn derive_proc_macro_impl(input: TokenStream) -> proc_macro::TokenStream {
    let DeriveInput {
        attrs: _,
        vis: _,
        ident,
        generics: _,
        data,
    } = parse_macro_input!(input);

    let description = match data {
        Struct(my_struct) => gen_description_str_for_struct(my_struct),
        _ => panic!("Only Structures are supported"),
    };

    let impl_quote = if description.is_empty() {
        quote! {
            Ok([0u8; 16])
        }
    } else {
        let mut serialization_statements = proc_macro2::TokenStream::new();

        for (var_name, _type_name) in description {
            let var_id = format_ident!("{}", var_name);
            let var_id_ser = format_ident!("{}_ser", var_id);

            let serialization_statement = quote! {
                let #var_id_ser = cdr::serialize::<_, _, cdr::CdrBe>(&self.#var_id, cdr::Infinite).map_err(|e| KeyCalculationError::new(&format!("Error while serializing `{}`", #var_name), e.into()))?;
                // &#var_id_ser[4..] because we don't want to write the CDR Header
                cursor.write_all(&#var_id_ser[4..]).map_err(|e| KeyCalculationError::new(&format!("Error while writing serialized `{}`", #var_name), e.into()))?;
                let pad = calculate_pad(cursor.position() as usize);
                let pad = vec![0u8; pad];
                cursor.write_all(&pad).unwrap();
            };

            serialization_statements.extend(serialization_statement);
        }

        quote! {
            use std::io::Write;

            fn calculate_pad(len: usize) -> usize {
                (4 - (len % 4)) % 4
            }

            let buffer = Vec::new();
            let mut cursor = std::io::Cursor::new(buffer);

            #serialization_statements

            let key = match cursor.position() as usize {
                pos @ ..=16 => {
                    // PADDING
                    let pad_needed = vec![0u8; 16 - pos];
                    cursor.write_all(&pad_needed).map_err(|e| KeyCalculationError::new(&format!("Error while writing serialized `{}`", "placeholder"), e.into()))?;
                    let buffer = cursor.into_inner();
                    let buffer: [u8; 16] = buffer[0..16].try_into().expect("Buffer should be 16 bytes long");
                    buffer
                }
                17.. => {
                    // MD5
                    let buffer = cursor.into_inner();
                    let digest = md5::compute(&buffer);
                    digest.0
                }
            };

            Ok(key)
        }
    };

    let quote = quote! {
        impl Keyed for #ident {
            fn key(&self) -> Result<[u8; 16], KeyCalculationError> {
                #impl_quote
            }
        }
    };

    quote.into()
}

fn gen_description_str_for_struct(my_struct: DataStruct) -> Vec<(String, String)> {
    match my_struct.fields {
        Named(fields) => handle_named_fields(fields),
        _ => panic!("Tuple or fieldless structures are not supported"),
    }
}

fn handle_named_fields(fields: FieldsNamed) -> Vec<(String, String)> {
    fields
        .named
        .into_iter()
        .filter(has_key_attribute)
        .map(|f| {
            let ident = f.ident.clone().unwrap().to_string();
            match &f.ty {
                Type::Path(TypePath { path, .. })
                    if path.is_ident("u8")
                        || path.is_ident("i8")
                        || path.is_ident("u16")
                        || path.is_ident("i16")
                        || path.is_ident("u32")
                        || path.is_ident("i32")
                        || path.is_ident("u64")
                        || path.is_ident("i64")
                        || path.is_ident("bool")
                        || path.is_ident("String") =>
                {
                    (ident, path.get_ident().unwrap().to_string())
                }
                _ => panic!("Type not supported"),
            }
        })
        .collect::<Vec<_>>()
}

fn has_key_attribute(field: &Field) -> bool {
    for attr in field.attrs.iter() {
        let path = attr.path();
        if let Some(path_ident) = path.get_ident()
            && path_ident == "key"
        {
            return true;
        }
    }
    false
}
