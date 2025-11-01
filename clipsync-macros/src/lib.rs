mod command;
mod invoke;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn invoke(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut output = item.clone();
    let generated = match invoke::invoke_impl(attr.into(), item.into()) {
        Ok(ts) => TokenStream::from(ts),
        Err(err) => TokenStream::from(err.to_compile_error()),
    };

    output.extend(generated);
    output
}

#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut output = item.clone();
    let generated = match command::command_impl(attr.into(), item.into()) {
        Ok(ts) => TokenStream::from(ts),
        Err(err) => TokenStream::from(err.to_compile_error()),
    };

    output.extend(generated);
    output
}
