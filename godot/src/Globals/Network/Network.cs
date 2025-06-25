using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Godot;

public partial class NetworkClient
{
    private static bool _isHost;
    public static bool IsHost { get => _isHost; }
    unsafe void* inner;
    ~NetworkClient()
    {
        GD.PrintErr("Ouchi!");
        System.Environment.Exit(1);
    }

    public delegate void OnFail([MarshalAs(UnmanagedType.LPUTF8Str)] string error);

    private static bool _success = true;
    public static bool StartHostLoop(short port)
    {
        _success = true;
        _isHost = true;
        [DllImport("../target/debug/libblastcap.so", SetLastError = true)]
        static extern void start_host_loop(Int16 port, OnFail onFail);

        start_host_loop(port, (err) =>
        {
            _success = false;
            _isHost = false;
            GD.PrintErr($"SERVER - {err}");
        });

        System.Threading.Thread.Sleep(50);
        return _success;
    }

    public void Drop()
    {
        unsafe
        {
            [DllImport("../target/debug/libblastcap.so", SetLastError = true)]
            static extern void client_drop_handle(void* inner);

            client_drop_handle(this.inner);
            _isHost = false;
        }
    }
}

