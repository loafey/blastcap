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
        [DllImport("blastcap", SetLastError = true)]
        static extern void start_host_loop(short port, OnFail onFail);

        start_host_loop(port, (err) => {
            _success = false;
            IsHost = false;
            GD.PrintErr($"SERVER - {err}");
        });

        System.Threading.Thread.Sleep(50);
        return _success;
    }

    public string GetName() {
        [DllImport("blastcap", SetLastError = true)]
        static extern string get_string();
        var str = get_string();
        return str;
    }

    private static void DropString([MarshalAs(UnmanagedType.LPUTF8Str)] string str) {
        [DllImport("blastcap", SetLastError = true)]
        static extern void drop_string([MarshalAs(UnmanagedType.LPUTF8Str)] string str);
        drop_string(str);
    }

    public void Drop() {
        unsafe {
            [DllImport("blastcap", SetLastError = true)]
            static extern void client_drop_handle(void* inner);

            client_drop_handle(this._inner);
            IsHost = false;
        }
    }
}

