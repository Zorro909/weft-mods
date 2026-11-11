import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

/**
 * Prints `Abi.java`, what this mod must know of the engine's ABI, read from Weft's
 * `crates/abi/src`: the version hash a mod states so that the engine loads it, and where in the
 * `Host` table each function the mod calls is. Run by `build.sh`:
 *
 *     java tools/GenerateAbi.java Weft/crates/abi/src > Abi.java
 */
public class GenerateAbi {
    /** The `Host` functions the mod calls, by their names there. */
    static final List<String> CALLED = List.of("log", "reg_command", "reg_command_permission", "command_reply");

    public static void main(String[] args) throws IOException {
        Path src = Path.of(args[0]);
        String lib = Files.readString(src.resolve("lib.rs"));
        long version = fnv(concat(src.resolve("lib.rs"), src.resolve("data.rs")));
        List<String> host = hostFields(lib);

        StringBuilder out = new StringBuilder();
        out.append("package javahello;\n\n");
        out.append("/** The engine ABI, written by `tools/GenerateAbi.java` from Weft's `crates/abi/src`. */\n");
        out.append("final class Abi {\n");
        out.append("    /** `ABI_VERSION`: the engine loads only a mod that states its own. */\n");
        out.append("    static final long VERSION = 0x%016xL;\n".formatted(version));
        for (String name : CALLED) {
            int index = host.indexOf(name);
            if (index < 0) {
                throw new IllegalStateException("Host has no function " + name);
            }
            out.append("    /** Byte offset of `Host::%s`. */\n".formatted(name));
            out.append("    static final int %s = %d;\n".formatted(name.toUpperCase(), index * 8));
        }
        out.append("\n    private Abi() {}\n}\n");
        System.out.print(out);
    }

    /** `abi::fnv`: FNV-1a, 64 bits. */
    static long fnv(byte[] bytes) {
        long hash = 0xcbf29ce484222325L;
        for (byte b : bytes) {
            hash = (hash ^ (b & 0xff)) * 0x100000001b3L;
        }
        return hash;
    }

    static byte[] concat(Path... files) throws IOException {
        ByteArrayOutputStream all = new ByteArrayOutputStream();
        for (Path file : files) {
            all.write(Files.readAllBytes(file));
        }
        return all.toByteArray();
    }

    /**
     * The names of `Host`'s fields in order. Each is a function pointer, 8 bytes, so a field's
     * offset is 8 times its index; a field of any other kind fails the build instead.
     */
    static List<String> hostFields(String lib) {
        int start = lib.indexOf("\npub struct Host {\n");
        int end = lib.indexOf("\n}\n", start);
        List<String> names = new ArrayList<>();
        String field = null;
        for (String line : lib.substring(start, end).split("\n")) {
            if (line.startsWith("    pub ")) {
                checkFunction(field);
                field = line.trim();
                names.add(field.substring("pub ".length(), field.indexOf(':')));
            } else if (field != null && !line.trim().startsWith("//")) {
                field += line.trim();
            }
        }
        checkFunction(field);
        return names;
    }

    static void checkFunction(String field) {
        if (field != null && !field.contains("extern \"C\" fn(")) {
            throw new IllegalStateException("Host field is not a function pointer: " + field);
        }
    }
}
