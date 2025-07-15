using System;
using System.Runtime.CompilerServices;
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

    public NetworkClient(OnFail onFail) {
        [DllImport("blastcap", SetLastError = true)]
        static extern unsafe void* create_client(OnFail onFail);
        GD.Print("= Creating network client");
        unsafe { this._inner = create_client(onFail); }
        GD.Print("= Creating network client: done");
    }

    public bool IsConnected() {
        [DllImport("blastcap", SetLastError = true)]
        static extern unsafe int is_connected(void* inner);

        unsafe {
            return is_connected(this._inner) != 0;
        }
    }

    private static bool _success = true;
    public static bool StartHostLoop() {
        _success = true;
        IsHost = true;
        [DllImport("blastcap", SetLastError = true)]
        static extern void start_host_loop(OnFail onFail);

        start_host_loop((err) => {
            _success = false;
            IsHost = false;
            GD.PrintErr($"SERVER - {err}");
        });

        System.Threading.Thread.Sleep(50);
        return _success;
    }

    public string GetName(ulong id) {
        [DllImport("blastcap", SetLastError = true)]
        static extern unsafe string metadata_get_name(void* inner, ulong id);
        unsafe { return metadata_get_name(this._inner, id); }
    }

    public ulong GetMyId() {
        [DllImport("blastcap", SetLastError = true)]
        static extern unsafe ulong metadata_get_id(void* inner);
        unsafe { return metadata_get_id(this._inner); }
    }

    public delegate void AvatarCallback(byte[] data, ushort width, ushort height);
    public delegate void AvatarCallbackRaw([MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 1)] byte[] data, int len, ushort width, ushort height);
    public void GetAvatar(ulong id, AvatarCallback cb) {
        [DllImport("blastcap", SetLastError = true)]
        static extern unsafe string metadata_get_avatar(void* inner, ulong id, AvatarCallbackRaw cb);
        unsafe { metadata_get_avatar(this._inner, id, (data, _len, w, h) => cb(data, w, h)); }
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

