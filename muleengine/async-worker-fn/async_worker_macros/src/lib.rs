use std::collections::HashMap;

use convert_case::Casing;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    parse::{discouraged::Speculative, Parse, ParseStream},
    punctuated::Punctuated,
    token::{And, Comma, SelfValue},
    Attribute, Binding, FnArg, Ident, ImplItem, ItemEnum, ItemImpl, ItemStruct, Pat, Receiver,
    ReturnType, Signature, Token,
};

// fn read_exact_ident<'a>(ident_name: &'a str, input: &ParseStream) -> syn::Result<&'a str> {
//     input.step(|cursor| {
//         if let Some((ident, rest)) = cursor.ident() {
//             if ident == ident_name {
//                 return Ok((ident, rest));
//             }
//         }
//         Err(cursor.error(format!("expected `{ident_name}`")))
//     })?;

//     Ok(ident_name)
// }

enum UserErrorType {
    Struct(ItemStruct),
    Enum(ItemEnum),
    TypeBinding(Binding),
}

impl Parse for UserErrorType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let input_fork_0 = input.fork();
        let input_fork_1 = input.fork();
        let input_fork_2 = input.fork();
        if let Ok(item_struct) = input_fork_0.parse::<ItemStruct>() {
            input.advance_to(&input_fork_0);
            Ok(UserErrorType::Struct(item_struct))
        } else if let Ok(item_enum) = input_fork_1.parse::<ItemEnum>() {
            input.advance_to(&input_fork_1);
            Ok(UserErrorType::Enum(item_enum))
        } else if let Ok(item_type_binding) = input_fork_2.parse::<Binding>() {
            input.advance_to(&input_fork_2);
            Ok(UserErrorType::TypeBinding(item_type_binding))
        } else {
            Err(input.error("expected struct declaration, enum declaration, or type binding"))
        }
    }
}

impl ToTokens for UserErrorType {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let generated_tokens = match self {
            UserErrorType::Struct(item_struct) => {
                quote! {
                    #item_struct
                }
            }
            UserErrorType::Enum(item_enum) => {
                quote! {
                    #item_enum
                }
            }
            UserErrorType::TypeBinding(type_binding) => {
                quote! {
                    #type_binding
                }
            }
        };
        tokens.extend(quote! {
            #generated_tokens
        });
    }
}

fn is_attribute_worker_fn(attr: &Attribute) -> bool {
    let mut path_segments_iter = attr.path.segments.iter();
    if let Some(first_segment) = path_segments_iter.next() {
        if first_segment.ident == "async_worker_fn" {
            return path_segments_iter.next().is_none();
        }
    }

    false
}

struct AsyncWorkerArg {
    key: Ident,
    value: Ident,
}

impl Parse for AsyncWorkerArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let value = input.parse::<Ident>()?;

        Ok(Self { key, value })
    }
}

struct AsyncWorkerArgs {
    client_name: String,
    command_type: String,
    channel_creator_fn: String,
}

impl Parse for AsyncWorkerArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let required_params = ["client_name", "channel_creator_fn", "command_type"];
        let mut read_params = HashMap::<String, (i32, AsyncWorkerArg)>::new();

        while !input.is_empty() {
            let arg = input.parse::<AsyncWorkerArg>()?;

            let key_str = arg.key.to_string();
            if !required_params.contains(&key_str.as_str()) {
                return Err(input.error(format!("unexpected parameter `{}`", key_str)));
            }
            if read_params
                .entry(key_str.clone())
                .or_insert_with_key(|_key| (0, arg))
                .0
                > 1
            {
                return Err(input.error(format!("multiple parameter: `{}`", key_str)));
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            client_name: read_params
                .get("client_name")
                .ok_or_else(|| input.error("missing parameter: `client_name`"))?
                .1
                .value
                .to_string(),
            channel_creator_fn: read_params
                .get("channel_creator_fn")
                .ok_or_else(|| input.error("missing parameter: `channel_creator_fn`"))?
                .1
                .value
                .to_string(),
            command_type: read_params
                .get("command_type")
                .ok_or_else(|| input.error("missing parameter: `command_type`"))?
                .1
                .value
                .to_string(),
        })
    }
}

