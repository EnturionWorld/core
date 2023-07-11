use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};

fn wow_packet(
    input: proc_macro::TokenStream,
    session_struct: TokenStream,
) -> proc_macro::TokenStream {
    let cloned = input.clone();
    let ast = parse_macro_input!(cloned as DeriveInput);
    let name = ast.ident;

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(ref fields),
        ..
    }) = ast.data
    {
        fields
    } else {
        panic!("Only support Struct")
    };

    let mut field_names = vec![];
    let mut field_types = vec![];
    for field in fields.named.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let type_name = ty.to_token_stream();

        field_names.push(field_name);
        field_types.push(type_name);
    }

    let new_fn_name = Ident::new(&format!("{}_New", name), Span::call_site());
    let send_fn_name = Ident::new(&format!("{}_Send", name), Span::call_site());
    let input: TokenStream = input.into();
    let output = quote! {
        #[derive(::bincode::Encode, ::bincode::Decode)]
        #[repr(C)]
        #[must_use]
        #input

        impl ::enturion_shared::net::WoWPacket for #name {}
        impl #name {
            #[no_mangle]
            pub unsafe extern "C" fn #new_fn_name(#(#field_names: #field_types),*) -> Self {
                Self {
                    #(#field_names),*
                }
            }

            #[no_mangle]
            pub unsafe extern "C" fn #send_fn_name(self, session: *const ::std::ffi::c_void) {
                let session = std::mem::transmute::<_, #session_struct>(session);
                let _ = ::tokio::runtime::Handle::try_current().unwrap().spawn(async move {
                    let _ = ::enturion_shared::net::Session::send_packet::<#name>(session, self).await;
                });
            }
        }
    };

    output.into()
}

#[proc_macro_attribute]
pub fn wow_auth_packet(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let session_type = quote! { &mut ::enturion_authserver::auth_session::AuthSession };
    wow_packet(input, session_type)
}
