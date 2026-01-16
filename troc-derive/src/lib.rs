mod dds_type;

use proc_macro::TokenStream;

// TODO: better handle the error
// https://stackoverflow.com/questions/54392702/how-to-report-errors-in-a-procedural-macro-using-the-quote-macro
#[proc_macro_derive(DDSType, attributes(key))]
pub fn dds_type_derive(input: TokenStream) -> TokenStream {
    dds_type::derive_proc_macro_impl(input)
}