struct FnSignatures {
    items: Vec<Signature>,
}

fn signature_to_command_enum_variant(signature: &Signature) -> TokenStream2 {
    let mut input_args: TokenStream2 = signature
        .inputs
        .iter()
        .filter_map(|input_arg| match input_arg {
            FnArg::Typed(_) => Some(quote! { #input_arg, }),
            FnArg::Receiver(_) => None,
        })
        .collect();

    let output_type = match &signature.output {
        ReturnType::Default => {
            quote! {
                ()
            }
        }
        ReturnType::Type(_, boxed_type) => {
            quote! {
                #boxed_type
            }
        }
    };

    input_args.extend(quote! {
        result_sender: ::tokio::sync::oneshot::Sender<#output_type>,
    });

    let command_name = signature
        .ident
        .to_string()
        .to_case(convert_case::Case::Pascal);
    let command_variant_ident = Ident::new(&command_name, Span::call_site());

    quote! {
        #command_variant_ident{#input_args}
    }
}

struct ClientFunction {
    head: TokenStream2,
    body: TokenStream2,
}

fn signature_to_client_function(
    signature: &Signature,
    client_name: &str,
    command_type_name: &Ident,
) -> ClientFunction {
    let mut client_signature = signature.clone();

    client_signature.asyncness = None;

    // create empty inputs
    client_signature.inputs = Punctuated::new();
    // add &self as first parameter
    client_signature.inputs.push(FnArg::Receiver(Receiver {
        attrs: vec![],
        reference: Some((
            And {
                spans: [Span::call_site()],
            },
            None,
        )),
        mutability: None,
        self_token: SelfValue {
            span: Span::call_site(),
        },
    }));
    // add every non self (non receiver) type parameters
    client_signature
        .inputs
        .extend(
            signature
                .inputs
                .iter()
                .filter_map(|input_arg| match input_arg {
                    FnArg::Typed(_) => Some(input_arg.clone()),
                    FnArg::Receiver(_) => None,
                }),
        );

    // empty client signature, and construct a TokenStream instead
    client_signature.output = ReturnType::Default;
    let client_fn_output_type = match &signature.output {
        ReturnType::Default => {
            quote! {
                Result<(), async_worker_fn::AllWorkersDroppedError>
            }
        }
        ReturnType::Type(_, boxed_type) => {
            quote! {
                Result<#boxed_type, async_worker_fn::AllWorkersDroppedError>
            }
        }
    };

    let fn_arg_idents: Punctuated<Box<Pat>, Comma> = signature
        .inputs
        .iter()
        .filter_map(|input_arg| match input_arg {
            FnArg::Typed(arg) => Some(arg.pat.clone()),
            FnArg::Receiver(_) => None,
        })
        .collect();

    let fn_name = signature.ident.to_string();
    let command_name = fn_name.to_case(convert_case::Case::Pascal);
    let command_variant_ident = Ident::new(&command_name, Span::call_site());

    let error_string_send = format!("{}::{}, msg = {{:?}}", client_name, fn_name);
    let error_string_recv = format!("{}::{} response, msg = {{:?}}", client_name, fn_name);

    let head = quote! {
        pub #client_signature -> impl ::std::future::Future<Output = #client_fn_output_type>
    };

    let body = quote! {
        let (result_sender, result_receiver) = tokio::sync::oneshot::channel();

        let ret = self.command_sender.send(#command_type_name::#command_variant_ident {
            result_sender,
            #fn_arg_idents
        });

        async move {
            if let Err(e) = ret {
                ::log::error!(#error_string_send, e);
                return Err(async_worker_fn::AllWorkersDroppedError);
            }

            match result_receiver.await {
                Ok(ret) => Ok(ret),
                Err(e) => {
                    ::log::error!(#error_string_recv, e);
                    Err(async_worker_fn::AllWorkersDroppedError)
                },
            }
        }
    };

    ClientFunction { head, body }
}

fn signature_to_command_execution_match_line(
    signature: &Signature,
    command_type_name: &Ident,
) -> TokenStream2 {
    let function_name = &signature.ident;
    let command_name = function_name
        .to_string()
        .to_case(convert_case::Case::Pascal);
    let command_name = Ident::new(&command_name, Span::call_site());

    let parameters: Punctuated<Box<Pat>, Comma> = signature
        .inputs
        .iter()
        .filter_map(|input_arg| match input_arg {
            FnArg::Typed(arg) => Some(arg.pat.clone()),
            FnArg::Receiver(_) => None,
        })
        .collect();

    if signature.asyncness.is_some() {
        quote! {
            #command_type_name::#command_name {
                result_sender,
                #parameters
            } => {
                tokio::spawn(async move {
                    let ret = self.#function_name(#parameters).await;
                    let _ = result_sender.send(ret);
                });
            }
        }
    } else {
        quote! {
            #command_type_name::#command_name {
                result_sender,
                #parameters
            } => {
                let ret = self.#function_name(#parameters);
                let _ = result_sender.send(ret);
            }
        }
    }
}

impl FnSignatures {
    fn to_command_enum_variants(&self) -> Punctuated<TokenStream2, Comma> {
        let mut ret = Punctuated::<TokenStream2, Comma>::new();

        ret.extend(self.items.iter().map(signature_to_command_enum_variant));

        ret
    }

    fn to_client_impl(&self, client_name: &str, command_type_name: &Ident) -> TokenStream2 {
        let client_functions: Vec<ClientFunction> = self
            .items
            .iter()
            .map(|sig| signature_to_client_function(sig, client_name, command_type_name))
            .collect();

        let client_function_definitions: TokenStream2 = client_functions
            .iter()
            .map(|client_fn| {
                let head = &client_fn.head;
                let body = &client_fn.body;
                quote! {
                    #head {
                        #body
                    }
                }
            })
            .collect();

        let client_name_ident = Ident::new(client_name, Span::call_site());

        quote! {
            #[derive(Clone)]
            pub struct #client_name_ident {
                command_sender: async_worker_fn::command_channel::CommandSender<#command_type_name>,
            }

            impl #client_name_ident {
                fn new(
                    sender: async_worker_fn::command_channel::CommandSender<#command_type_name>,
                ) -> Self {
                    Self {
                        command_sender: sender,
                    }
                }

                #client_function_definitions
            }
        }
    }

    fn to_command_execution_match(&self, command_type_name: &Ident) -> TokenStream2 {
        let command_execution_match_lines: TokenStream2 = self
            .items
            .iter()
            .map(|signature| {
                signature_to_command_execution_match_line(signature, command_type_name)
            })
            .collect();

        quote! {
            match command {
                #command_execution_match_lines
            }
        }
    }
}

