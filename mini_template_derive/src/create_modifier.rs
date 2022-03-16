use std::collections::HashMap;

use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use syn::spanned::Spanned;

pub fn create_modifier(attrs: syn::AttributeArgs, item: syn::ItemFn) -> Result<TokenStream, syn::Error> {
    let found_crate = crate_name("mini_template").expect("my-crate is present in `Cargo.toml`");
    let mini_template_crate_name = match found_crate {
        FoundCrate::Itself => syn::Ident::new("crate", proc_macro2::Span::call_site()),
        FoundCrate::Name(name) => {
            syn::Ident::new(&name, proc_macro2::Span::call_site())
        }
    };

    let inputs = Inputs::new(&item.sig.inputs)?;
    let attrs = Attrs::new(attrs, &&inputs)?;

    if let syn::ReturnType::Default = item.sig.output {
        return Err(syn::Error::new(item.sig.span() ,"Modifier requires return type"))
    }


    let modifier_ident = if let Some(ident) = &attrs.modifier_ident {
        ident
    } else {
        &item.sig.ident
    };

    let vars = create_var_init_code(&inputs, &attrs, &mini_template_crate_name)?;
    let inner_fn = &item;
    let modifier_code_call = modifier_code_call(&item.sig.ident, &inputs.inputs);

    if attrs.modifier_ident.is_some() {
        Ok(quote::quote! {
            pub fn #modifier_ident(
                value: &#mini_template_crate_name::value::Value,
                args: Vec<&#mini_template_crate_name::value::Value>
            ) -> #mini_template_crate_name::modifier::error::Result<#mini_template_crate_name::value::Value> {
                use #mini_template_crate_name::modifier::error::Error;
                #vars
                let result: #mini_template_crate_name::modifier::error::Result<_> = #modifier_code_call.or_else(|e| Err(Error::Modifier(e)));
                result.map(#mini_template_crate_name::value::Value::from)
            }
            #inner_fn
        })
    } else {
        Ok(quote::quote! {
            pub fn #modifier_ident(
                value: &#mini_template_crate_name::value::Value,
                args: Vec<&#mini_template_crate_name::value::Value>
            ) -> #mini_template_crate_name::modifier::error::Result<#mini_template_crate_name::value::Value> {
                use #mini_template_crate_name::modifier::error::Error;
                #vars
                #inner_fn
                let result: #mini_template_crate_name::modifier::error::Result<_> = #modifier_code_call.or_else(|e| Err(Error::Modifier(e)));
                result.map(#mini_template_crate_name::value::Value::from)
            }
        })
    }
}

fn modifier_code_call<'a>(ident: &syn::Ident, inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>) -> TokenStream {
    let inputs = inputs.iter().map(|i| {
        if let syn::FnArg::Typed(syn::PatType {
            pat,
            ..
        }) = i {
            if let syn::Pat::Ident(ident) = pat.as_ref() {
                ident
            } else {
                unreachable!()
            }
        } else {
            unreachable!()
        }
    }).collect::<syn::punctuated::Punctuated<_, syn::token::Comma>>();
    quote::quote! {
        #ident(#inputs)
    }
}

