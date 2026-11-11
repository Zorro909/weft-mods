// The engine calls a mod through plain C functions: `weft_mod`, the `init` it hands back, and
// the callbacks the mod registers. Java code compiled by native-image runs only on a thread
// attached to a GraalVM isolate, so these make that isolate and pass each call on to Java with
// such a thread (`Entry.java`, `Callbacks.java`). native-image links this file into the mod's
// library.
#include <stddef.h>
#include <stdint.h>

typedef struct graal_isolate_t graal_isolate_t;
typedef struct graal_isolatethread_t graal_isolatethread_t;

// From native-image's runtime and the Java code, in the same library.
int graal_create_isolate(void *params, graal_isolate_t **isolate, graal_isolatethread_t **thread);
int graal_attach_thread(graal_isolate_t *isolate, graal_isolatethread_t **thread);
const void *javahello_decl(graal_isolatethread_t *thread, void *init);
uint32_t javahello_init(graal_isolatethread_t *thread, const void *host, uint32_t me, void *reg, void *callback);
uint32_t javahello_callback(graal_isolatethread_t *thread, uint64_t handler, void *args);

static graal_isolate_t *isolate;
static const void *decl;

// Every callback the Java code registers: `data` is which of its handlers to run.
static uint32_t callback(const void *data, void *args) {
    graal_isolatethread_t *thread;
    if (graal_attach_thread(isolate, &thread) != 0) {
        return 1;
    }
    return javahello_callback(thread, (uint64_t)(uintptr_t)data, args);
}

static uint32_t init(const void *host, uint32_t me, void *reg) {
    graal_isolatethread_t *thread;
    if (graal_attach_thread(isolate, &thread) != 0) {
        return 1;
    }
    return javahello_init(thread, host, me, reg, (void *)callback);
}

__attribute__((visibility("default"))) const void *weft_mod(void) {
    if (decl == NULL) {
        graal_isolatethread_t *thread;
        if (graal_create_isolate(NULL, &isolate, &thread) != 0) {
            return NULL;
        }
        decl = javahello_decl(thread, (void *)init);
    }
    return decl;
}
