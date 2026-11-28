package com.tkexig.skyward.data.skyward

/**
 * base64 解码，标准字母表，`=` 补位。
 *
 * 为什么自己写：配置要经过 shell 传回来，只能用 base64（见 [SkywardConfig.read]），
 * 而这一步最需要测试——之前「解出 0 字节」正是它和取输出的地方出的问题。
 * `android.util.Base64` 在 JVM 单元测试里是空实现，测不了；`java.util.Base64`
 * 要 API 26，而这里 minSdk 是 24。所以自己解，纯 Kotlin，两边都能跑。
 *
 * 只做解码：写回去用 `android.util.Base64` 编码，那条路不需要测。
 */
internal object Base64Codec {

    private const val ALPHABET =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"

    /** 每个字符对应的 6 位值，-1 表示不是合法字符。 */
    private val VALUES = IntArray(128) { -1 }.also { table ->
        ALPHABET.forEachIndexed { index, c -> table[c.code] = index }
    }

    /**
     * 解码。
     *
     * 容忍空白（base64 折行时会出现）和 URL 安全字母表里的 `-` `_`：真实
     * 数据里这两种都可能碰上，为此让整份配置读不出来不值得。
     *
     * @throws IllegalArgumentException 遇到字母表之外的字符时。
     */
    fun decode(text: String): ByteArray {
        val out = java.io.ByteArrayOutputStream(text.length * 3 / 4 + 3)
        var buffer = 0
        var bits = 0

        for (c in text) {
            if (c == '=') continue
            // 补位之后不该再有数据，但真出现了也照解，不因此丢掉整份。
            if (c.isWhitespace()) continue
            val value = when (c) {
                '-' -> 62
                '_' -> 63
                else -> if (c.code < 128) VALUES[c.code] else -1
            }
            require(value >= 0) { "not a base64 character: '$c'" }

            buffer = (buffer shl 6) or value
            bits += 6
            if (bits >= 8) {
                bits -= 8
                out.write((buffer shr bits) and 0xFF)
            }
        }
        return out.toByteArray()
    }
}
