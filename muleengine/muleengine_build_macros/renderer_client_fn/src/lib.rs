use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    Expr, ExprType, Ident, Token, Type,
};

fn read_exact_ident<'a>(ident_name: &'a str, input: &ParseStream) -> syn::Result<&'a str> {
    input.step(|cursor| {
        if let Some((ident, rest)) = cursor.ident() {
            if ident == ident_name {
                return Ok((ident, rest));
            }
        }
        Err(cursor.error(format!("expected `{ident_name}`")))
    })?;

    Ok(ident_name)
}
struct RendererClientFnSignature {
    fn_name: Ident,
    fn_args: Punctuated<ExprType, Comma>,
    ok_ret_type: Type,
}

impl Parse for RendererClientFnSignature {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        input.parse::<Token![pub]>()?;
        input.parse::<Token![async]>()?;
        input.parse::<Token![fn]>()?;
        let fn_name: Ident = input.parse()?;

        let fn_args_parse_buffer;
        parenthesized!(fn_args_parse_buffer in input);

        fn_args_parse_buffer.parse::<Token![&]>()?;
        fn_args_parse_buffer.parse::<Token![self]>()?;

        let fn_args = if fn_args_parse_buffer.is_empty() {
            Punctuated::new()
        } else {
            fn_args_parse_buffer.parse::<Token![,]>()?;

            Punctuated::<ExprType, Comma>::parse_terminated(&fn_args_parse_buffer)?
        };

        input.parse::<Token![->]>()?;

        read_exact_ident("Result", &input)?;

        input.parse::<Token![<]>()?;
        let ok_ret_type = input.parse::<Type>()?;
        input.parse::<Token![,]>()?;
        read_exact_ident("RendererError", &input)?;
        input.parse::<Token![>]>()?;

        input.parse::<Token![;]>()?;

        Ok(Self {
            fn_name,
            fn_args,
            ok_ret_type,
        })
    }
}

struct CommandType(Ident);

impl Parse for CommandType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        read_exact_ident("Command", &input)?;
        input.parse::<Token![::]>()?;
        let command_type = input.parse::<Ident>()?;

        Ok(Self(command_type))
    }
}

#[proc_macro_attribute]
pub fn renderer_client_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let command_type: CommandType = syn::parse_macro_input!(args);

    let fn_signature: RendererClientFnSignature = syn::parse_macro_input!(input);

    let fn_name = &fn_signature.fn_name;
    let fn_args = &fn_signature.fn_args;
    let ok_ret_type = &fn_signature.ok_ret_type;
    let command_ty = &command_type.0;

    let fn_arg_idents =
        Punctuated::<Box<Expr>, Comma>::from_iter(fn_args.iter().map(|fn_arg| fn_arg.expr.clone()));

    let error_string_send = format!("renderer_client::{}, msg = {{:?}}", fn_name);
    let error_string_recv = format!("renderer_client::{} response, msg = {{:?}}", fn_name);

    quote! {
        pub async fn #fn_name (&self, #fn_args) -> Result<#ok_ret_type, RendererError> {
            let (result_sender, result_receiver) = oneshot::channel();
            self.command_sender
                .send(Command::#command_ty {
                    result_sender,
                    #fn_arg_idents
                })
                .inspect_err(|e| log::error!(#error_string_send, e))
                .map_err(|_| RendererError::RendererSystemDropped)?;

            match result_receiver
                .await
                .inspect_err(|e| log::error!(#error_string_recv, e))
            {
                Ok(ret) => ret,
                Err(_) => Err(RendererError::RendererSystemDropped),
            }
        }
    }
    .into()
}
