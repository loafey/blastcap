using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Godot;

public partial class NetworkClient
{
    unsafe void* inner;
    public unsafe NetworkClient(void* inner)
    {
        this.inner = inner;
    }
    ~NetworkClient()
    {
        GD.PrintErr("Ouchi!");
        System.Environment.Exit(1);
    }


    public static void StartHostLoop(short port)
    {
        [DllImport("../target/debug/libblastcap.so", SetLastError = true)]
        static extern void start_host_loop(Int16 port);
        start_host_loop(port);
    }


    public static NetworkClient StartClientLoop([MarshalAs(UnmanagedType.LPUTF8Str)] string addr)
    {
        [DllImport("../target/debug/libblastcap.so", SetLastError = true)]
        static extern unsafe void* start_client_loop([MarshalAs(UnmanagedType.LPUTF8Str)] string addr);
        unsafe
        {
            void* ptr = start_client_loop(addr);
            return new NetworkClient(ptr);
        }
    }

    public void SendChatMessage()
    {
        unsafe
        {
            [DllImport("../target/debug/libblastcap.so", SetLastError = true)]
            static extern void client_send_chat_msg(void* ptr);
            client_send_chat_msg(this.inner);
        }
    }
}

