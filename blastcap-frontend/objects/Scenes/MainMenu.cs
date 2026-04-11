using System.Collections.Generic;
using Godot;

public partial class MainMenu : Node3D {
    private NetworkManager nw;

    [Export]
    public ChatBox ChatBox;

    [Export]
    public VBoxContainer PlayerList;

    [Export]
    public VBoxContainer MapList;

    [Export]
    public Button HostButton;

    [Export]
    public Button ConnectButton;

    [Export]
    public Label UserNameLabel;

    [Export]
    public TextureRect UserAvatar;

    [Export]
    public Control MainMenuNode;

    [Export]
    public Button LaunchGameButton;

    private void DrawPlayerList() {
        foreach (var child in this.PlayerList.GetChildren()) {
            child.QueueFree();
        }

        foreach (var player in this.nw.Players) {
            var lab = new Label {
                Text = $"{player}"
            };
            this.PlayerList.AddChild(lab);
        }
    }

    private void OnConnect() {
        this.nw.Inner.OnChatMessage += (user, msg) => this.ChatBox.ShowMessage($"{user}: {msg}");
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
        this.nw.Inner.OnStatus += (count, diff) => { };
        this.nw.Inner.OnAbilityMap += (map) => { Data.AbilitiesOld = map; };
        this.nw.Inner.OnPlayerList += (playerList) => {
            this.nw.Players.Clear();
            foreach (var player in playerList) {
                this.nw.Players.Add(player);
            }

            this.DrawPlayerList();
        };
        this.nw.Inner.OnNotifyHost += () => {
            this.LaunchGameButton.Visible = true;
        };
        this.LaunchGameButton.Pressed += this.nw.Inner.SendChangeToEnterDungeon;
        this.nw.Inner.OnMapList += (list) => {
            this.MapList.Visible = true;
            foreach (var map in list) {
                var button = new Button {
                    Text = map
                };
                button.Pressed += () => {
                    this.nw.Inner.SendStartMap(map);
                };
                this.MapList.AddChild(button);
            }
        };
        this.nw.Inner.OnEnterDungeonState += () => {
            this.GetTree().ChangeSceneToFile("uid://f1axvwf5favr");
        };
        this.nw.Inner.OnStartMap += (map) => {
            GD.Print($"Starting map: {map}");
            this.GetTree().ChangeSceneToFile("uid://bjmfx6nsekf58");
        };
    }

    public override void _Ready() {
        GD.Print("==================================");
        this.nw = this.GetNode<NetworkManager>("/root/NetworkManager");

        this.HostButton.Pressed += () => {
            if (NetworkClient.StartHostLoop()) {
                this.nw.Connect("localhost");
                this.OnConnect();
            }
        };

        this.ConnectButton.Pressed += () => {
            this.nw.Connect("localhost");
            this.OnConnect();
        };

        this.UserNameLabel.Text = this.nw.Inner.GetName(this.nw.Inner.GetMyId());
        this.nw.Inner.GetAvatar(this.nw.Inner.GetMyId(), (data, width, height) => {
            var img = Image.CreateFromData(width, height, false, Image.Format.Rgba8, data);
            var texture = ImageTexture.CreateFromImage(img);
            this.UserAvatar.Texture = texture;
        });
    }


    public override void _Process(double delta) {
    }
}
