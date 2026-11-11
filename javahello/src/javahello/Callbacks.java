package javahello;

import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;
import org.graalvm.nativeimage.IsolateThread;
import org.graalvm.nativeimage.c.function.CEntryPoint;
import org.graalvm.nativeimage.c.function.CFunctionPointer;
import org.graalvm.word.Pointer;

/**
 * Java code the engine calls back. Each registered callback is `native/entry.c`'s trampoline
 * with the handler's index as its data; the trampoline gives the call a Java thread and comes
 * back here.
 */
final class Callbacks {
    /** One callback's code; `args` points to that callback's args struct. */
    interface Handler {
        void run(Pointer args);
    }

    private static final List<Handler> HANDLERS = new CopyOnWriteArrayList<>();
    private static CFunctionPointer trampoline;

    private Callbacks() {}

    static void useTrampoline(CFunctionPointer function) {
        trampoline = function;
    }

    /** The function the engine calls, for every handler. */
    static CFunctionPointer trampoline() {
        return trampoline;
    }

    /** Adds `handler`, and gives the data its callback must carry. */
    static long add(Handler handler) {
        HANDLERS.add(handler);
        return HANDLERS.size() - 1;
    }

    /** Runs handler `index`; 0 when it went well. */
    @CEntryPoint(name = "javahello_callback")
    static int run(IsolateThread thread, long index, Pointer args) {
        try {
            HANDLERS.get((int) index).run(args);
            return 0;
        } catch (Throwable e) {
            e.printStackTrace();
            return 1;
        }
    }
}
