use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Type};

#[proc_macro_attribute]
pub fn service(_args: TokenStream, item: TokenStream) -> TokenStream {
    let stub = gen_stub(_args.clone(), item.clone());
    let service = gen_service(_args.clone(), item.clone());
    TokenStream::from_iter(vec![stub, service])
}

fn gen_stub(_: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    assert_eq!(input_fn.sig.inputs.len(), 2);

    let pat_type = if let syn::FnArg::Typed(pat_type) = input_fn.sig.inputs[1].clone() {
        pat_type
    } else {
        panic!("")
    };
    let request_ident = if let syn::Pat::Ident(pat_ident) = *pat_type.pat {
        pat_ident.ident
    } else {
        panic!("")
    };

    let request = input_fn.sig.inputs[1].clone();

    let request_ident = quote! { & #request_ident };
    let response = input_fn.sig.output.clone();

    let vis = input_fn.vis.clone();
    let service_ident = input_fn.sig.ident.clone();
    let gen_func_ident = Ident::new(
        &format!("rpc_call_{}", input_fn.sig.ident),
        input_fn.sig.ident.span(),
    );

    let transformed_fn = quote! {
        #vis async fn #gen_func_ident(addr: std::net::SocketAddr, #request) #response {
            use tokio::io::AsyncReadExt;
            use tokio::io::AsyncWriteExt;
            use tokio::io::BufReader;
            use tokio::net::TcpStream;
            use rpc_core::transport::*;

            let mut stream = TcpStream::connect(addr).await.unwrap();
            let (read, mut write) = stream.split();
            write
                .write(serde_json::to_vec(&Request {
                    name: stringify!(#service_ident).to_string(),
                    payload: serde_json::to_vec(#request_ident).unwrap(),
                }).unwrap().as_slice())
                .await
                .unwrap();
            let mut buf = [0u8; 1024];
            let mut buf_reader = BufReader::new(read);
            let bytes_read = buf_reader.read(&mut buf).await.unwrap();
            let res: Response = serde_json::from_slice(&buf[..bytes_read]).unwrap();
            serde_json::from_slice(&res.payload[..]).unwrap()
        }
    };

    // 返回生成的代码
    TokenStream::from(transformed_fn)
}

fn gen_service(_: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);

    let vis = function.vis.clone();

    let mut service_function = function.clone();

    let service_function_ident = Ident::new(
        format!("{}_service", function.sig.ident.to_string()).as_str(),
        function.sig.ident.span(),
    );
    service_function.sig.ident = service_function_ident.clone();
    service_function.vis = syn::Visibility::Inherited;

    let parameter_types: Vec<Box<Type>> = function
        .sig
        .inputs
        .clone()
        .into_iter()
        .filter_map(|input| match input {
            syn::FnArg::Typed(arg) => Some(arg.ty.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(parameter_types.len(), 2);

    let state_type = parameter_types[0].clone();
    let request_type = parameter_types[1].clone();
    let response_type = match function.sig.output.clone() {
        syn::ReturnType::Default => panic!(),
        syn::ReturnType::Type(_, ty) => ty,
    };

    let ident = function.sig.ident.clone();
    let gen = quote! {
        #[allow(non_camel_case_types)]
        #vis struct #ident;

        #service_function
        
        impl #ident {
            #vis fn service() -> rpc_core::service::Service<#state_type, #request_type, #response_type, impl std::future::Future<Output = #response_type>> {
                rpc_core::service::Service::new(stringify!(#ident).to_string(), #service_function_ident)
            }
        }
        
    };
    gen.into()
}
