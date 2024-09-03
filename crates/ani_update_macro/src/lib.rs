extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::{Parse, ParseStream}, parse_macro_input, Ident, LitStr, Token};


struct LangStruct {
    id: Ident,
    en: LitStr,
    zh: LitStr,
}

impl Parse for LangStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let id = input.parse::<Ident>()?;
        input.parse::<Token![=>]>()?;
        let en = input.parse::<LitStr>()?;
        input.parse::<Token![,]>()?;
        let zh = input.parse::<LitStr>()?;
        Ok(Self { id, en, zh })
    }
}

#[proc_macro]
pub fn invoke(input: TokenStream) -> TokenStream {
    let LangStruct { id, en, zh } = parse_macro_input!(input as LangStruct);

    let en_str = en.value();
    let zh_str = zh.value();
    let expanded = quote! {
        #[macro_export]
        macro_rules! #id {
            ($( $tt:tt )*) => {
                match crate::lang::Language::get() {
                    crate::lang::Language::En => format!(#en_str, $( $tt )*),
                    crate::lang::Language::Zh => format!(#zh_str, $( $tt )*),
                }
            };
        }
    };
    expanded.into()
}