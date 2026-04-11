using System.Collections.Generic;

public static partial class Data {
    public static Dictionary<string, string> Abilities { get; set; }

    public static Dictionary<ulong, Card> Cards { get; set; } = [];
}