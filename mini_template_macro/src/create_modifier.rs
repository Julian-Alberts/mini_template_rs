use std::collections::HashMap;

use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use syn::spanned::Spanned;

///## With body
/// ```
/// use mini_template::value::Value;
/// use mini_template_derive::create_modifier;
///
/// #[create_modifier]
/// fn fizz_buzz(n: usize) -> String {
///     match (n % 3, n % 5) {
///         (0, 0) => String::from("FIZZBUZZ"),
///         (0, _) => String::from("FIZZ"),
///         (_, 0) => String::from("BUZZ"),
///         _ => n.to_string()
///     }
/// }
///
///
/// assert_eq!(
///     fizz_buzz(
///         &Value::Number(3.),
///         Vec::default()
///     ),
///     Ok(Value::String(String::from("FIZZ")))
/// );
/// ```
/// ## Returns Result
/// ```
/// use mini_template::value::Value;
/// use mini_template_derive::create_modifier;
///
/// #[create_modifier(returns_result = true)]
/// fn as_usize(n: String) -> Result<usize, String> {
///     match n.parse() {
///         Ok(n) => Ok(n),
///         Err(_) => Err(format!("Can not convert {n} to usize"))
///     }
/// }
///
///
/// assert!(as_usize(&Value::String("17".to_owned()), Vec::default()).is_ok());
/// assert!(as_usize(&Value::String("Foo".to_owned()), Vec::default()).is_err());
/// ```
pub fn create_modifier(attrs: syn::AttributeArgs, item: syn::ItemFn) -> Result<TokenStream, syn::Error> {
    let inputs = Inputs::new(&item.sig.inputs)?;
    let attrs = Attrs::new(attrs, &inputs)?;
    let mini_template_crate_name = get_mini_template_crate_name(&attrs);

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
    let modifier_code_call = modifier_code_call(&item.sig.ident, &inputs.inputs, &attrs, &mini_template_crate_name);

    if attrs.modifier_ident.is_some() {
        Ok(quote::quote! {
            pub fn #modifier_ident(
                value: &#mini_template_crate_name::value::Value,
                args: Vec<&#mini_template_crate_name::value::Value>
            ) -> #mini_template_crate_name::modifier::error::Result<#mini_template_crate_name::value::Value> {
                use #mini_template_crate_name::modifier::error::Error;
                #vars
                let result: #mini_template_crate_name::modifier::error::Result<_> = #modifier_code_call;
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
                let result: #mini_template_crate_name::modifier::error::Result<_> = #modifier_code_call;
                result.map(#mini_template_crate_name::value::Value::from)
            }
        })
    }
}

fn get_mini_template_crate_name(attrs: &Attrs) -> syn::Ident {

    if let Some(mini_template_ident) = attrs.mini_template_crate.as_ref() {
        return mini_template_ident.clone()
    }

    let found_crate = crate_name("mini_template").expect("my-crate is present in `Cargo.toml`");
    match found_crate {
        FoundCrate::Itself => syn::Ident::new("crate", proc_macro2::Span::call_site()),
        FoundCrate::Name(name) => {
            syn::Ident::new(&name, proc_macro2::Span::call_site())
        }
    }
}

fn modifier_code_call<'a>(ident: &syn::Ident, inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>, attrs: &Attrs, mini_template_crate_name: &syn::Ident) -> TokenStream {
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
    if attrs.returns_result {
        quote::quote! {
            #ident(#inputs).or_else(|e| Err(#mini_template_crate_name::modifier::error::Error::Modifier(e)))
        }
    } else {
        quote::quote! {
            Ok(#ident(#inputs))
        }
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
        let into = into_value(quote::quote! {value}, mini_template_crate_name);
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
                let into = into_value(quote::quote! {v}, mini_template_crate_name);
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

fn into_value(value: TokenStream, mini_template_crate_name: &syn::Ident) -> TokenStream {
    quote::quote! {
        match #value.try_into() {
            Ok(inner) => inner,
            Err(e) => return Err(#mini_template_crate_name::modifier::Error::Type{value: #value.to_string(), type_error: e})
        }
    }
}

struct Attrs {
    defaults: HashMap<syn::Ident, syn::Lit>,
    modifier_ident: Option<syn::Ident>,
    returns_result: bool,
    mini_template_crate: Option<syn::Ident>
}

impl Attrs {

    fn new(args: syn::AttributeArgs, inputs: &Inputs) -> Result<Self, syn::Error> {
        let mut attrs = Attrs {
            defaults: HashMap::default(),
            modifier_ident: None,
            returns_result: false,
            mini_template_crate: None
        };

        for arg in args {
            if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                path,
                lit,
                ..
            })) = arg {
                if path.is_ident("modifier_ident") {
                    if let syn::Lit::Str(s_lit) = &lit {
                        attrs.modifier_ident = Some(syn::Ident::new(&s_lit.value(), lit.span()));
                        continue;
                    }
                    return Err(syn::Error::new(lit.span(), "modifier identifier needs to be string"));
                }

                if path.is_ident("mini_template_crate") {
                    if let syn::Lit::Str(s_lit) = &lit {
                        attrs.mini_template_crate = Some(syn::Ident::new(&s_lit.value(), lit.span()));
                        continue;
                    }
                    return Err(syn::Error::new(lit.span(), "mini_template_crate needs to be string"));
                }

                if path.is_ident("returns_result") {
                    if let syn::Lit::Bool(b_lit) = &lit {
                        attrs.returns_result = b_lit.value;
                        continue;
                    }
                    return Err(syn::Error::new(lit.span(), "returns_result needs to be boolean"));
                }

                let mut segments_iter = path.segments.iter();
                if let Some(syn::PathSegment{ ident, .. }) = segments_iter.next() {
                    if ident == &syn::Ident::new("defaults", proc_macro2::Span::call_site()) {
                        if let Some(syn::PathSegment{ ident, .. }) = segments_iter.next() {
                            if inputs.idents.iter().any(|ii| {
                                ident == *ii
                            }) {
                                attrs.defaults.insert(ident.clone(), lit);
                                continue;
                            }
                        }
                    }
                }

                return Err(syn::Error::new(path.span(), "Unknown argument"))
            }
            return Err(syn::Error::new(arg.span(), "Unknown argument"))
        }

        Ok(attrs)
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