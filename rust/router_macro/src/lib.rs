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

static mut ROUTE_ENTRIES: Vec<(String, String, String)> = Vec::new();

#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: LambdaHandler = parse_macro_input!(args as LambdaHandler);
    let function = parse_macro_input!(input as ItemFn);
    let func_name = &function.sig.ident;
    let path = &args.path;
    let method = &args.method;

    println!("Method: {:?} Path: {:?}", args.method, args.path);

    unsafe {
        ROUTE_ENTRIES.push((method.into(), path.into(), func_name.to_string()));
    }

    // Generate function as usual
    let token_stream = quote! {
        #function
    };

    token_stream.into()
}

#[proc_macro]
pub fn generate_routes(_input: TokenStream) -> TokenStream {
    let mut route_inserts = vec![];

    unsafe {
        for (method, path, handler) in ROUTE_ENTRIES.iter() {
            let handler_ident = syn::Ident::new(handler, proc_macro2::Span::call_site());

            route_inserts.push(quote! {
                trie.insert(#method, #path, #handler_ident);
            });
        }
    }

    // return the trie
    let expanded = quote! {
        {
            use router_container::Trie;
            let mut trie = Trie::new();

            #(#route_inserts)*

            trie
        }
    };

    expanded.into()
}
