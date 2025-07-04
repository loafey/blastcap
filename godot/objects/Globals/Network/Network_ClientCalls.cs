using System;
using MessagePack;
using System.Runtime.InteropServices;

public partial class NetworkClient {
public void SendPing() {
    unsafe {
        [DllImport("libblastcap.so", SetLastError = true)]
        static extern void client_send_ping(void* ptr);
        
        client_send_ping(this.inner);
    }
}

public void SendChatMessage(string arg0) {
    unsafe {
        [DllImport("libblastcap.so", SetLastError = true)]
        static extern void client_send_chat_message(void* ptr, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex=1)] byte[] arg0, UInt32 arg0_len);
        byte[] arg0_conv = MessagePackSerializer.Serialize(arg0);
        UInt32 arg0_len = (UInt32)arg0_conv.Length;
        client_send_chat_message(this.inner, arg0_conv, arg0_len);
    }
}

public void SendRequestMapList() {
    unsafe {
        [DllImport("libblastcap.so", SetLastError = true)]
        static extern void client_send_request_map_list(void* ptr);
        
        client_send_request_map_list(this.inner);
    }
}

public void SendStartMap(string arg0) {
    unsafe {
        [DllImport("libblastcap.so", SetLastError = true)]
        static extern void client_send_start_map(void* ptr, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex=1)] byte[] arg0, UInt32 arg0_len);
        byte[] arg0_conv = MessagePackSerializer.Serialize(arg0);
        UInt32 arg0_len = (UInt32)arg0_conv.Length;
        client_send_start_map(this.inner, arg0_conv, arg0_len);
    }
}

public void SendNotifyReady() {
    unsafe {
        [DllImport("libblastcap.so", SetLastError = true)]
        static extern void client_send_notify_ready(void* ptr);
        
        client_send_notify_ready(this.inner);
    }
}

public void SendAction(string arg0, UInt64 arg1, UInt64 arg2) {
    unsafe {
        [DllImport("libblastcap.so", SetLastError = true)]
        static extern void client_send_action(void* ptr, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex=1)] byte[] arg0, UInt32 arg0_len, UInt64 arg1, UInt64 arg2);
        byte[] arg0_conv = MessagePackSerializer.Serialize(arg0);
        UInt32 arg0_len = (UInt32)arg0_conv.Length;var arg1_conv = arg1;var arg2_conv = arg2;
        client_send_action(this.inner, arg0_conv, arg0_len, arg1, arg2);
    }
}

public void SendEndTurn() {
    unsafe {
        [DllImport("libblastcap.so", SetLastError = true)]
        static extern void client_send_end_turn(void* ptr);
        
        client_send_end_turn(this.inner);
    }
}
}