struct WorkerCommandExecutorImpl {
    head: TokenStream2,
    body: TokenStream2,
}

impl WorkerCommandExecutorImpl {
    pub fn new(
        item_impl: &ItemImpl,
        command_type_name: &Ident,
        fn_signatures: &FnSignatures,
    ) -> Self {
        let attrs = &item_impl.attrs;
        let defaultness = &item_impl.defaultness;
        let unsafety = &item_impl.unsafety;
        let impl_token = &item_impl.impl_token;
        let generics = &item_impl.generics;
        let trait_ = &item_impl.trait_;
        let self_ty = &item_impl.self_ty;

        let attributes_ts: TokenStream2 = attrs
            .iter()
            .map(|attr| {
                quote! {
                    #attr
                }
            })
            .collect();

        let trait_ts = match trait_ {
            Some((bang, path, for_)) => {
                quote! {
                    #bang #path #for_
                }
            }
            None => quote! {},
        };

        let command_execution_match = fn_signatures.to_command_execution_match(command_type_name);

        Self {
            head: quote! {
                #attributes_ts
                #defaultness
                #unsafety #impl_token #generics #trait_ts #self_ty
            },
            body: quote! {
                pub fn execute_command(
                    &mut self,
                    command: #command_type_name
                ) {
                    #command_execution_match
                }

                pub fn try_execute_task(
                    &mut self,
                    receiver: &mut async_worker_fn::command_channel::CommandReceiver<#command_type_name>,
                ) -> Result<bool, async_worker_fn::AllClientsDroppedError> {
                    let command = receiver.try_recv();
                    match command {
                        Ok(command) => {
                            self.execute_command(command);
                            return Ok(true);
                        }
                        Err(async_worker_fn::command_channel::TryRecvError::Empty) => return Ok(false),
                        Err(async_worker_fn::command_channel::TryRecvError::Disconnected) => {
                            return Err(async_worker_fn::AllClientsDroppedError)
                        }
                    }
                }

                // pub async fn execute_task(
                //     &mut self,
                //     receiver: &mut async_worker_fn::command_channel::CommandReceiver<#command_type_name>,
                // ) -> Result<(), async_worker_fn::AllClientsDroppedError> {
                //     let command = receiver.recv_async().await;
                //     match command {
                //         Ok(command) => {
                //             self.execute_command(command)
                //         }
                //         Err(async_worker_fn::command_channel::RecvError::Disconnected) => {
                //             Err(async_worker_fn::AllClientsDroppedError)
                //         }
                //     }
                // }

                pub fn execute_command_queue(
                    &mut self,
                    receiver: &mut async_worker_fn::command_channel::CommandReceiver<#command_type_name>,
                ) -> Result<(), async_worker_fn::AllClientsDroppedError> {
                    while self.try_execute_task(receiver)? {}
                    Ok(())
                }
            },
        }
    }
}

