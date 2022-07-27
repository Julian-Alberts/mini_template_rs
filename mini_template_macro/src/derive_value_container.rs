use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};

pub fn derive_value_container(input: syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let data = if let syn::Data::Struct(data) = input.data {
        data
    } else {
        return Err(syn::Error::new(input.ident.span() ,"Only structs are supported"))
    };

    let fields = if let syn::Fields::Named(fields) = data.fields {
        fields.named.into_iter().map(|named| {
            named.ident
        })
    } else {
        return Err(syn::Error::new(input.ident.span() ,"Only named structs are supported"))
    };

    let mini_template_crate_name = get_mini_template_crate_name();
    let struct_ident = input.ident;

    Ok(quote::quote! {
        impl #mini_template_crate_name::value::ValueContainer for #struct_ident {}
        impl Into<#mini_template_crate_name::value::ValueManager> for #struct_ident {

            fn into(self) -> #mini_template_crate_name::value::ValueManager{
                let mut vm = #mini_template_crate_name::value::ValueManager::default();
                #(
                    vm.set_value(#mini_template_crate_name::value::ident::ResolvedIdent::from(stringify!(#fields)), self.#fields.into()).unwrap();
                )*
                vm
            }

        }
    })
}

fn get_mini_template_crate_name() -> syn::Ident {
    let found_crate = crate_name("mini_template").expect("mini_template is present in `Cargo.toml`");
    println!("{:#?}", found_crate);
    match found_crate {
        FoundCrate::Itself => syn::Ident::new("crate", proc_macro2::Span::call_site()),
        FoundCrate::Name(name) => {
            syn::Ident::new(&name, proc_macro2::Span::call_site())
        }
    }
}
