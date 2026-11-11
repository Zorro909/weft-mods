package javahello;

import org.graalvm.word.Pointer;

/** `/hellojava`: anyone may run it, and Java answers with the version it runs on. */
final class HelloCommand {
    private static final int EVERYONE = 0;

    private HelloCommand() {}

    static void register(Pointer registrar) {
        Host.command(registrar, "hellojava", EVERYONE, HelloCommand::run);
    }

    private static void run(Pointer commandArgs) {
        Host.reply(commandArgs, "javahello.hello", Runtime.version().toString());
    }
}
