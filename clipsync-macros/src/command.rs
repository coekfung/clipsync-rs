use proc_macro2::TokenStream;
use quote::quote;
use syn::{ImplItem, ItemImpl, Result, parse2};

pub fn command_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let impl_item = parse2::<ItemImpl>(item)?;

    let (trait_path, self_ty, generics) = match &impl_item.trait_ {
        Some((_, path, _)) => (path, &impl_item.self_ty, &impl_item.generics),
        None => {
            return Err(syn::Error::new_spanned(
                &impl_item,
                "The #[command] attribute can only be applied to trait implementations.",
            ));
        }
    };

    let where_clause = &generics.where_clause;

    let mut proxy_methods = Vec::new();
    let mut proxy_method_names = Vec::new();
    for item in &impl_item.items {
        if let ImplItem::Fn(method) = item {
            let method_name = &method.sig.ident;
            let method_inputs = &method.sig.inputs;
            let method_output = &method.sig.output;
            let method_async = &method.sig.asyncness;

            let method_awaits = if method_async.is_some() {
                quote! { .await }
            } else {
                quote! {}
            };

            let mut method_input_names = Vec::new();
            for input in method_inputs.iter() {
                if let syn::FnArg::Typed(pat_type) = input {
                    method_input_names.push(&pat_type.pat);
                } else {
                    return Err(syn::Error::new_spanned(
                        input,
                        "Expected typed function argument",
                    ));
                }
            }

            let proxy = quote! {
                #[::tauri::command]
                #method_async fn #method_name(#method_inputs) #method_output {
                    <#self_ty as #trait_path #generics>::#method_name(#(#method_input_names),*) #method_awaits
                }
            };
            proxy_methods.push(proxy);
            proxy_method_names.push(method_name);
        }
    }

    Ok(quote! {
        impl #generics #self_ty #where_clause {
            pub fn invoke_handler<R: ::tauri::Runtime>(invoke: ::tauri::ipc::Invoke<R>) -> bool {
                mod inner {
                    use super::#trait_path;
                    use super::#self_ty;

                    #(#proxy_methods)*

                    pub fn invoke_handler<R: ::tauri::Runtime>(
                    ) -> impl Fn(::tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static {
                        ::tauri::generate_handler![#(#proxy_method_names)*]
                    }
                }
                inner::invoke_handler()(invoke)
            }
        }
    })
}
