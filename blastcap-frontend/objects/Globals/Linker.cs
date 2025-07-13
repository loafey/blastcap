using Godot;
using System;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;

public partial class Linker : Node {
    // public override void _Ready() {
    //     var assembly = Assembly.GetExecutingAssembly();
    //     NativeLibrary.SetDllImportResolver(assembly, DllImportResolver);
    // }
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