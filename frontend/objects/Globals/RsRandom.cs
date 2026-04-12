using System;
using System.Runtime.InteropServices;

public class RsRandom {
#pragma warning disable SYSLIB1054
    private readonly unsafe void* _inner;
    public RsRandom(ulong seed) {
        unsafe { this._inner = __random_new(seed); }
    }
    ~RsRandom() {
        unsafe { __random_drop(this._inner); }
    }

    public byte GenByte() { unsafe { return __random_gen_byte(this._inner); } }
    public byte GenByteRange(byte start, byte end) { unsafe { return __random_range_byte(this._inner, start, end); } }
    public ushort GenUShort() { unsafe { return __random_gen_ushort(this._inner); } }
    public ushort GenUShortRange(ushort start, ushort end) { unsafe { return __random_range_ushort(this._inner, start, end); } }
    public uint GenUInt() { unsafe { return __random_gen_uint(this._inner); } }
    public uint GenUIntRange(uint start, uint end) { unsafe { return __random_range_uint(this._inner, start, end); } }
    public ulong GenULong() { unsafe { return __random_gen_ulong(this._inner); } }
    public ulong GenULongRange(ulong start, ulong end) { unsafe { return __random_range_ulong(this._inner, start, end); } }

    public sbyte GenSByte() { unsafe { return __random_gen_sbyte(this._inner); } }
    public sbyte GenSByteRange(sbyte start, sbyte end) { unsafe { return __random_range_sbyte(this._inner, start, end); } }
    public short GenShort() { unsafe { return __random_gen_short(this._inner); } }
    public short GenShortRange(short start, short end) { unsafe { return __random_range_short(this._inner, start, end); } }
    public int GenInt() { unsafe { return __random_gen_int(this._inner); } }
    public int GenIntRange(int start, int end) { unsafe { return __random_range_int(this._inner, start, end); } }
    public long GenLong() { unsafe { return __random_gen_long(this._inner); } }
    public long GenLongRange(long start, long end) { unsafe { return __random_range_long(this._inner, start, end); } }

    public double GenFloat() { unsafe { return __random_gen_float(this._inner); } }
    public double GenFloatRange(float start, float end) { unsafe { return __random_range_float(this._inner, start, end); } }
    public double GenDouble() { unsafe { return __random_gen_double(this._inner); } }
    public double GenDoubleRange(double start, double end) { unsafe { return __random_range_double(this._inner, start, end); } }

    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe void* __random_new(ulong seed);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe void __random_drop(void* random);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe byte __random_gen_byte(void* random);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe byte __random_range_byte(void* random, byte start, byte end);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe ushort __random_gen_ushort(void* random);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe ushort __random_range_ushort(void* random, ushort start, ushort end);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe uint __random_gen_uint(void* random);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe uint __random_range_uint(void* random, uint start, uint end);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe ulong __random_gen_ulong(void* random);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe ulong __random_range_ulong(void* random, ulong start, ulong end);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe sbyte __random_gen_sbyte(void* random);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe sbyte __random_range_sbyte(void* random, sbyte start, sbyte end);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe short __random_gen_short(void* random);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe short __random_range_short(void* random, short start, short end);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe int __random_gen_int(void* random);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe int __random_range_int(void* random, int start, int end);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe long __random_gen_long(void* random);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe long __random_range_long(void* random, long start, long end);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe float __random_gen_float(void* random);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe float __random_range_float(void* random, float start, float end);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe double __random_gen_double(void* random);
    [DllImport("blastcap", SetLastError = true)]
    private static extern unsafe double __random_range_double(void* random, double start, double end);
#pragma warning restore SYSLIB1054
}