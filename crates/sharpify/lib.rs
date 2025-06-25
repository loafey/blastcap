extern crate proc_macro;
use convert_case::Casing;
use proc_macro::TokenStream as TS1;
use std::{fmt::Write as _, fs::File, io::Write as _};
use syn::{ItemEnum, spanned::Spanned};

fn is_rust_prim(ty: &str) -> bool {
    matches!(
        ty,
        "i8" | "u8"
            | "i16"
            | "u16"
            | "i32"
            | "u32"
            | "i64"
            | "u64"
            | "isize"
            | "usize"
            | "bool"
            | "f32"
            | "f64"
    )
}

fn csharp_gen_conv(name: &str, ty: &str) -> String {
    if is_rust_prim(ty) {
        return format!("var {name}_conv = {name};");
    }

    format!(
        "byte[] {name}_conv = MessagePackSerializer.Serialize({name});
        UInt32 {name}_len = (UInt32){name}_conv.Length;"
    )
}

fn csharp_gen_de_conv(name: &str, ty: &str) -> String {
    if is_rust_prim(ty) {
        return format!("                var {name}_conv = {name};");
    }

    format!(
        "                var {name}_conv = MessagePackSerializer.Deserialize<{}>({name});",
        csharp_type(ty)
    )
}

fn csharp_type(ty: &str) -> &str {
    match ty {
        "String" => "string",
        "i8" => "sbyte",
        "u8" => "byte",
        "i16" => "Int16",
        "u16" => "UInt16",
        "i32" => "Int32",
        "u32" => "UInt32",
        "i64" => "Int64",
        "u64" => "UInt64",
        "isize" => "nint",
        "usize" => "nuint",
        "bool" => "bool",
        "f32" => "float",
        "f64" => "double",
        "Vec<String>" => "List<string>",
        _ => panic!("unsupported C# type {ty:?}"),
    }
}

fn rust_type(ty: &str) -> &str {
    match ty {
        _ if is_rust_prim(ty) => ty,
        _ => " *const std::ffi::c_char, u32",
    }
}
fn rust_gen_type(name: &str, ty: &str) -> String {
    match ty {
        _ if is_rust_prim(ty) => ty.to_string(),
        _ => format!("*const std::ffi::c_char, {name}_len: u32"),
    }
}

fn rust_convert_arg(arg: &str, ty: &str) -> String {
    if is_rust_prim(ty) {
        return String::new();
    }
    format!(
        "let {arg} = unsafe {{ std::slice::from_raw_parts({arg} as *const u8, {arg}_len as usize) }};
    let {arg} = rmp_serde::from_slice({arg}).unwrap();\n"
    )
}

fn rust_poll_pre(arg: &str, ty: &str) -> String {
    if is_rust_prim(ty) {
        return String::new();
    }

    format!(
        "let ({arg}, {arg}_len, {arg}_size) = rmp_serde::to_vec(&{arg}).unwrap().into_raw_parts();
let {arg}_len = {arg}_len as u32;
let {arg} = {arg} as *const i8;"
    )
}
fn rust_poll_suf(arg: &str, ty: &str) -> String {
    if is_rust_prim(ty) {
        return String::new();
    }
    format!("_ = Vec::from_raw_parts({arg} as *mut i8, {arg}_len as usize, {arg}_size);")
}

