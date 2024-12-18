extern crate proc_macro;

use self::proc_macro::TokenStream as TokenStream1;
use proc_macro2::{TokenStream, TokenTree};
use proc_macro_error2::proc_macro_error;
use quote::quote;

mod chains;
mod check;
mod patterns;
mod script;
mod token;

use crate::{
    chains::ChainIter,
    check::Checker,
    patterns::{all, caps},
    script::Script,
};

fn to_ident(tt: &TokenTree) -> TokenStream {
    let s: TokenStream = tt.clone().into();
    s
}

fn gen_all(script: &Script) -> TokenStream {
    let mut s = TokenStream::new();

    let chains = ChainIter::new(script);
    let pchains = chains.pchains();

    for c in chains {
        s.extend(c.gen());
    }

    for c in pchains {
        s.extend(c.gen());
    }

    s
}

#[proc_macro_error]
#[proc_macro]
pub fn lua(input: TokenStream1) -> TokenStream1 {
    let script = Script::new(input.into(), true);

    Checker::new()
        .defines(all(&script).map(|(_, arg)| arg.as_lua().into()).collect())
        .check(&script);

    let defs = gen_all(&script);

    let body_str = script.script();
    let script_str = script.wrap();

    let args = all(&script).map(|(_, arg)| {
        let arg = arg.as_lua().to_string();
        quote! { #arg }
    });

    let caps = caps(&script).map(|(_, arg)| {
        let arg = to_ident(arg.as_rust());
        quote! { #arg }
    });

    let script_code = quote! {
        {
            use redis_lua::Script;

            #defs

            Chain0::new(redis_lua::Info::new(#script_str, #body_str, &[#(#args),*]), (), #(#caps),*)
        }
    };
    script_code.into()
}

#[proc_macro_error]
#[proc_macro]
pub fn lua_s(input: TokenStream1) -> TokenStream1 {
    let script = Script::new(input.into(), false);

    Checker::new().define("ARGV").check(&script);

    let script = script.script();
    let script_code = quote! {
        #script
    };
    script_code.into()
}
