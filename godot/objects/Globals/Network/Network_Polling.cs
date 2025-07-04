using System.Runtime.InteropServices;
using System;
using Godot;
using MessagePack;
using System.Collections.Generic;

public partial class NetworkClient {
    public delegate void PongCallback();
    private delegate void PongCallbackRaw();
    public delegate void ChatMessageCallback(string ChatMessage_arg0, string ChatMessage_arg1);
    private delegate void ChatMessageCallbackRaw([MarshalAs(UnmanagedType.LPArray, SizeParamIndex=1)] byte[] ChatMessage_arg0, UInt32 ChatMessage_arg0_len, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex=3)] byte[] ChatMessage_arg1, UInt32 ChatMessage_arg1_len);
    public delegate void NewUserCallback(string NewUser_arg0);
    private delegate void NewUserCallbackRaw([MarshalAs(UnmanagedType.LPArray, SizeParamIndex=1)] byte[] NewUser_arg0, UInt32 NewUser_arg0_len);
    public delegate void UserLeftCallback(string UserLeft_arg0);
    private delegate void UserLeftCallbackRaw([MarshalAs(UnmanagedType.LPArray, SizeParamIndex=1)] byte[] UserLeft_arg0, UInt32 UserLeft_arg0_len);
    public delegate void PlayerListCallback(List<string> PlayerList_arg0);
    private delegate void PlayerListCallbackRaw([MarshalAs(UnmanagedType.LPArray, SizeParamIndex=1)] byte[] PlayerList_arg0, UInt32 PlayerList_arg0_len);
    public delegate void StatusCallback(UInt32 Status_user_count, float Status_tick_diff);
    private delegate void StatusCallbackRaw(UInt32 Status_user_count, float Status_tick_diff);
    public delegate void NotifyHostCallback();
    private delegate void NotifyHostCallbackRaw();
    public delegate void MapListCallback(List<string> MapList_arg0);
    private delegate void MapListCallbackRaw([MarshalAs(UnmanagedType.LPArray, SizeParamIndex=1)] byte[] MapList_arg0, UInt32 MapList_arg0_len);
    public delegate void StartMapCallback(string StartMap_arg0);
    private delegate void StartMapCallbackRaw([MarshalAs(UnmanagedType.LPArray, SizeParamIndex=1)] byte[] StartMap_arg0, UInt32 StartMap_arg0_len);
    public delegate void SpawnActorCallback(bool SpawnActor_yours, string SpawnActor_name, UInt64 SpawnActor_id, UInt64 SpawnActor_x, UInt64 SpawnActor_y, List<string> SpawnActor_abilities);
    private delegate void SpawnActorCallbackRaw(bool SpawnActor_yours, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex=2)] byte[] SpawnActor_name, UInt32 SpawnActor_name_len, UInt64 SpawnActor_id, UInt64 SpawnActor_x, UInt64 SpawnActor_y, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex=7)] byte[] SpawnActor_abilities, UInt32 SpawnActor_abilities_len);
    public delegate void YourTurnCallback(UInt64 YourTurn_actor);
    private delegate void YourTurnCallbackRaw(UInt64 YourTurn_actor);
    public delegate void ActorTurnCallback(UInt64 ActorTurn_actor);
    private delegate void ActorTurnCallbackRaw(UInt64 ActorTurn_actor);
    public delegate void MoveActorCallback(UInt64 MoveActor_actor, List<UInt64> MoveActor_x, List<UInt64> MoveActor_y);
    private delegate void MoveActorCallbackRaw(UInt64 MoveActor_actor, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex=2)] byte[] MoveActor_x, UInt32 MoveActor_x_len, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex=4)] byte[] MoveActor_y, UInt32 MoveActor_y_len);
    public delegate void AbilityMapCallback(Dictionary<string, string> AbilityMap_arg0);
    private delegate void AbilityMapCallbackRaw([MarshalAs(UnmanagedType.LPArray, SizeParamIndex=1)] byte[] AbilityMap_arg0, UInt32 AbilityMap_arg0_len);
    
    private PongCallback PongFn;
    private ChatMessageCallback ChatMessageFn;
    private NewUserCallback NewUserFn;
    private UserLeftCallback UserLeftFn;
    private PlayerListCallback PlayerListFn;
    private StatusCallback StatusFn;
    private NotifyHostCallback NotifyHostFn;
    private MapListCallback MapListFn;
    private StartMapCallback StartMapFn;
    private SpawnActorCallback SpawnActorFn;
    private YourTurnCallback YourTurnFn;
    private ActorTurnCallback ActorTurnFn;
    private MoveActorCallback MoveActorFn;
    private AbilityMapCallback AbilityMapFn;

    private event PongCallback _onPong;
    public event PongCallback OnPong
    {
        add
        {
            _onPong = null;
            _onPong += value;
        }
        remove
        {
            _onPong -= value;
        }
    }
    private event ChatMessageCallback _onChatMessage;
    public event ChatMessageCallback OnChatMessage
    {
        add
        {
            _onChatMessage = null;
            _onChatMessage += value;
        }
        remove
        {
            _onChatMessage -= value;
        }
    }
    private event NewUserCallback _onNewUser;
    public event NewUserCallback OnNewUser
    {
        add
        {
            _onNewUser = null;
            _onNewUser += value;
        }
        remove
        {
            _onNewUser -= value;
        }
    }
    private event UserLeftCallback _onUserLeft;
    public event UserLeftCallback OnUserLeft
    {
        add
        {
            _onUserLeft = null;
            _onUserLeft += value;
        }
        remove
        {
            _onUserLeft -= value;
        }
    }
    private event PlayerListCallback _onPlayerList;
    public event PlayerListCallback OnPlayerList
    {
        add
        {
            _onPlayerList = null;
            _onPlayerList += value;
        }
        remove
        {
            _onPlayerList -= value;
        }
    }
    private event StatusCallback _onStatus;
    public event StatusCallback OnStatus
    {
        add
        {
            _onStatus = null;
            _onStatus += value;
        }
        remove
        {
            _onStatus -= value;
        }
    }
    private event NotifyHostCallback _onNotifyHost;
    public event NotifyHostCallback OnNotifyHost
    {
        add
        {
            _onNotifyHost = null;
            _onNotifyHost += value;
        }
        remove
        {
            _onNotifyHost -= value;
        }
    }
    private event MapListCallback _onMapList;
    public event MapListCallback OnMapList
    {
        add
        {
            _onMapList = null;
            _onMapList += value;
        }
        remove
        {
            _onMapList -= value;
        }
    }
    private event StartMapCallback _onStartMap;
    public event StartMapCallback OnStartMap
    {
        add
        {
            _onStartMap = null;
            _onStartMap += value;
        }
        remove
        {
            _onStartMap -= value;
        }
    }
    private event SpawnActorCallback _onSpawnActor;
    public event SpawnActorCallback OnSpawnActor
    {
        add
        {
            _onSpawnActor = null;
            _onSpawnActor += value;
        }
        remove
        {
            _onSpawnActor -= value;
        }
    }
    private event YourTurnCallback _onYourTurn;
    public event YourTurnCallback OnYourTurn
    {
        add
        {
            _onYourTurn = null;
            _onYourTurn += value;
        }
        remove
        {
            _onYourTurn -= value;
        }
    }
    private event ActorTurnCallback _onActorTurn;
    public event ActorTurnCallback OnActorTurn
    {
        add
        {
            _onActorTurn = null;
            _onActorTurn += value;
        }
        remove
        {
            _onActorTurn -= value;
        }
    }
    private event MoveActorCallback _onMoveActor;
    public event MoveActorCallback OnMoveActor
    {
        add
        {
            _onMoveActor = null;
            _onMoveActor += value;
        }
        remove
        {
            _onMoveActor -= value;
        }
    }
    private event AbilityMapCallback _onAbilityMap;
    public event AbilityMapCallback OnAbilityMap
    {
        add
        {
            _onAbilityMap = null;
            _onAbilityMap += value;
        }
        remove
        {
            _onAbilityMap -= value;
        }
    }

    public void Poll()
    {
        unsafe
        {
            [DllImport("libblastcap.so", SetLastError = true)]
            static extern void client_poll(void* ptr, PongCallbackRaw PongFn, ChatMessageCallbackRaw ChatMessageFn, NewUserCallbackRaw NewUserFn, UserLeftCallbackRaw UserLeftFn, PlayerListCallbackRaw PlayerListFn, StatusCallbackRaw StatusFn, NotifyHostCallbackRaw NotifyHostFn, MapListCallbackRaw MapListFn, StartMapCallbackRaw StartMapFn, SpawnActorCallbackRaw SpawnActorFn, YourTurnCallbackRaw YourTurnFn, ActorTurnCallbackRaw ActorTurnFn, MoveActorCallbackRaw MoveActorFn, AbilityMapCallbackRaw AbilityMapFn);
            client_poll(this.inner, () => {

                this._onPong();
            }, ([MarshalAs(UnmanagedType.LPArray, SizeParamIndex=1)] byte[] ChatMessage_arg0, UInt32 ChatMessage_arg0_len, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex=3)] byte[] ChatMessage_arg1, UInt32 ChatMessage_arg1_len) => {
                var ChatMessage_arg0_conv = MessagePackSerializer.Deserialize<string>(ChatMessage_arg0);
                var ChatMessage_arg1_conv = MessagePackSerializer.Deserialize<string>(ChatMessage_arg1);

                this._onChatMessage(ChatMessage_arg0_conv, ChatMessage_arg1_conv);
            }, ([MarshalAs(UnmanagedType.LPArray, SizeParamIndex=1)] byte[] NewUser_arg0, UInt32 NewUser_arg0_len) => {
                var NewUser_arg0_conv = MessagePackSerializer.Deserialize<string>(NewUser_arg0);

                this._onNewUser(NewUser_arg0_conv);
            }, ([MarshalAs(UnmanagedType.LPArray, SizeParamIndex=1)] byte[] UserLeft_arg0, UInt32 UserLeft_arg0_len) => {
                var UserLeft_arg0_conv = MessagePackSerializer.Deserialize<string>(UserLeft_arg0);

                this._onUserLeft(UserLeft_arg0_conv);
            }, ([MarshalAs(UnmanagedType.LPArray, SizeParamIndex=1)] byte[] PlayerList_arg0, UInt32 PlayerList_arg0_len) => {
                var PlayerList_arg0_conv = MessagePackSerializer.Deserialize<List<string>>(PlayerList_arg0);

                this._onPlayerList(PlayerList_arg0_conv);
            }, (UInt32 Status_user_count, float Status_tick_diff) => {
                var Status_user_count_conv = Status_user_count;
                var Status_tick_diff_conv = Status_tick_diff;

                this._onStatus(Status_user_count_conv, Status_tick_diff_conv);
            }, () => {

                this._onNotifyHost();
            }, ([MarshalAs(UnmanagedType.LPArray, SizeParamIndex=1)] byte[] MapList_arg0, UInt32 MapList_arg0_len) => {
                var MapList_arg0_conv = MessagePackSerializer.Deserialize<List<string>>(MapList_arg0);

                this._onMapList(MapList_arg0_conv);
            }, ([MarshalAs(UnmanagedType.LPArray, SizeParamIndex=1)] byte[] StartMap_arg0, UInt32 StartMap_arg0_len) => {
                var StartMap_arg0_conv = MessagePackSerializer.Deserialize<string>(StartMap_arg0);

                this._onStartMap(StartMap_arg0_conv);
            }, (bool SpawnActor_yours, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex=2)] byte[] SpawnActor_name, UInt32 SpawnActor_name_len, UInt64 SpawnActor_id, UInt64 SpawnActor_x, UInt64 SpawnActor_y, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex=7)] byte[] SpawnActor_abilities, UInt32 SpawnActor_abilities_len) => {
                var SpawnActor_yours_conv = SpawnActor_yours;
                var SpawnActor_name_conv = MessagePackSerializer.Deserialize<string>(SpawnActor_name);
                var SpawnActor_id_conv = SpawnActor_id;
                var SpawnActor_x_conv = SpawnActor_x;
                var SpawnActor_y_conv = SpawnActor_y;
                var SpawnActor_abilities_conv = MessagePackSerializer.Deserialize<List<string>>(SpawnActor_abilities);

                this._onSpawnActor(SpawnActor_yours_conv, SpawnActor_name_conv, SpawnActor_id_conv, SpawnActor_x_conv, SpawnActor_y_conv, SpawnActor_abilities_conv);
            }, (UInt64 YourTurn_actor) => {
                var YourTurn_actor_conv = YourTurn_actor;

                this._onYourTurn(YourTurn_actor_conv);
            }, (UInt64 ActorTurn_actor) => {
                var ActorTurn_actor_conv = ActorTurn_actor;

                this._onActorTurn(ActorTurn_actor_conv);
            }, (UInt64 MoveActor_actor, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex=2)] byte[] MoveActor_x, UInt32 MoveActor_x_len, [MarshalAs(UnmanagedType.LPArray, SizeParamIndex=4)] byte[] MoveActor_y, UInt32 MoveActor_y_len) => {
                var MoveActor_actor_conv = MoveActor_actor;
                var MoveActor_x_conv = MessagePackSerializer.Deserialize<List<UInt64>>(MoveActor_x);
                var MoveActor_y_conv = MessagePackSerializer.Deserialize<List<UInt64>>(MoveActor_y);

                this._onMoveActor(MoveActor_actor_conv, MoveActor_x_conv, MoveActor_y_conv);
            }, ([MarshalAs(UnmanagedType.LPArray, SizeParamIndex=1)] byte[] AbilityMap_arg0, UInt32 AbilityMap_arg0_len) => {
                var AbilityMap_arg0_conv = MessagePackSerializer.Deserialize<Dictionary<string, string>>(AbilityMap_arg0);

                this._onAbilityMap(AbilityMap_arg0_conv);
            });
        }
    }
    public NetworkClient([MarshalAs(UnmanagedType.LPUTF8Str)] string addr, OnFail onFail)
    {
        [DllImport("libblastcap.so", SetLastError = true)]
        static extern unsafe void* start_client_loop([MarshalAs(UnmanagedType.LPUTF8Str)] string addr, OnFail onFail);
        unsafe
        {
            void* ptr = start_client_loop(addr, onFail);
            this.inner = ptr;
            
            this.PongFn = () => _onPong();
            this.OnPong += () => {GD.PrintErr("Using default event handler for OnPong, please set it!");};
            this.ChatMessageFn = (ChatMessage_arg0, ChatMessage_arg1) => _onChatMessage(ChatMessage_arg0, ChatMessage_arg1);
            this.OnChatMessage += (ChatMessage_arg0, ChatMessage_arg1) => {GD.PrintErr("Using default event handler for OnChatMessage, please set it!");};
            this.NewUserFn = (NewUser_arg0) => _onNewUser(NewUser_arg0);
            this.OnNewUser += (NewUser_arg0) => {GD.PrintErr("Using default event handler for OnNewUser, please set it!");};
            this.UserLeftFn = (UserLeft_arg0) => _onUserLeft(UserLeft_arg0);
            this.OnUserLeft += (UserLeft_arg0) => {GD.PrintErr("Using default event handler for OnUserLeft, please set it!");};
            this.PlayerListFn = (PlayerList_arg0) => _onPlayerList(PlayerList_arg0);
            this.OnPlayerList += (PlayerList_arg0) => {GD.PrintErr("Using default event handler for OnPlayerList, please set it!");};
            this.StatusFn = (Status_user_count, Status_tick_diff) => _onStatus(Status_user_count, Status_tick_diff);
            this.OnStatus += (Status_user_count, Status_tick_diff) => {GD.PrintErr("Using default event handler for OnStatus, please set it!");};
            this.NotifyHostFn = () => _onNotifyHost();
            this.OnNotifyHost += () => {GD.PrintErr("Using default event handler for OnNotifyHost, please set it!");};
            this.MapListFn = (MapList_arg0) => _onMapList(MapList_arg0);
            this.OnMapList += (MapList_arg0) => {GD.PrintErr("Using default event handler for OnMapList, please set it!");};
            this.StartMapFn = (StartMap_arg0) => _onStartMap(StartMap_arg0);
            this.OnStartMap += (StartMap_arg0) => {GD.PrintErr("Using default event handler for OnStartMap, please set it!");};
            this.SpawnActorFn = (SpawnActor_yours, SpawnActor_name, SpawnActor_id, SpawnActor_x, SpawnActor_y, SpawnActor_abilities) => _onSpawnActor(SpawnActor_yours, SpawnActor_name, SpawnActor_id, SpawnActor_x, SpawnActor_y, SpawnActor_abilities);
            this.OnSpawnActor += (SpawnActor_yours, SpawnActor_name, SpawnActor_id, SpawnActor_x, SpawnActor_y, SpawnActor_abilities) => {GD.PrintErr("Using default event handler for OnSpawnActor, please set it!");};
            this.YourTurnFn = (YourTurn_actor) => _onYourTurn(YourTurn_actor);
            this.OnYourTurn += (YourTurn_actor) => {GD.PrintErr("Using default event handler for OnYourTurn, please set it!");};
            this.ActorTurnFn = (ActorTurn_actor) => _onActorTurn(ActorTurn_actor);
            this.OnActorTurn += (ActorTurn_actor) => {GD.PrintErr("Using default event handler for OnActorTurn, please set it!");};
            this.MoveActorFn = (MoveActor_actor, MoveActor_x, MoveActor_y) => _onMoveActor(MoveActor_actor, MoveActor_x, MoveActor_y);
            this.OnMoveActor += (MoveActor_actor, MoveActor_x, MoveActor_y) => {GD.PrintErr("Using default event handler for OnMoveActor, please set it!");};
            this.AbilityMapFn = (AbilityMap_arg0) => _onAbilityMap(AbilityMap_arg0);
            this.OnAbilityMap += (AbilityMap_arg0) => {GD.PrintErr("Using default event handler for OnAbilityMap, please set it!");};
        }
    }
}