fn ffi_type(name: &str, ty: &str) -> String {
    if is_rust_prim(ty) {
        return format!("{} {name}", csharp_type(ty));
    }
    format!("[MarshalAs(UnmanagedType.LPArray, SizeParamIndex=1)] byte[] {name}, UInt32 {name}_len")
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
        let mut csharp_conv = String::new();
        let mut call = "this.inner".to_string();
        for (i, (n, t)) in args.iter().enumerate() {
            let prefix = if i == 0 { "" } else { ", " };
            write!(arg_string, "{prefix}{} {n}", csharp_type(t)).unwrap();
            write!(csharp_rust_arg_string, ", {}", ffi_type(n, t)).unwrap();
            if is_rust_prim(t) {
                write!(call, ", {n}").unwrap();
            } else {
                write!(call, ", {n}_conv, {n}_len").unwrap();
            }
            write!(csharp_conv, "{}", csharp_gen_conv(n, t)).unwrap();
            write!(rust_arg_string, ", {n}: {}", rust_gen_type(n, t)).unwrap();
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
        {csharp_conv}
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
    let Err(err) = client
        .send
        .blocking_send(ClientRequest::{name}{rust_args})
        else {{ return; }};
    unsafe {{
        let str = CString::new(format!(\"{{err}}\")).unwrap().into_raw();
        (client.on_fail)(str);
        _ = CString::from_raw(str)
    }};
}}
"
        )
        .unwrap();
    }
    csharp_out = format!(
        "using System;
using MessagePack;
using System.Runtime.InteropServices;

public partial class NetworkClient {{{csharp_out}}}"
    );
    let mut f = std::fs::File::create("godot/src/Globals/Network/Network_ClientCalls.cs").unwrap();
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
    let mut cs_set_cons = String::new();
    let mut cs_signals = String::new();
    let mut rust_matches = String::new();
    let mut rust_fn_args = String::new();
    let mut named = false;
    for em in em.variants {
        let name = em.ident.span().source_text().unwrap();
        let mut args = Vec::new();
        cs_signals.push_str(&format!("    private event {name}Callback _on{name};\n"));
        cs_signals.push_str(&format!(
            "    public event {name}Callback On{name}
    {{
        add
        {{
            _on{name} = null;
            _on{name} += value;
        }}
        remove
        {{
            _on{name} -= value;
        }}
    }}
"
        ));
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
        let rust_processed_args = args
            .iter()
            .map(|(_, n, ty)| {
                if is_rust_prim(ty) {
                    n.clone()
                } else {
                    format!("{n}, {n}_len")
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        let rust_type_args = args
            .iter()
            .map(|(_, _, ty)| rust_type(ty))
            .collect::<Vec<_>>()
            .join(", ");
        let cs_raw_callback_args = args
            .iter()
            .map(|(n, _, ty2)| ffi_type(n, ty2))
            .collect::<Vec<_>>()
            .join(", ");
        let cs_callback_args = args
            .iter()
            .map(|(n, _, _)| format!("{n}_conv"))
            .collect::<Vec<_>>()
            .join(", ");
        let cs_de_conv = args
            .iter()
            .map(|(n, _, ty)| format!("{}\n", csharp_gen_de_conv(n, ty)))
            .collect::<String>();
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
{pre}{}_callback({rust_processed_args});
{suf}
}}\n",
            name.to_case(convert_case::Case::Snake)
        ));
        let cs_args = args
            .iter()
            .map(|(name, _, _)| name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        let cs_args_with_types = args
            .iter()
            .map(|(name, _, ty)| format!("{} {name}", csharp_type(ty)))
            .collect::<Vec<_>>()
            .join(", ");
        cs_rust_callback_types.push_str(&format!(
            "public delegate void {name}Callback({cs_args_with_types});
    private delegate void {name}CallbackRaw({cs_raw_callback_args});
    "
        ));
        cs_callbacks_fields.push_str(&format!("    private {name}Callback {name}Fn;\n"));
        cs_callbacks.push_str(&format!(
            ", ({cs_raw_callback_args}) => {{
{cs_de_conv}
                this._on{name}({cs_callback_args});
            }}"
        ));
        cs_cons_args.push_str(&format!(", {name}Callback {name}Fn"));
        cs_set_cons.push_str(&format!(
            "
            this.{name}Fn = ({cs_args}) => _on{name}({});
            this.On{name} += ({cs_args}) => {{GD.PrintErr(\"Using default event handler for On{name}, please set it!\");}};",
            args.iter()
                .map(|(n, _, _)| n.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ));
        cs_rust_callbacks.push_str(&format!(", {name}CallbackRaw {name}Fn"));
    }

    let csharp_out = format!(
        "using System.Runtime.InteropServices;
using System;
using Godot;
using MessagePack;
using System.Collections.Generic;

public partial class NetworkClient {{
    {cs_rust_callback_types}
{cs_callbacks_fields}
{cs_signals}
    public void Poll()
    {{
        unsafe
        {{
            [DllImport(\"../target/debug/libblastcap.so\", SetLastError = true)]
            static extern void client_poll(void* ptr{cs_rust_callbacks});
            client_poll(this.inner{cs_callbacks});
        }}
    }}
    public NetworkClient([MarshalAs(UnmanagedType.LPUTF8Str)] string addr, OnFail onFail)
    {{
        [DllImport(\"../target/debug/libblastcap.so\", SetLastError = true)]
        static extern unsafe void* start_client_loop([MarshalAs(UnmanagedType.LPUTF8Str)] string addr, OnFail onFail);
        unsafe
        {{
            void* ptr = start_client_loop(addr, onFail);
            this.inner = ptr;
            {cs_set_cons}
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
    let mut f = File::create("godot/src/Globals/Network/Network_Polling.cs").unwrap();
    f.write_all(csharp_out.as_bytes()).unwrap();
    let mut f = File::create("src/lib_poll.rs").unwrap();
    f.write_all(rust_out.as_bytes()).unwrap();

    item_og
}
