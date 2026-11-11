package javahello;

import org.graalvm.nativeimage.c.function.CFunctionPointer;
import org.graalvm.nativeimage.c.function.InvokeCFunctionPointer;
import org.graalvm.word.Pointer;

/**
 * The engine's calls this mod makes, from its `Host` table of function pointers
 * (`Weft/crates/abi/src/lib.rs`). A `Str`, `Slice` or `Callback` the engine takes by value is
 * two 8-byte words, which C passes just as it passes two separate arguments: each is given here
 * as its two halves.
 */
final class Host {
    private static final int LOG_INFO = 3;

    private static Pointer table;
    private static int me;

    private Host() {}

    /** The table and this mod's id, as `init` gets them. */
    static void use(Pointer host, int modId) {
        table = host;
        me = modId;
    }

    interface Log extends CFunctionPointer {
        @InvokeCFunctionPointer
        void call(int me, int level, Pointer msg, long msgLen);
    }

    interface RegCommand extends CFunctionPointer {
        @InvokeCFunctionPointer
        void call(Pointer reg, Pointer name, long nameLen, Pointer usage, long usageLen, int client,
                CFunctionPointer callback, long callbackData);
    }

    interface RegCommandPermission extends CFunctionPointer {
        @InvokeCFunctionPointer
        void call(Pointer reg, Pointer name, long nameLen, int level);
    }

    interface CommandReply extends CFunctionPointer {
        @InvokeCFunctionPointer
        void call(Pointer args, Pointer key, long keyLen, Pointer list, long listLen);
    }

    /** An info line in the game's log. */
    static void log(String message) {
        Log log = table.readWord(Abi.LOG);
        try (Text msg = Text.of(message)) {
            log.call(me, LOG_INFO, msg.bytes, msg.length);
        }
    }

    /** Chat command `/name` with no arguments, run on the server, for players of `level` and up. */
    static void command(Pointer reg, String name, int level, Callbacks.Handler handler) {
        RegCommand regCommand = table.readWord(Abi.REG_COMMAND);
        RegCommandPermission regPermission = table.readWord(Abi.REG_COMMAND_PERMISSION);
        try (Text n = Text.of(name); Text usage = Text.of("")) {
            regCommand.call(reg, n.bytes, n.length, usage.bytes, usage.length, 0, Callbacks.trampoline(), Callbacks.add(handler));
            regPermission.call(reg, n.bytes, n.length, level);
        }
    }

    /** Answers a command with the text of lang `key`, with `arg` as its `{0}`. */
    static void reply(Pointer commandArgs, String key, String arg) {
        CommandReply reply = table.readWord(Abi.COMMAND_REPLY);
        try (Text k = Text.of(key); Text a = Text.of(arg); UiArg list = UiArg.text(a)) {
            reply.call(commandArgs, k.bytes, k.length, list.struct, 1);
        }
    }
}
