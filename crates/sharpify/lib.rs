extern crate proc_macro;
use convert_case::Casing;
use proc_macro::TokenStream as TS1;
use proc_macro2::TokenStream;
use quote::quote;
use std::{fmt::Write as _, io::Write as _};
use syn::{ItemEnum, spanned::Spanned};

fn csharp_type(ty: &str) -> &str {
    match ty {
        "String" => "string",
        _ => panic!("unsupported type {ty:?}"),
    }
}

fn rust_type(ty: &str) -> &str {
    match ty {
        "String" => "*const std::ffi::c_char",
        _ => panic!("unsupported type {ty:?}"),
    }
}

fn rust_convert_arg(arg: &str, ty: &str) -> String {
    match ty {
        "String" => {
            format!(
                "let {arg} = unsafe {{ CStr::from_ptr({arg}) }}
        .to_string_lossy()
        .to_string();\n"
            )
        }
        _ => String::new(),
    }
}

fn ffi_type(ty: &str) -> &str {
    match ty {
        "String" => "[MarshalAs(UnmanagedType.LPUTF8Str)] string",
        _ => panic!("unsupported type {ty:?}"),
    }
}

#[proc_macro_attribute]
pub fn client_interface(_attr: TS1, item_og: TS1) -> TS1 {
    let item = TokenStream::from(item_og.clone());
    let em = syn::parse_macro_input!(item_og as ItemEnum);
    let mut csharp_out = String::new();
    let mut rust_out = String::new();
    for em in em.variants {
        let name = em.ident.span().source_text().unwrap();
        let mut args = Vec::new();
        match em.fields {
            syn::Fields::Named(f) => {
                for f in f.named {
                    let ident = f.ident.unwrap();
                    args.push((
                        ident.span().source_text().unwrap(),
                        f.ty.span().source_text().unwrap(),
                    ))
                }
            }
            syn::Fields::Unnamed(f) => {
                for (i, f) in f.unnamed.into_iter().enumerate() {
                    args.push((format!("arg{i}"), f.ty.span().source_text().unwrap()));
                }
            }
            syn::Fields::Unit => {}
        };
        let mut arg_string = String::new();
        let mut csharp_rust_arg_string = "void* ptr".to_string();
        let mut rust_arg_string = "client: *mut ClientHandle".to_string();
        let mut rust_pre_process = String::new();
        let mut rust_args = String::new();
        let mut call = "this.inner".to_string();
        for (i, (n, t)) in args.iter().enumerate() {
            let prefix = if i == 0 { "" } else { ", " };
            write!(arg_string, "{prefix}{} {n}", csharp_type(t)).unwrap();
            write!(csharp_rust_arg_string, ", {} {n}", ffi_type(t)).unwrap();
            write!(rust_arg_string, ", {n}: {}", rust_type(t)).unwrap();
            write!(call, ", {n}").unwrap();
            write!(rust_pre_process, "{}", rust_convert_arg(n, t)).unwrap();
            write!(rust_args, "{n}").unwrap();
        }
        if !rust_args.is_empty() {
            rust_args = format!("({rust_args})")
        }
        let rust_name = format!("client_send_{}", name.to_case(convert_case::Case::Snake));

        writeln!(
            csharp_out,
            "
public void Send{name}({arg_string}) {{
    unsafe {{
        [DllImport(\"../target/debug/libblastcap.so\", SetLastError = true)]
        static extern void {rust_name}({csharp_rust_arg_string});
        {rust_name}({call});
    }}
}}"
        )
        .unwrap();
        writeln!(
            rust_out,
            "
///
/// # Safety
/// 
#[unsafe(no_mangle)]
pub unsafe extern \"C\" fn {rust_name}({rust_arg_string}) {{
    let client = unsafe {{ &mut *client }} as &mut ClientHandle;
    {rust_pre_process}
    client
        .send
        .blocking_send(ClientRequest::{name}{rust_args})
        .unwrap();
}}
"
        )
        .unwrap();
    }
    csharp_out = format!(
        "
using System.Runtime.InteropServices;

public partial class NetworkClient {{{csharp_out}}}"
    );
    let mut f = std::fs::File::create("godot/src/FFI_ClientCalls.cs").unwrap();
    f.write_all(csharp_out.as_bytes()).unwrap();
    let mut f = std::fs::File::create("src/lib_gen.rs").unwrap();
    f.write_all(rust_out.as_bytes()).unwrap();
    let raw = proc_macro2::Literal::string(&csharp_out);
    quote! {
        const CSHARP_CODE: &str = #raw;
        #item
    }
    .into()
}
