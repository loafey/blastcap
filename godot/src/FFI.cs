using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Godot;

public partial class NetworkClient
{
    unsafe void* inner;
    ~NetworkClient()
    {
        GD.PrintErr("Ouchi!");
        System.Environment.Exit(1);
    }

    private delegate void OnFail([MarshalAs(UnmanagedType.LPUTF8Str)] string error);

    private static bool _success = true;
    public static bool StartHostLoop(short port)
    {
        _success = true;
        [DllImport("../target/debug/libblastcap.so", SetLastError = true)]
        static extern void start_host_loop(Int16 port, OnFail onFail);

        start_host_loop(port, (err) =>
        {
            _success = false;
            GD.PrintErr($"SERVER - {err}");
        });

        System.Threading.Thread.Sleep(50);
        return _success;
    }
}

