package io.github.juanluci.starling

/**
 * P0's read-only launcher bridge. It is deliberately not loaded by normal game startup.
 * Calls belong on a worker: inspection reads and hashes installed mod files.
 */
object RustlingCore {
    const val SCHEMA: Int = 1

    @Volatile
    private var loaded = false

    @JvmStatic
    @Synchronized
    fun load() {
        if (!loaded) {
            System.loadLibrary("rustling_launcher")
            loaded = true
        }
    }

    @JvmStatic
    fun ping(): String {
        load()
        return dispatch("""{"command":"ping"}""")
    }

    @JvmStatic
    fun inspectInstance(
        instanceId: String,
        gameRevision: String,
        nativeHooksApi: Int?,
        instanceDir: String,
    ): String {
        require(nativeHooksApi == null || nativeHooksApi > 0) { "nativeHooksApi must be positive" }
        load()
        val hooks = nativeHooksApi?.toString() ?: "null"
        return dispatch(
            """{"command":"inspect_instance","instanceId":${quote(instanceId)},""" +
                """"gameRevision":${quote(gameRevision)},"nativeHooksApi":$hooks,""" +
                """"instanceDir":${quote(instanceDir)}}""",
        )
    }

    private fun quote(value: String): String = buildString(value.length + 2) {
        append('"')
        for (char in value) {
            when (char) {
                '"' -> append("\\\"")
                '\\' -> append("\\\\")
                '\b' -> append("\\b")
                '\u000C' -> append("\\f")
                '\n' -> append("\\n")
                '\r' -> append("\\r")
                '\t' -> append("\\t")
                else -> if (char < ' ') append("\\u%04x".format(char.code)) else append(char)
            }
        }
        append('"')
    }

    @JvmStatic
    private external fun dispatch(request: String): String
}