fn create_var_init_code<'a>(
    inputs: &Inputs,
    attrs: &Attrs,
    mini_template_crate_name: &syn::Ident
) -> Result<TokenStream, syn::Error> {
    let mut inputs_iter = inputs.inputs.iter();
    let value = {
        let value = inputs_iter.next().unwrap();
        
        let value = if let syn::FnArg::Typed(typed) = value {
            typed
        } else {
            unreachable!()
        };

        let ty = &value.ty;
        let ident = if let syn::Pat::Ident(ident) = &*value.pat {
            &ident.ident
        } else {
            return Err(syn::Error::new(value.span(), "Ident must be named"));
        };
        let into = into_value(quote::quote! {value});
        quote::quote! {let #ident: #ty = #into;}
    };

    let args = if inputs.inputs.len() > 1 {
        let mut args = quote::quote! {let mut args = args.into_iter();};
        let init = inputs_iter
            .map(|value| {
                let value = if let syn::FnArg::Typed(typed) = value {
                    typed
                } else {
                    unreachable!()
                };
                let ty = &*value.ty;
                let ident = if let syn::Pat::Ident(ident) = &*value.pat {
                    &ident.ident
                } else {
                    return Err(syn::Error::new(value.span(), "Ident must be named"));
                };
                let into = into_value(quote::quote! {v});
                let default = var_init_default(ident, attrs.defaults.get(ident), mini_template_crate_name);
                Ok(quote::quote! {
                    let #ident: #ty = match args.next() {
                        Some(v) => #into,
                        None => #default
                    };
                })
            }).collect::<Result<TokenStream, _>>()?;
            args.extend(init);
            args
    } else {
        TokenStream::new()
    };

    Ok(quote::quote! {
        #value
        #args
    })
}

fn var_init_default(ident: &syn::Ident, default: Option<&syn::Lit>, mini_template_crate_name: &syn::Ident) -> TokenStream {
    match default {
        Some(d) => quote::quote! {#d},
        None => {
            let ident = syn::LitStr::new(&ident.to_string(), ident.span());
            quote::quote! {return Err(#mini_template_crate_name::modifier::error::Error::MissingArgument{argument_name: #ident})}
        }
    }
}

fn into_value(value: TokenStream) -> TokenStream {
    quote::quote! {
        match #value.try_into() {
            Ok(inner) => inner,
            Err(e) => return Err(Error::Type{value: #value.to_string(), type_error: e})
        }
    }
}

struct Attrs {
    defaults: HashMap<syn::Ident, syn::Lit>,
    modifier_ident: Option<syn::Ident>
}

impl Attrs {

    fn new(args: syn::AttributeArgs, inputs: &Inputs) -> Result<Self, syn::Error> {
        let mut defaults = HashMap::default();
        let mut macro_name = None;
        for arg in args {
            if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                path,
                lit,
                ..
            })) = arg {
                if path.is_ident("modifier_ident") {
                    if let syn::Lit::Str(s_lit) = &lit {
                        macro_name = Some(syn::Ident::new(&s_lit.value(), lit.span()));
                        continue;
                    }
                    return Err(syn::Error::new(lit.span(), "modifier identifier needs to be string"));
                }

                let mut segments_iter = path.segments.iter();
                if let Some(syn::PathSegment{ ident, .. }) = segments_iter.next() {
                    if ident == &syn::Ident::new("defaults", proc_macro2::Span::call_site()) {
                        if let Some(syn::PathSegment{ ident, .. }) = segments_iter.next() {
                            if inputs.idents.iter().any(|ii| {
                                ident == *ii
                            }) {
                                defaults.insert(ident.clone(), lit);
                                continue;
                            }
                        }
                    }
                }

                return Err(syn::Error::new(path.span(), "Unknown argument"))
            }
        }

        Ok(Self {
            defaults,
            modifier_ident: macro_name
        })
    }

}

struct Inputs<'a> {
    idents: Vec<&'a syn::Ident>,
    inputs: &'a syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>
}

impl <'a> Inputs<'a> {

    fn new(i: &'a syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>) -> Result<Self, syn::Error> {
        if i.is_empty() {
            return Err(syn::Error::new(Spanned::span(i), "Modifiers require at least one argument"));
        }
        Ok(Self {
            idents: i.iter().map(|i| {
                let typed = if let syn::FnArg::Typed(t) = i {
                    t
                } else {
                    return Err(syn::Error::new(i.span(), "All arguments need to be typed"))
                };
                if let syn::Pat::Ident(pat_ident) = &*typed.pat {
                    Ok(&pat_ident.ident)
                } else {
                    return Err(syn::Error::new(i.span(), "All arguments need to be typed"))
                }
            }).collect::<Result<_, _>>()?,
            inputs: i
        })
    }

}