use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    LitStr, Token,
};
use syn::{parse_macro_input, ItemFn};

struct LambdaHandler {
    path: String,
    method: String,
}

impl Parse for LambdaHandler {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut path = None;
        let mut method = None;

        while !input.is_empty() {
            let key = input.parse::<syn::Ident>()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "path" => path = Some(input.parse::<LitStr>()?),
                "method" => method = Some(input.parse::<LitStr>()?),
                _ => return Err(input.error("Unknown key")),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        let path = path.ok_or_else(|| input.error("Missing `path` argument"))?;
        let method = method.ok_or_else(|| input.error("Missing `method` argument"))?;

        Ok(LambdaHandler {
            path: path.value(),
            method: method.value(),
        })
    }
}

#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: LambdaHandler = parse_macro_input!(args as LambdaHandler);
    let function = parse_macro_input!(input as ItemFn);
    let func_name = &function.sig.ident;
    let path = &args.path;
    let method = &args.method;

    println!("Method register {:?}", args.method);
    println!("Path register   {:?}", args.path);

    let token_stream = quote! {
        #function
        // anonymous function
        const _: () = {
            #[::ctor::ctor]
            fn register_route() {
                let path = #path;
                let method = #method;
                let handler = #func_name;
                router_container::register_route(path, method, handler);
            }
        };
    };

    token_stream.into()
}
