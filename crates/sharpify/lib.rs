extern crate proc_macro;
use convert_case::Casing;
use proc_macro::TokenStream as TS1;
use std::{fmt::Write as _, fs::File, io::Write as _};
use syn::{ItemEnum, spanned::Spanned};

fn csharp_type(ty: &str) -> &str {
    match ty {
        "String" => "string",
        "u32" => "UInt32",
        "f32" => "float",
        _ => panic!("unsupported C# type {ty:?}"),
    }
}

fn rust_type(ty: &str) -> &str {
    match ty {
        "String" => "*const std::ffi::c_char",
        "u32" => "u32",
        "f32" => "f32",
        _ => panic!("unsupported Rust type {ty:?}"),
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

fn rust_poll_pre(arg: &str, ty: &str) -> String {
    match ty {
        "String" => format!("let {arg} = CString::new({arg}).unwrap().into_raw();"),
        _ => String::new(),
    }
}
fn rust_poll_suf(arg: &str, ty: &str) -> String {
    match ty {
        "String" => format!("_ = CString::from_raw({arg});"),
        _ => String::new(),
    }
}

fn ffi_type(ty: &str) -> &str {
    match ty {
        "String" => "[MarshalAs(UnmanagedType.LPUTF8Str)] string",
        "u32" => "UInt32",
        "f32" => "float",
        _ => panic!("unsupported FFI type {ty:?}"),
    }
}

#[proc_macro_attribute]
pub fn client_interface(_attr: TS1, item_og: TS1) -> TS1 {
    let item = item_og.clone();
    let em = syn::parse_macro_input!(item as ItemEnum);
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
    item_og
}

#[proc_macro_attribute]
pub fn client_poll(_attr: TS1, item_og: TS1) -> TS1 {
    let item = item_og.clone();
    let em = syn::parse_macro_input!(item as ItemEnum);

    let mut cs_rust_callbacks = String::new();
    let mut cs_callbacks = String::new();
    let mut cs_callbacks_fields = String::new();
    let mut cs_rust_callback_types = String::new();
    let mut cs_cons_args = String::new();
    let mut cs_cons = String::new();
    let mut cs_set_cons = String::new();
    let mut rust_matches = String::new();
    let mut rust_fn_args = String::new();
    let mut named = false;
    for em in em.variants {
        let name = em.ident.span().source_text().unwrap();
        let mut args = Vec::new();
        match em.fields {
            syn::Fields::Named(f) => {
                named = true;
                for f in f.named {
                    let ident = f.ident.unwrap();
                    args.push((
                        format!("{name}_{}", ident.span().source_text().unwrap()),
                        ident.span().source_text().unwrap(),
                        f.ty.span().source_text().unwrap(),
                    ))
                }
            }
            syn::Fields::Unnamed(f) => {
                for (i, f) in f.unnamed.into_iter().enumerate() {
                    args.push((
                        format!("{name}_arg{i}"),
                        format!("arg{i}"),
                        f.ty.span().source_text().unwrap(),
                    ));
                }
            }
            syn::Fields::Unit => {}
        };

        let rust_args = args
            .iter()
            .map(|(_, n, _)| n.clone())
            .collect::<Vec<_>>()
            .join(", ");
        let rust_type_args = args
            .iter()
            .map(|(_, _, ty)| rust_type(ty))
            .collect::<Vec<_>>()
            .join(", ");
        let pre = args
            .iter()
            .map(|(_, n, ty)| format!("{}\n", rust_poll_pre(n, ty)))
            .collect::<String>();
        let suf = args
            .iter()
            .map(|(_, n, ty)| format!("{}\n", rust_poll_suf(n, ty)))
            .collect::<String>();
        let wrap_rust_args = if rust_args.is_empty() {
            String::new()
        } else if named {
            format!("{{ {rust_args} }}")
        } else {
            format!("({rust_args})")
        };
        rust_fn_args.push_str(&format!(
            "{}_callback: unsafe extern \"C\" fn({rust_type_args}), ",
            name.to_case(convert_case::Case::Snake)
        ));
        rust_matches.push_str(&format!(
            "ServerMessage::{name}{wrap_rust_args} => {{
{pre}{}_callback({rust_args});
{suf}
}}\n",
            name.to_case(convert_case::Case::Snake)
        ));
        let cs_args = args
            .iter()
            .map(|(name, _, ty)| format!("{} {name}", ffi_type(ty)))
            .collect::<Vec<_>>()
            .join(", ");
        cs_rust_callback_types.push_str(&format!(
            "public delegate void {name}Callback({cs_args});
    "
        ));
        cs_callbacks_fields.push_str(&format!("    private {name}Callback {name}Fn;\n"));
        cs_callbacks.push_str(&format!(", this.{name}Fn"));
        cs_cons_args.push_str(&format!(", {name}Callback {name}Fn"));
        cs_set_cons.push_str(&format!("this.{name}Fn = {name}Fn;"));
        cs_cons.push_str(&format!(", {name}Fn"));
        cs_rust_callbacks.push_str(&format!(", {name}Callback {name}Fn"));
    }

    let csharp_out = format!(
        "using System.Runtime.InteropServices;
using System;

public partial class NetworkClient {{
    {cs_rust_callback_types}
{cs_callbacks_fields}
    public void Poll()
    {{
        unsafe
        {{
            [DllImport(\"../target/debug/libblastcap.so\", SetLastError = true)]
            static extern void client_poll(void* ptr{cs_rust_callbacks});
            client_poll(this.inner{cs_callbacks});
        }}
    }}
    private unsafe NetworkClient (void* inner{cs_cons_args}) {{
        this.inner = inner;
        {cs_set_cons}
    }}
    public static NetworkClient StartClientLoop([MarshalAs(UnmanagedType.LPUTF8Str)] string addr{cs_cons_args})
    {{
        [DllImport(\"../target/debug/libblastcap.so\", SetLastError = true)]
        static extern unsafe void* start_client_loop([MarshalAs(UnmanagedType.LPUTF8Str)] string addr);
        unsafe
        {{
            void* ptr = start_client_loop(addr);
            return new NetworkClient(ptr{cs_cons});
        }}
    }}
}}"
    );
    let rust_out = format!(
        "
///
/// # Safety
///
#[unsafe(no_mangle)]
pub unsafe extern \"C\" fn client_poll(
    client: *mut ClientHandle,
    {rust_fn_args}
) {{
    let client = unsafe {{ &mut *client }} as &mut ClientHandle;
    let Ok(msg) = client.recv.try_recv() else {{
        return;
    }};
    unsafe {{
        match msg {{
{rust_matches}
        }}
    }}
}}
"
    );
    let mut f = File::create("godot/src/FFI_Polling.cs").unwrap();
    f.write_all(csharp_out.as_bytes()).unwrap();
    let mut f = File::create("src/lib_poll.rs").unwrap();
    f.write_all(rust_out.as_bytes()).unwrap();

    item_og
}
