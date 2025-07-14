using Godot;
using System;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;

public partial class Linker : Node {
    public delegate void OnPanic([MarshalAs(UnmanagedType.LPUTF8Str)] string error);

    public override void _Ready() {
        GD.Print("= Registering panic callback =");
        [DllImport("blastcap", SetLastError = true)]
        static extern unsafe int register_panic_callback(OnPanic func);
        try {
            register_panic_callback(GD.PrintErr);
        } catch (Exception e) {
            GD.Print($"Failed registering hook: {e}");
        }
        GD.Print("= Done registering panic callback =");
        // var assembly = Assembly.GetExecutingAssembly();
        // NativeLibrary.SetDllImportResolver(assembly, DllImportResolver);
    }
    // private static IntPtr DllImportResolver(string libraryName, Assembly assembly, DllImportSearchPath? searchPath) {
    //     if (libraryName == "blastcap") {
    //         var path = Path.Join(AppContext.BaseDirectory, "libblastcap.so");
    //         try {
    //             var lib = NativeLibrary.Load(path);
    //             return lib;
    //         } catch (Exception e) {
    //             OS.Crash($"Tried to load \"{libraryName}\" but got error: \"{e}\"");
    //         }
    //     }
    //     return IntPtr.Zero;
    // }
}