use proc_macro::TokenStream;
use proc_macro_argue::{ArgumentList, argue};

argue! {
    MetaArgument {
        Method: syn::Ident,
        Tag: syn::LitStr,
        Summary: syn::LitStr,
        Description: syn::LitStr,
        Request: RequestArgument,
        Response: ResponseArgument,
        Security: SecurityArgument
    };
    RequestArgument {
        Description: syn::LitStr,
        Schema: SchemaArgument,
        Required
    };
    ResponseArgument {
        Code: syn::LitInt,
        Default,
        Description: syn::LitStr,
        Schema: SchemaArgument,
    };
    SchemaArgument(syn::LitStr, syn::token::Comma, syn::Ident);
    SecurityArgument {
        Name: syn::LitStr,
        Scope: syn::LitStr
    }
}

pub fn add_path(
    args: TokenStream, // (_, args): (&syn::Ident, &ArgumentList<syn::MetaList>),
                       // route: &Vec<String>,
                       // method: &ArgumentList<syn::Expr>,
) -> Result<(), syn::Error> {
    Ok(())
}
