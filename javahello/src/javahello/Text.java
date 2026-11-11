package javahello;

import java.nio.charset.StandardCharsets;
import org.graalvm.nativeimage.UnmanagedMemory;
import org.graalvm.word.Pointer;

/** UTF-8 text in C memory, as the engine's `Str` takes it: a pointer and a length, no NUL. */
final class Text implements AutoCloseable {
    final Pointer bytes;
    final long length;

    private Text(Pointer bytes, long length) {
        this.bytes = bytes;
        this.length = length;
    }

    /** Text for one call into the engine, which reads it during the call only: close it after. */
    static Text of(String text) {
        byte[] utf8 = text.getBytes(StandardCharsets.UTF_8);
        Pointer bytes = UnmanagedMemory.malloc(Math.max(utf8.length, 1));
        for (int i = 0; i < utf8.length; i++) {
            bytes.writeByte(i, utf8[i]);
        }
        return new Text(bytes, utf8.length);
    }

    /** Text the engine keeps reading for as long as the game runs: never closed. */
    static Text lasting(String text) {
        return of(text);
    }

    /** Writes it as a `Str` at `offset` of `struct`. */
    void writeTo(Pointer struct, int offset) {
        struct.writeWord(offset, bytes);
        struct.writeLong(offset + 8, length);
    }

    @Override
    public void close() {
        UnmanagedMemory.free(bytes);
    }
}
