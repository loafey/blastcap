using Godot;
using System;
using static Godot.Control;

public partial class LobbyScreen : Node {
    private NetworkManager nw;

    [Export]
    public ChatBox ChatBox;

    [Export]
    public VBoxContainer PlayerList;

    [Export]
    public SubViewport Viewport;

    [Export]
    public Panel ServerSettings;

    [Export]
    public Button ServerSettingsToggle;

    [Export]
    public SpinBox SettingsBotAmount;

    [Export]
    public CheckButton ReadyButton;

    private void DrawPlayerList() {
        foreach (var child in this.PlayerList.GetChildren()) {
            child.QueueFree();
        }
        foreach (var player in this.nw.Players) {
            var nameLabel = new Label {
                Text = this.nw.Inner.GetName(player),
                SizeFlagsHorizontal = SizeFlags.ExpandFill,
                HorizontalAlignment = HorizontalAlignment.Fill
            };
            var readyLabel = new Label { Text = "Not Ready", Name = "ReadyLabel" };
            var avatar = new TextureRect {
                Texture = ImageTexture.CreateFromImage(
                    Image.LoadFromFile("uid://bynqq3gi3gdtv")
                ),
                ExpandMode = TextureRect.ExpandModeEnum.FitWidth
            };
            this.nw.Inner.GetAvatar(player, (data, width, height) => {
                var img = Image.CreateFromData(width, height, false, Image.Format.Rgba8, data);
                var texture = ImageTexture.CreateFromImage(img);
                avatar.Texture = texture;
            });
            var container = new HBoxContainer {
                Name = $"{player}",
                CustomMinimumSize = new Vector2(0, 64)
            };
            container.AddChild(avatar);
            container.AddChild(nameLabel);
            container.AddChild(readyLabel);
            this.PlayerList.AddChild(container);
        }
    }

    private void FixViewPortSize() {
        var size = this.GetViewport().GetVisibleRect().End;
        var ratio = 16f / 7f;
        var min = size.X;
        var max = size.Y * ratio;
        this.Viewport.Size = new Vector2I((int)min, (int)max);
    }

    public override void _Ready() {
        base._Ready();
        this.nw = this.GetNode<NetworkManager>("/root/NetworkManager");
        this.nw.Inner.OnServerNotice += (msg) => {
            this.ChatBox.ShowMessage($"SERVER: {msg}");
        };
        this.SettingsBotAmount.ValueChanged += (value) => {
            this.nw.Inner.SendChangeDungeonSetting(0, (uint)value);
        };
        this.DrawPlayerList();
        this.FixViewPortSize();
        this.GetViewport().SizeChanged += this.FixViewPortSize;
        this.ServerSettings.Visible = false;
        this.ServerSettingsToggle.Visible = this.nw.IsHost;
        this.ServerSettingsToggle.Pressed += () => {
            this.ServerSettings.Visible = !this.ServerSettings.Visible;
        };

        this.ReadyButton.Toggled += (val) => {
            this.nw.Inner.SendNotifyReady(val ? (byte)1 : (byte)0);
        };

        this.nw.Inner.OnReadyStatus += (user, status) => {
            this.PlayerList.GetNode<Label>($"{user}/ReadyLabel").Text =
                status == 0 ? "Not Ready" : "Ready";
        };

        this.nw.Inner.OnPlayerList += (playerList) => {
            this.nw.Players.Clear();
            foreach (var player in playerList) {
                this.nw.Players.Add(player);
            }

            this.DrawPlayerList();
        };
        this.nw.Inner.OnNewUser += (user) => {
            this.ChatBox.ShowMessage($"{user} joined");
            this.nw.Players.Add(user);
            this.DrawPlayerList();
        };
        this.nw.Inner.OnUserLeft += (user) => {
            this.ChatBox.ShowMessage($"{user} left");
            this.nw.Players.Remove(user);
            this.DrawPlayerList();
        };
        this.nw.Inner.OnChatMessage += (id, msg) => {
            var name = this.nw.Inner.GetName(id);
            this.ChatBox.ShowMessage($"{name}: {msg}");
        };
        this.nw.Inner.OnEnterClearRoomState += () => {
            this.GetTree().ChangeSceneToFile("uid://bjmfx6nsekf58");
        };
    }
}
