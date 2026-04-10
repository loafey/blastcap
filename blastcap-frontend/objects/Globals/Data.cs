using System.Collections.Generic;

public static class Data {
    public static Dictionary<string, string> Abilities { get; set; }

    public static Dictionary<ulong, Card> Cards { get; set; } = [];

    public enum CardType : ushort {
        Projectile = 0
    }

    public class Card {
        public string Name { get; set; }

        public float? ProjectileSpeed { get; set; }

        public CardType Type { get; set; }
    }
}