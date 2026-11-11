package javahello;

import org.graalvm.nativeimage.UnmanagedMemory;
import org.graalvm.word.Pointer;

/**
 * One argument of a translated text, the engine's `UiArg`: `kind: u32` at 0, `text: Str` at 8,
 * `num: f64` at 24, `digits: u32` at 32, 40 bytes in all.
 */
final class UiArg implements AutoCloseable {
    private static final int SIZE = 40;
    private static final int KIND_TEXT = 0;

    final Pointer struct;

    private UiArg(Pointer struct) {
        this.struct = struct;
    }

    /** Plain text, shown as it is; `text` must stay open as long as this. */
    static UiArg text(Text text) {
        Pointer struct = UnmanagedMemory.calloc(SIZE);
        struct.writeInt(0, KIND_TEXT);
        text.writeTo(struct, 8);
        return new UiArg(struct);
    }

    @Override
    public void close() {
        UnmanagedMemory.free(struct);
    }
}
