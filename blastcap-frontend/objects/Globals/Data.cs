using MessagePack;
using System.Collections.Generic;

public static class Data {
    public static Dictionary<string, string> Abilities { get; set; }

    public static Dictionary<ulong, Card> Cards { get; set; } = [];
    [MessagePackObject]
    public class Card {
        [Key(0)]
        public string Name { get; set; }
    }
}