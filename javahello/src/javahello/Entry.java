package javahello;

import org.graalvm.nativeimage.IsolateThread;
import org.graalvm.nativeimage.UnmanagedMemory;
import org.graalvm.nativeimage.c.function.CEntryPoint;
import org.graalvm.nativeimage.c.function.CFunctionPointer;
import org.graalvm.word.Pointer;

/** Where the engine first reaches Java, through `native/entry.c`. */
final class Entry {
    static final String ID = "javahello";
    static final String VERSION = "0.1.0";

    private Entry() {}

    /**
     * The engine's `ModDecl` for this mod: `abi_version: u64`, `id: Str`, `version: Str`,
     * `deps: Slice<Str>`, `init`. It stays in memory for as long as the game runs.
     */
    @CEntryPoint(name = "javahello_decl")
    static Pointer decl(IsolateThread thread, CFunctionPointer init) {
        Pointer decl = UnmanagedMemory.calloc(64);
        decl.writeLong(0, Abi.VERSION);
        Text.lasting(ID).writeTo(decl, 8);
        Text.lasting(VERSION).writeTo(decl, 24);
        // No dependencies: an empty slice, whose pointer must still not be null.
        decl.writeWord(40, decl);
        decl.writeLong(48, 0);
        decl.writeWord(56, init);
        return decl;
    }

    /** Registers the mod's content; 0 when that went well. */
    @CEntryPoint(name = "javahello_init")
    static int init(IsolateThread thread, Pointer host, int me, Pointer registrar, CFunctionPointer callback) {
        try {
            Host.use(host, me);
            Callbacks.useTrampoline(callback);
            HelloCommand.register(registrar);
            Host.log("Hello from Java " + Runtime.version() + ", compiled by GraalVM native-image");
            return 0;
        } catch (Throwable e) {
            e.printStackTrace();
            return 1;
        }
    }
}
