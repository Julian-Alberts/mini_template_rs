use std::collections::HashMap;

use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use syn::spanned::Spanned;

pub fn create_modifier(
    attrs: syn::AttributeArgs,
    item: syn::ItemFn,
) -> Result<TokenStream, syn::Error> {
    let inputs = as_fn_args(&item.sig.inputs)?;
    let attrs = Attrs::new(attrs, &inputs)?;
    let mini_template_crate_name = get_mini_template_crate_name();

    if let syn::ReturnType::Default = item.sig.output {
        return Err(syn::Error::new(
            item.sig.span(),
            "Modifier requires return type",
        ));
    }
    let modifier_ident = if let Some(ident) = &attrs.modifier_ident {
        ident
    } else {
        &item.sig.ident
    };

    let vars = create_var_init_code(&inputs, &attrs, &mini_template_crate_name)?;
    let inner_fn = &item;
    let modifier_code_call = modifier_code_call(&item.sig.ident, &inputs);
    let use_of_deprecated_feature_default_values = if !attrs.defaults.is_empty() {
        Some(
            quote::quote! {#[deprecated(since = "0.2.0", note = "Marked as deprecated by mini_template_macro: Default values will be removed in version 0.3.0")]},
        )
    } else {
        None
    };
    let use_of_deprecated_feature_returns_result = if attrs.returns_result_is_set {
        Some(
            quote::quote! {#[deprecated(since = "0.2.0", note = "Marked as deprecated by mini_template_macro: `returns_result` is no longer required and will be removed in version 0.3.0")]},
        )
    } else {
        None
    };
    let use_of_deprecated_feature = match (
        use_of_deprecated_feature_default_values,
        use_of_deprecated_feature_returns_result,
    ) {
        (Some(a), Some(b)) => quote::quote! {#a #b},
        (None, Some(b)) => b,
        (Some(a), None) => a,
        (None, None) => TokenStream::default(),
    };

    let res = if let Some(Input::Receiver(r)) = inputs.first() {
        quote::quote! {#r,}
    } else {
        quote::quote! {}
    };

    if attrs.modifier_ident.is_some() {
        Ok(quote::quote! {
            pub fn #modifier_ident(
                #res
                value: &#mini_template_crate_name::value::Value,
                args: Vec<&#mini_template_crate_name::value::Value>
            ) -> #mini_template_crate_name::modifier::error::Result<#mini_template_crate_name::value::Value> {
                use #mini_template_crate_name::modifier::error::Error;
                use #mini_template_crate_name::modifier::error::IntoModifierResult;
                #vars
                let result: #mini_template_crate_name::modifier::error::Result<_> = #modifier_code_call;
                result.map(#mini_template_crate_name::value::Value::from)
            }
            #use_of_deprecated_feature
            #inner_fn
        })
    } else {
        Ok(quote::quote! {
            #use_of_deprecated_feature
            pub fn #modifier_ident(
                #res
                value: &#mini_template_crate_name::value::Value,
                args: Vec<&#mini_template_crate_name::value::Value>
            ) -> #mini_template_crate_name::modifier::error::Result<#mini_template_crate_name::value::Value> {
                use #mini_template_crate_name::modifier::error::Error;
                use #mini_template_crate_name::modifier::error::IntoModifierResult;
                #vars
                #inner_fn
                let result: #mini_template_crate_name::modifier::error::Result<_> = #modifier_code_call;
                result.map(#mini_template_crate_name::value::Value::from)
            }
        })
    }
}

fn get_mini_template_crate_name() -> syn::Ident {
    let found_crate =
        crate_name("mini_template").expect("mini_template is present in `Cargo.toml`");
    match found_crate {
        FoundCrate::Itself => syn::Ident::new("crate", proc_macro2::Span::call_site()),
        FoundCrate::Name(name) => syn::Ident::new(&name, proc_macro2::Span::call_site()),
    }
}

fn modifier_code_call(ident: &syn::Ident, inputs: &[Input]) -> TokenStream {
    let res = if let Some(Input::Receiver(_)) = inputs.first() {
        quote::quote! {self.}
    } else {
        quote::quote! {}
    };
    let inputs = inputs
        .iter()
        .filter_map(filter_map_named_only)
        .map(|i| &i.ident)
        .collect::<syn::punctuated::Punctuated<_, syn::token::Comma>>();
    quote::quote! {
        #res #ident(#inputs).into_modifier_result()
    }
}

fn create_var_init_code(
    inputs: &[Input],
    attrs: &Attrs,
    mini_template_crate_name: &syn::Ident,
) -> Result<TokenStream, syn::Error> {
    let mut inputs_iter = inputs.iter().filter_map(filter_map_named_only);
    let value = {
        let value = inputs_iter.next().unwrap();

        let ty = value.ty;
        let ident = &value.ident;
        let into = into_value(quote::quote! {value}, mini_template_crate_name);

        if value.is_option {
            quote::quote! {
                let #ident: #ty = match value {
                    #mini_template_crate_name::value::Value::Null => None,
                    value => Some(#into)
                };
            }
        } else {
            quote::quote! {let #ident: #ty = #into;}
        }
    };

    let args = if inputs.len() > 1 {
        let mut args = quote::quote! {let mut args = args.into_iter();};
        let init = inputs_iter
            .map(|value| {
                let ty = value.ty;
                let ident = &value.ident;
                let into = into_value(quote::quote! {v}, mini_template_crate_name);
                if value.is_option {
                    if value.allow_missing {
                        return quote::quote! {
                            let #ident: #ty = match args.next() {
                                Some(#mini_template_crate_name::value::Value::Null) | None => None,
                                Some(v) => Some(#into),
                            };
                        }
                    }
                    let ident_str = syn::LitStr::new(&ident.to_string(), ident.span());
                    return quote::quote! {
                        let #ident: #ty = match args.next() {
                            Some(#mini_template_crate_name::value::Value::Null) => None,
                            Some(v) => Some(#into),
                            None => return Err(#mini_template_crate_name::modifier::error::Error::MissingArgument{argument_name: #ident_str})
                        };
                    }
                }

                let default = var_init_default(ident, attrs.defaults.get(ident), mini_template_crate_name);
                quote::quote! {
                    let #ident: #ty = match args.next() {
                        Some(v) => #into,
                        None => #default
                    };
                }

            }).collect::<TokenStream>();
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

fn var_init_default(
    ident: &syn::Ident,
    default: Option<&syn::Lit>,
    mini_template_crate_name: &syn::Ident,
) -> TokenStream {
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
    returns_result_is_set: bool,
}

impl Attrs {
    fn new(args: syn::AttributeArgs, inputs: &[Input]) -> Result<Self, syn::Error> {
        let mut attrs = Attrs {
            defaults: HashMap::default(),
            modifier_ident: None,
            returns_result_is_set: false,
        };

        for arg in args {
            if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                path,
                lit,
                ..
            })) = arg
            {
                if path.is_ident("modifier_ident") {
                    if let syn::Lit::Str(s_lit) = &lit {
                        attrs.modifier_ident = Some(syn::Ident::new(&s_lit.value(), lit.span()));
                        continue;
                    }
                    return Err(syn::Error::new(
                        lit.span(),
                        "modifier identifier must to be of type string",
                    ));
                }

                if path.is_ident("returns_result") {
                    if let syn::Lit::Bool(_) = &lit {
                        attrs.returns_result_is_set = true;
                        continue;
                    }
                    return Err(syn::Error::new(
                        lit.span(),
                        "returns_result must to be of type boolean",
                    ));
                }

                let mut segments_iter = path.segments.iter();
                if let Some(syn::PathSegment { ident, .. }) = segments_iter.next() {
                    if ident == &syn::Ident::new("defaults", proc_macro2::Span::call_site()) {
                        if let Some(syn::PathSegment { ident, .. }) = segments_iter.next() {
                            if let Some(input_info) = inputs
                                .iter()
                                .filter_map(|i| {
                                    if let Input::Named(n) = i {
                                        Some(n)
                                    } else {
                                        None
                                    }
                                })
                                .find(|ii| ident == &ii.ident)
                            {
                                if input_info.is_option {
                                    return Err(syn::Error::new(
                                        path.span(),
                                        "Arguments can not be optional and have a default value",
                                    ));
                                }
                                attrs.defaults.insert(ident.clone(), lit);
                                continue;
                            }
                        }
                    }
                }

                return Err(syn::Error::new(path.span(), "Unknown argument"));
            }
            return Err(syn::Error::new(arg.span(), "Unknown argument"));
        }

        Ok(attrs)
    }
}

enum Input<'a> {
    Named(InputNamed<'a>),
    Receiver(&'a syn::Receiver),
}

struct InputNamed<'a> {
    ident: syn::Ident,
    ty: &'a syn::Type,
    is_option: bool,
    allow_missing: bool,
}

fn as_fn_args<'a>(
    i: &'a syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
) -> Result<Vec<Input<'a>>, syn::Error> {
    if i.is_empty() {
        return Err(syn::Error::new(
            Spanned::span(i),
            "Modifiers require at least one argument",
        ));
    }
    let mut unnamed_value_index = 0;

    let mut inputs = i
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Typed(t) => syn_fn_arg_to_arg(t, &mut unnamed_value_index),
            syn::FnArg::Receiver(receiver) => Input::Receiver(receiver),
        })
        .collect::<Vec<_>>();

    for i in inputs.iter_mut().rev() {
        let i = if let Input::Named(i) = i {
            i
        } else {
            continue;
        };
        if i.is_option {
            i.allow_missing = true
        } else {
            break;
        }
    }

    Ok(inputs)
}

fn syn_fn_arg_to_arg<'a>(typed: &'a syn::PatType, unnamed_value_index: &mut usize) -> Input<'a> {
    let ident = if let syn::Pat::Ident(pat_ident) = &*typed.pat {
        pat_ident.ident.clone()
    } else {
        let i = syn::Ident::new(
            &format!("mini_template_unnamed_{unnamed_value_index}"),
            typed.span(),
        );
        *unnamed_value_index += 1;
        i
    };

    let ty = &*typed.ty;

    let is_option = if let syn::Type::Path(path) = &*typed.ty {
        is_pat_type(
            &path.path,
            syn::Ident::new("Option", proc_macro2::Span::call_site()),
        )
    } else {
        false
    };

    Input::Named(InputNamed {
        ident,
        is_option,
        ty,
        allow_missing: false,
    })
}

fn is_pat_type(path: &syn::Path, ident: syn::Ident) -> bool {
    path.segments.len() == 1 && path.segments.iter().next().unwrap().ident == ident
}

fn filter_map_named_only<'a>(i: &'a Input<'a>) -> Option<&'a InputNamed<'a>> {
    if let Input::Named(n) = i {
        Some(n)
    } else {
        None
    }
}
