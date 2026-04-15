using Godot;
using System;
using System.Runtime.InteropServices;

public class MapGen {
    private delegate void SpawnBlock(ulong x, ulong y, ulong z);
    private delegate void Done();

    private MapGenFuncs? _funcs;

    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    private readonly struct MapGenFuncs(SpawnBlock spawnBlock, Done done) {
        public readonly SpawnBlock spawnBlock = spawnBlock;
        public readonly Done done = done;
    }

    [DllImport("blastcap", SetLastError = true, CallingConvention = CallingConvention.Cdecl)]
    private static extern void __generate_map(ulong seed, MapGenFuncs funcs, ulong x, ulong y, ulong z);

    public void Generate(ulong seed, ulong x, ulong y, ulong z, Game game) {
        if (this._funcs != null) { throw new MethodAccessException("already in use"); }
        var funcs = new MapGenFuncs(
            (x, y, z) => game.SpawnCube(new((int)x, (int)y, (int)z)),
            () => this._funcs = null
        );
        this._funcs = funcs;
        GD.Print(game);
        __generate_map(seed, funcs, x, y, z);
    }
}