impl ToTokens for WorkerCommandExecutorImpl {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let head = &self.head;
        let body = &self.body;

        tokens.extend(quote! {
            #head {
                #body
            }
        });
    }
}

struct ImplBlock {
    item_impl: ItemImpl,
    fn_items: FnSignatures,
}

impl Parse for ImplBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let input_fork_0 = input.fork();
        let mut item_impl: ItemImpl = input.parse()?;

        let fn_items = FnSignatures {
            items: item_impl
                .items
                .iter_mut()
                .filter_map(|item| {
                    if let ImplItem::Method(method) = item {
                        let mut found_async_worker_fn_attribute = false;
                        method.attrs.retain(|attr| {
                            if is_attribute_worker_fn(attr) {
                                found_async_worker_fn_attribute = true;
                                false
                            } else {
                                true
                            }
                        });

                        if found_async_worker_fn_attribute {
                            Some(method.sig.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect(),
        };

        for sig in fn_items.items.iter() {
            if sig.asyncness.is_some() {
                return Err(input_fork_0.error(format!(
                    "a worker function cannot be async `{}`",
                    sig.to_token_stream(),
                )));
            }
        }

        Ok(Self {
            item_impl,
            fn_items,
        })
    }
}

#[proc_macro_attribute]
pub fn async_worker_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: AsyncWorkerArgs = syn::parse_macro_input!(args);
    let impl_block: ImplBlock = syn::parse_macro_input!(input);

    let command_type_name = Ident::new(&args.command_type, Span::call_site());
    let worker_command_executor_impl = WorkerCommandExecutorImpl::new(
        &impl_block.item_impl,
        &command_type_name,
        &impl_block.fn_items,
    );
    let command_declarations: Punctuated<TokenStream2, Comma> =
        impl_block.fn_items.to_command_enum_variants();
    let client_impl = impl_block
        .fn_items
        .to_client_impl(&args.client_name, &command_type_name);
    let channel_creator_fn_name = Ident::new(&args.channel_creator_fn, Span::call_site());

    let item_impl = &impl_block.item_impl;

    let tokenstream = quote! {
        #item_impl

        #worker_command_executor_impl

        pub enum #command_type_name {
            #command_declarations
        }

        fn #channel_creator_fn_name() -> (
            async_worker_fn::command_channel::CommandSender<#command_type_name>,
            async_worker_fn::command_channel::CommandReceiver<#command_type_name>,
        ) {
            async_worker_fn::command_channel::command_channel()
        }

        #client_impl
    };

    // panic!("{tokenstream}");

    tokenstream.into()
}
