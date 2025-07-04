using System.Runtime.InteropServices;
using Godot;

public partial class NetworkClient {
    public static bool IsHost { get; private set; }
    private readonly unsafe void* _inner;
    ~NetworkClient() {
        GD.PrintErr("Ouch!");
        System.Environment.Exit(1);
    }

    public delegate void OnFail([MarshalAs(UnmanagedType.LPUTF8Str)] string error);

    private static bool _success = true;
    public static bool StartHostLoop(short port) {
        _success = true;
        IsHost = true;
        [DllImport("libblastcap.so", SetLastError = true)]
        static extern void start_host_loop(short port, OnFail onFail);

        start_host_loop(port, (err) => {
            _success = false;
            IsHost = false;
            GD.PrintErr($"SERVER - {err}");
        });

        System.Threading.Thread.Sleep(50);
        return _success;
    }

    public void Drop() {
        unsafe {
            [DllImport("libblastcap.so", SetLastError = true)]
            static extern void client_drop_handle(void* inner);

            client_drop_handle(this._inner);
            IsHost = false;
        }
    }
}

