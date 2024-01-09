use darling::{
    ast::{Data, Fields, Style},
    util::Ignored,
    FromDeriveInput, FromVariant,
};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{DeriveInput, Expr, Generics, Ident};

#[derive(FromVariant, Debug)]
#[darling(attributes(grpc_error))]
struct ErrorVariant {
    ident: Ident,
    fields: Fields<()>,

    #[darling(default)]
    status: Option<Expr>,
}

#[derive(FromDeriveInput, Debug)]
#[darling(attributes(grpc_error))]
struct GrpcErrorOpts {
    ident: Ident,
    generics: Generics,
    data: Data<ErrorVariant, Ignored>,
}

impl ToTokens for GrpcErrorOpts {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let enum_ident = &self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let data = match self.data.as_ref() {
            Data::Enum(val) => val,
            Data::Struct(_) => panic!("expected an error enum, not a struct"),
        };

        let matcher = data.iter().map(|variant| {
            let name = &variant.ident;
            let fields = match &variant.fields.style {
                Style::Tuple => {
                    let placeholders = variant.fields.iter().map(|_| quote! { _ });

                    quote! { (#(#placeholders),*) }
                }
                Style::Struct => quote! { { .. } },
                Style::Unit => quote! {},
            };

            let internal_error = syn::parse_quote! {
                ::tonic::Code::Internal
            };

            let status = variant.status.as_ref().unwrap_or(&internal_error);

            quote! {
                #enum_ident :: #name #fields => {
                    if #status == #internal_error {
                        ::tracing::error!(error = %e, "internal server error");
                        ::tonic::Status::new(#status, "Internal server error.")
                    } else {
                        ::tonic::Status::new(#status, e.to_string())
                    }
                }
            }
        });

        quote! {
            impl #impl_generics From<#enum_ident #ty_generics> for ::tonic::Status #where_clause {
                fn from(e: #enum_ident #ty_generics) -> Self {
                    match e {
                        #(#matcher),*
                    }
                }
            }
        }
        .to_tokens(tokens);
    }
}

#[proc_macro_derive(GrpcError, attributes(grpc_error))]
pub fn derive_grpc_error(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    match GrpcErrorOpts::from_derive_input(&input) {
        Ok(val) => val.to_token_stream().into(),
        Err(e) => e.write_errors().into(),
    }
}
