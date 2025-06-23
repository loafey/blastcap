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
        _ => panic!("unsupported Rust type {ty:?}"),
    }
}

fn csharp_type_offset(off: usize, ty: &str) -> usize {
    let size = match ty {
        "String" => 8,
        "u32" | "f32" => 4,
        _ => panic!("unsupported offset type {ty:?}"),
    };
    let new_off = off + size;
    let diff = new_off % 8;
    new_off + diff
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

    let mut csharp_union: Vec<String> = Vec::new();
    let mut csharp_tags = Vec::new();

    for em in em.variants {
        let name = em.ident.span().source_text().unwrap();
        csharp_tags.push(name.clone());
        let mut args = Vec::new();
        match em.fields {
            syn::Fields::Named(f) => {
                for f in f.named {
                    let ident = f.ident.unwrap();
                    args.push((
                        format!("{name}_{}", ident.span().source_text().unwrap()),
                        f.ty.span().source_text().unwrap(),
                    ))
                }
            }
            syn::Fields::Unnamed(f) => {
                for (i, f) in f.unnamed.into_iter().enumerate() {
                    args.push((format!("{name}_arg{i}"), f.ty.span().source_text().unwrap()));
                }
            }
            syn::Fields::Unit => {}
        };
        let mut start = 4;
        for (name, ty) in args {
            start = csharp_type_offset(start, &ty);
            let ty = csharp_type(&ty);
            csharp_union.push(format!(
                "
    [System.Runtime.InteropServices.FieldOffset({start})]
    {ty} {name};
"
            ));
        }
    }

    let mut csharp_out = String::new();
    {
        let mut em = String::new();
        for (i, t) in csharp_tags.into_iter().enumerate() {
            let pre = if i == 0 { "" } else { ", " };
            em.push_str(&format!("{pre}{t}"));
        }
        writeln!(csharp_out, "enum ServerMessageTag {{ {em} }}").unwrap();
    }
    {
        let mut str = String::new();
        for s in csharp_union {
            str.push_str(&s);
        }
        writeln!(
            csharp_out,
            "
[System.Runtime.InteropServices.StructLayout(LayoutKind.Explicit)]
struct ServerMessage {{ 
    [System.Runtime.InteropServices.FieldOffset(0)]
    public ServerMessageTag tag;
    {str}
}}"
        )
        .unwrap();
    }
    csharp_out = format!(
        "using System.Runtime.InteropServices;
using System;

public partial class NetworkClient {{
{csharp_out}

    public void Poll()
    {{
        unsafe
        {{
            [DllImport(\"../target/debug/libblastcap.so\", SetLastError = true)]
            static extern ServerMessage* client_poll(void* ptr);
            client_poll(this.inner);
        }}
    }}
}}"
    );
    let mut f = File::create("godot/src/FFI_Polling.cs").unwrap();
    f.write_all(csharp_out.as_bytes()).unwrap();

    item_og
}
