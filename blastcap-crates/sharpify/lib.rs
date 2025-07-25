extern crate proc_macro;
use convert_case::Casing;
use proc_macro::TokenStream as TS1;
use quote::ToTokens;
use std::{fmt::Write as _, fs::File, io::Write as _};
use syn::{Item, ItemEnum, ItemMod, spanned::Spanned};

// TODO: This is a bunch of spaghetti!
// I am going to fix this soon!

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

fn csharp_type(ty: &str) -> String {
    let ty = ty.trim();
    match ty {
        "String" => "string".to_string(),
        "i8" => "sbyte".to_string(),
        "u8" => "byte".to_string(),
        "i16" => "Int16".to_string(),
        "u16" => "UInt16".to_string(),
        "i32" => "Int32".to_string(),
        "u32" => "UInt32".to_string(),
        "i64" => "Int64".to_string(),
        "u64" => "UInt64".to_string(),
        #[cfg(target_pointer_width = "64")]
        "isize" => "Int64".to_string(),
        #[cfg(not(target_pointer_width = "64"))]
        "isize" => "Int32".to_string(),
        #[cfg(target_pointer_width = "64")]
        "usize" => "UInt64".to_string(),
        #[cfg(not(target_pointer_width = "64"))]
        "usize" => "Int64".to_string(),
        "bool" => "bool".to_string(),
        "f32" => "float".to_string(),
        "f64" => "double".to_string(),
        _ if ty.starts_with("Vec < ") && ty.ends_with(" >") => {
            format!("List<{}>", csharp_type(&ty[6..ty.len() - 2]))
        }
        _ if ty.starts_with("HashMap < ") && ty.ends_with(" >") => {
            format!("Dictionary<{}>", csharp_type(&ty[10..ty.len() - 2]))
        }
        _ if ty.split_once(",").is_some() => {
            let Some((a, b)) = ty.split_once(",") else {
                unreachable!()
            };
            format!("{}, {}", csharp_type(a), csharp_type(b))
        }
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

fn ffi_type(index: usize, name: &str, ty: &str) -> (usize, String) {
    if is_rust_prim(ty) {
        return (0, format!("{} {name}", csharp_type(ty)));
    }
    (
        1,
        format!(
            "[MarshalAs(UnmanagedType.LPArray, SizeParamIndex={})] byte[] {name}, UInt32 {name}_len",
            index + 1
        ),
    )
}

#[proc_macro_attribute]
pub fn client_interface(_attr: TS1, item_og: TS1) -> TS1 {
    let item = item_og.clone();
    let em = syn::parse_macro_input!(item as ItemEnum);
    let mut csharp_out = String::new();
    let mut rust_out = String::new();
    for em in em.variants {
        let name = em.ident.to_string();
        let mut args = Vec::new();
        match em.fields {
            syn::Fields::Named(f) => {
                for f in f.named {
                    let ident = f.ident.unwrap().to_string();
                    let ty = f.ty.to_token_stream().to_string();
                    args.push((ident, ty))
                }
            }
            syn::Fields::Unnamed(f) => {
                for (i, f) in f.unnamed.into_iter().enumerate() {
                    args.push((format!("arg{i}"), f.ty.to_token_stream().to_string()));
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
        let mut call = "this._inner".to_string();
        let mut r#mod = 0;
        for (i, (n, t)) in args.iter().enumerate() {
            let prefix = if i == 0 { "" } else { ", " };
            write!(arg_string, "{prefix}{} {n}", csharp_type(t)).unwrap();
            let (m, ffi) = ffi_type(i + r#mod, n, t);
            r#mod += m;
            write!(csharp_rust_arg_string, ", {ffi}").unwrap();
            if is_rust_prim(t) {
                write!(call, ", {n}").unwrap();
            } else {
                write!(call, ", {n}_conv, {n}_len").unwrap();
            }
            write!(csharp_conv, "{}", csharp_gen_conv(n, t)).unwrap();
            write!(rust_arg_string, ", {n}: {}", rust_gen_type(n, t)).unwrap();
            write!(rust_pre_process, "{}", rust_convert_arg(n, t)).unwrap();
            write!(
                rust_args,
                "{}{n}",
                if rust_args.is_empty() { "" } else { ", " }
            )
            .unwrap();
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
        [DllImport(\"blastcap\", SetLastError = true)]
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
        .send_blocking(ClientRequest::{name}{rust_args})
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
    let mut f =
        std::fs::File::create("blastcap-frontend/objects/Globals/Network/Network_ClientCalls.cs")
            .unwrap();
    f.write_all(csharp_out.as_bytes()).unwrap();
    let mut f = std::fs::File::create("blastcap-core/lib_gen.rs").unwrap();
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
    for em in em.variants {
        let mut named = false;
        let name = em.ident.to_string();
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
                    let ident = f
                        .ident
                        .unwrap_or_else(|| panic!("field name missing"))
                        .to_string();
                    let ty = f.ty.to_token_stream().to_string();
                    args.push((format!("{name}_{ident}"), ident, ty));
                }
            }
            syn::Fields::Unnamed(f) => {
                for (i, f) in f.unnamed.into_iter().enumerate() {
                    let ty = f.ty.to_token_stream().to_string();
                    args.push((format!("{name}_arg{i}"), format!("arg{i}"), ty));
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

        let mut cs_raw_callback_args = Vec::new();
        let mut r#mod = 0;
        args.iter().enumerate().for_each(|(i, (n, _, ty2))| {
            let (m, ffi) = ffi_type(i + r#mod, n, ty2);
            r#mod += m;
            cs_raw_callback_args.push(ffi);
        });
        let cs_raw_callback_args = cs_raw_callback_args.join(", ");
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
            [DllImport(\"blastcap\", SetLastError = true)]
            static extern void client_poll(void* ptr{cs_rust_callbacks});
            client_poll(this._inner{cs_callbacks});
        }}
    }}
    public void Connect([MarshalAs(UnmanagedType.LPUTF8Str)] string addr)
    {{
        [DllImport(\"blastcap\", SetLastError = true)]
        static extern unsafe void start_client_loop(void* inner, [MarshalAs(UnmanagedType.LPUTF8Str)] string addr);
        unsafe
        {{
            start_client_loop(_inner, addr);
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
    // if let Ok(msg) = client.recv.try_recv() {{
    while let Ok(msg) = client.recv.try_recv() {{
        unsafe {{
            match msg {{
    {rust_matches}
            }}
        }}
    }};
}}
"
    );
    let mut f =
        File::create("blastcap-frontend/objects/Globals/Network/Network_Polling.cs").unwrap();
    f.write_all(csharp_out.as_bytes()).unwrap();
    let mut f = File::create("blastcap-core/lib_poll.rs").unwrap();
    f.write_all(rust_out.as_bytes()).unwrap();

    item_og
}

#[proc_macro]
pub fn constants(item_og: TS1) -> TS1 {
    let item = item_og.clone();
    let moddy = syn::parse_macro_input!(item as ItemMod);
    let Some(content) = moddy.content else {
        panic!("wrong kind of mod!")
    };
    let mut constants = vec![];
    for item in content.1 {
        let Item::Const(con) = item else {
            panic!("non-const items are not supported!")
        };
        let name = con.ident.span().source_text().unwrap();
        let ty = con.ty.span().source_text().unwrap();
        let val = con.expr.span().source_text().unwrap();
        constants.push((ty, name, val));
    }

    let mut csharp_code = String::new();
    for (ty, name, value) in constants {
        writeln!(
            csharp_code,
            "    public const {} {name} = {value};",
            csharp_type(&ty)
        )
        .unwrap();
    }
    csharp_code = format!("using System;\nstatic class Constants {{\n{csharp_code}}}");

    let mut f = std::fs::File::create("blastcap-frontend/objects/Globals/Constants.cs").unwrap();
    f.write_all(csharp_code.as_bytes()).unwrap();
    item_og
}
