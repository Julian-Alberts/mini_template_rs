use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::__private::ext::RepToTokensExt;

pub fn derive_value_container(input: syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let data = if let syn::Data::Struct(data) = input.data {
        data
    } else {
        return Err(syn::Error::new(
            input.ident.span(),
            "Only structs are supported",
        ));
    };

    let (fields, template_names) = if let syn::Fields::Named(fields) = data.fields {
        let field_names = fields
            .named
            .clone()
            .into_iter()
            .map(|named| named.ident.expect("named field"));
        let template_names = fields.named.into_iter().map(|named| {
            let attr = named.attrs.iter().find(|attr| {
                if let Some(attr) = attr.path.segments.first() {
                    attr
                } else {
                    return false;
                }
                .ident
                    == "name"
            });

            if let Some(proc_macro2::TokenTree::Group(group)) = attr
                .and_then(|attr| attr.tokens.next())
                .and_then(|tokens| tokens.clone().into_iter().next())
            {
                let key = group.stream().to_string();
                syn::Ident::new(key.as_str(), proc_macro2::Span::call_site())
            } else {
                named.ident.expect("named field")
            }
        });
        (field_names, template_names)
    } else {
        return Err(syn::Error::new(
            input.ident.span(),
            "Only named structs are supported",
        ));
    };

    let mini_template_crate_name = get_mini_template_crate_name();
    let struct_ident = input.ident;

    Ok(quote::quote! {
        impl #mini_template_crate_name::value::ValueContainer for #struct_ident {}
        impl Into<#mini_template_crate_name::value::ValueManager> for #struct_ident {

            fn into(self) -> #mini_template_crate_name::value::ValueManager{
                let mut vm = #mini_template_crate_name::value::ValueManager::default();
                #(
                    vm.set_value(#mini_template_crate_name::value::ident::ResolvedIdent::from(stringify!(#template_names)), self.#fields.into()).unwrap();
                )*
                vm
            }

        }
    })
}

fn get_mini_template_crate_name() -> syn::Ident {
    let found_crate =
        crate_name("mini_template").expect("mini_template is present in `Cargo.toml`");
    match found_crate {
        FoundCrate::Itself => syn::Ident::new("crate", proc_macro2::Span::call_site()),
        FoundCrate::Name(name) => syn::Ident::new(&name, proc_macro2::Span::call_site()),
    }
}
