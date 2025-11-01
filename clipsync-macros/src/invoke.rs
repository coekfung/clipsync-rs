use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Error, Expr, ExprLit, Ident, ItemTrait, Lit, Meta, Result, Token, TraitItem, TraitItemFn,
    parse::Parser, parse_quote, parse2, punctuated::Punctuated,
};

pub fn invoke_impl(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    struct InvokeArgs {
        ident: Option<String>,
    }
    let args = Parser::parse2(Punctuated::<Meta, Token![,]>::parse_terminated, attr)?;
    let mut parsed_args: InvokeArgs = InvokeArgs { ident: None };

    for arg in args {
        match arg {
            Meta::NameValue(pair) if pair.path.is_ident("ident") => {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit), ..
                }) = &pair.value
                {
                    let ident_value = lit.value();
                    parsed_args.ident = Some(ident_value);
                } else {
                    return Err(Error::new_spanned(
                        &pair.value,
                        "Expected string literal for 'ident' argument",
                    ));
                }
            }
            _ => {
                return Err(Error::new_spanned(arg, "Unknown argument"));
            }
        }
    }

    let trait_item = parse2::<ItemTrait>(item)?;

    let ident = &trait_item.ident;
    let impl_span = ident.span();

    let impl_ident = Ident::new(
        &parsed_args
            .ident
            .unwrap_or_else(|| format!("{}Invoke", ident.to_string())),
        impl_span,
    );

    let fn_impls = gen_fn_impls(&impl_ident, &trait_item)?;

    Ok(quote!(
        pub struct #impl_ident;

        impl #impl_ident {
            async fn __invoke(cmd: &str, args: ::wasm_bindgen::JsValue) -> ::wasm_bindgen::JsValue {
                use ::wasm_bindgen::prelude::*;
                #[wasm_bindgen]
                extern "C" {
                    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
                    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
                }

                invoke(cmd, args).await
            }

            async fn __invoke0(cmd: &str) -> ::wasm_bindgen::JsValue {
                use ::wasm_bindgen::prelude::*;
                #[wasm_bindgen]
                extern "C" {
                    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
                    async fn invoke(cmd: &str) -> JsValue;
                }

                invoke(cmd).await
            }
        }

        #fn_impls
    ))
}

fn gen_fn_impls(impl_ident: &Ident, trait_item: &ItemTrait) -> Result<TokenStream> {
    let trait_ident = &trait_item.ident;

    let mut impls = Vec::new();
    let mut fn_idents = Vec::new();
    for item in trait_item.items.iter().filter_map(|item| match item {
        TraitItem::Fn(fn_item) => Some(fn_item),
        _ => None,
    }) {
        let fn_impl = gen_fn_impl(impl_ident, item)?;
        impls.push(fn_impl);
        fn_idents.push(&item.sig.ident);
    }

    Ok(quote! {
        impl #trait_ident for #impl_ident {
            #(#impls)*
        }
    })
}

fn gen_fn_impl(impl_ident: &Ident, fn_item: &TraitItemFn) -> Result<TokenStream> {
    let fn_ident = &fn_item.sig.ident;
    let fn_inputs = &fn_item.sig.inputs;
    let fn_output = &fn_item.sig.output;
    let fn_async = &fn_item.sig.asyncness;

    let mut args_fields = Vec::new();
    let mut args_field_names = Vec::new();
    for input in fn_inputs.iter() {
        if let syn::FnArg::Typed(pat_type) = input {
            let field_name = &pat_type.pat;
            let field_type = &pat_type.ty;
            args_fields.push(quote! {
                #field_name: #field_type
            });
            args_field_names.push(field_name.clone());
        } else {
            return Err(Error::new_spanned(
                input,
                "Expected typed function argument",
            ));
        }
    }

    if args_fields.is_empty() {
        return Ok(parse_quote!(
            #fn_async fn #fn_ident() #fn_output {
                let __result = #impl_ident::__invoke0(stringify!(#fn_ident)).await;
                ::serde_wasm_bindgen::from_value(__result).unwrap()
            }
        ));
    }

    Ok(parse_quote!(
        #fn_async fn #fn_ident(#fn_inputs) #fn_output {
            #[derive(::serde::Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
                #(#args_fields),*
            }
            let __args = Args {
                #(#args_field_names),*
            };
            let __args = ::serde_wasm_bindgen::to_value(&__args).unwrap();
            let __result = #impl_ident::__invoke(stringify!(#fn_ident), __args).await;
            ::serde_wasm_bindgen::from_value(__result).unwrap()
        }
    ))
}
