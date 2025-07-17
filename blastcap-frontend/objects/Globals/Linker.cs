using Godot;
using System;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;

public partial class Linker : Node {
    public delegate void Log([MarshalAs(UnmanagedType.LPUTF8Str)] string error);

    public override void _Ready() {
        GD.Print("= Registering logging callbacks");
        [DllImport("blastcap", SetLastError = true)]
        static extern unsafe int register_logging(Log print, Log errorError);
        try {
            register_logging(GD.Print, GD.PrintErr);
        } catch (Exception e) {
            GD.PrintErr($"= Registering logging callbacks: failed registering hook: {e}");
        }
        GD.Print("= Registering logging callbacks: done");
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