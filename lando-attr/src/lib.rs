//! provides function attribute macros for AWS Api Gateway for use in lando

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

// std lib(ish)
use proc_macro::TokenStream;

// third party
use syn::{parse, ItemFn, ReturnType};

/// Implements the `lando` attribute.
///
/// This attribute is used to turn a Rust function into an AWS Gateway
/// triggerable lambda. In lambda you can refer to these by path with
/// `lib{crate_name}.{fn_name}`
///
/// # Examples
///
/// ```rust,ignore
/// #[macro_use] extern crate lando;
/// use lando::{LambdaContext, Request, Response};
///
/// #[lando]
/// pub fn example<'a>(_: Request, _: LambdaContext) -> Response<&'a str> {
///   Ok("hello lambda")
/// }
/// ```
#[proc_macro_attribute]
pub fn lando(args: TokenStream, input: TokenStream) -> TokenStream {
    attr_impl(args, input)
}

// implementation. should expect the following
// * verify function type
// * input args are (lando::Request, lando::LambdaContext)
// * return type is lando::LandoResponse
fn attr_impl(_: TokenStream, input: TokenStream) -> TokenStream {
    let target: ItemFn = match parse(input.clone()) {
        Ok(f) => f,
        _ => {
            panic!("the 'lando' attribute can only be used on functions");
            // https://doc.rust-lang.org/proc_macro/struct.Span.html#method.error
            // use the following when this becomes stable
            /*Span::call_site()
            .error("the 'lando' attribute can only be used on functions")
            .emit();*/        }
    };
    if target.decl.inputs.len() != 2 {
        panic!(
            "the 'lando' attribute requires a function with two arguments. expecting {}(_: lando::Request, _: lando::LambdaContext) -> lando::Result", target.ident
            );
        // https://doc.rust-lang.org/proc_macro/struct.Span.html#method.error
        // use the following when it becomes stable
    }
    match target.decl.output {
        ReturnType::Default => {
            // https://doc.rust-lang.org/proc_macro/struct.Span.html#method.error
            // use the following when it becomes stable
            panic!("the 'lando' attribute requires a function that returns a value. expecting {}(_: lando::Request, _: lando::LambdaContext) -> lando::Result", target.ident);
        }
        _ => (),
    }
    let target_ident = target.ident.clone();
    let target_name = target_ident.to_string();
    let expanded = quote! {
        #target

        gateway!(stringify!(#target_name) => #target_ident);
    };
    expanded.into()
}
