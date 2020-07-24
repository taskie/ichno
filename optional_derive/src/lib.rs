use darling::{FromDeriveInput, FromField, FromMeta};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields};

#[derive(FromDeriveInput)]
#[darling(attributes(optional), forward_attrs(allow, doc, cfg, table_name))]
struct DeriveInputOption {
    #[darling(default)]
    name: Option<String>,
    #[darling(default)]
    derive: Option<String>,
    attrs: Vec<syn::Attribute>,
}

#[derive(FromField)]
#[darling(attributes(optional), forward_attrs(allow, doc, cfg, table_name))]
struct FieldOption {
    #[darling(default)]
    name: Option<String>,
    #[darling(default)]
    skip: bool,
    #[darling(default)]
    required: bool,
    attrs: Vec<syn::Attribute>,
}

#[proc_macro_derive(Optional, attributes(optional))]
pub fn derive_options(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let option = DeriveInputOption::from_derive_input(&input).unwrap();

    let name = &input.ident;
    let attrs = &option.attrs;
    let vis = &input.vis;
    let generics = &input.generics;

    let derive: Option<&String> = option.derive.as_ref();
    let derives: Vec<syn::Ident> = derive
        .map(|s| s.split(",").map(|s| s.trim()).filter(|s| !s.is_empty()).collect())
        .unwrap_or(vec![])
        .iter()
        .map(|s| syn::Ident::from_string(s).unwrap())
        .collect();

    let new_name = option.name.map(|n| syn::Ident::from_string(&n).unwrap()).unwrap_or(format_ident!("{}Opt", name));
    let new_fields = process_fields(&input.data);

    let expanded = quote! {
        #(#attrs)*
        #[derive(#(#derives, )*)]
        #vis struct #new_name #generics {
            #new_fields
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn process_fields(data: &Data) -> TokenStream {
    match data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let new_fields = fields
                    .named
                    .iter()
                    .map(|f| {
                        let option = FieldOption::from_field(f).unwrap();
                        if option.skip {
                            return None;
                        }
                        let vis = &f.vis;
                        let attrs = &option.attrs;
                        let name = &option.name.map(|n| syn::Ident::from_string(&n).unwrap());
                        let name = name.as_ref().or(f.ident.as_ref());
                        let ty = &f.ty;
                        let new_ty = if option.required {
                            quote! {#ty}
                        } else {
                            quote! {::std::option::Option<#ty>}
                        };
                        Some(quote_spanned! {f.span()=>
                            #(#attrs)*
                            #vis #name: #new_ty
                        })
                    })
                    .filter_map(|x| x);
                quote! {
                    #(#new_fields ,)*
                }
            }
            Fields::Unnamed(ref fields) => {
                let new_fields = fields
                    .unnamed
                    .iter()
                    .map(|f| {
                        let option = FieldOption::from_field(f).unwrap();
                        if option.skip {
                            return None;
                        }
                        let vis = &f.vis;
                        let attrs = &f.attrs;
                        let ty = &f.ty;
                        let new_ty = if option.required {
                            quote! {#ty}
                        } else {
                            quote! {::std::option::Option<#ty>}
                        };
                        Some(quote_spanned! {f.span()=>
                            #(#attrs)*
                            #vis #new_ty
                        })
                    })
                    .filter_map(|x| x);
                quote! {
                    #(#new_fields ,)*
                }
            }
            Fields::Unit => {
                quote! {}
            }
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}
