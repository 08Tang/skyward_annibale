package com.tkexig.skyward.data.skyward

import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test

/**
 * base64 解码。
 *
 * 这条路径是配置能不能读到的关键：文件经 shell 用 base64 传回来，解错了
 * 就是「解出 0 字节」或者乱码，界面只能显示默认值。用标准向量钉住它。
 */
class Base64CodecTest {

    private fun decode(s: String) = String(Base64Codec.decode(s), Charsets.UTF_8)

    @Test
    fun decodesTheStandardVectors() {
        // RFC 4648 里的例子。
        assertEquals("", decode(""))
        assertEquals("f", decode("Zg=="))
        assertEquals("fo", decode("Zm8="))
        assertEquals("foo", decode("Zm9v"))
        assertEquals("foob", decode("Zm9vYg=="))
        assertEquals("fooba", decode("Zm9vYmE="))
        assertEquals("foobar", decode("Zm9vYmFy"))
    }

    @Test
    fun decodesEveryByteValue() {
        val bytes = ByteArray(256) { it.toByte() }
        val encoded = java.util.Base64.getEncoder().encodeToString(bytes)
        assertEquals(bytes.toList(), Base64Codec.decode(encoded).toList())
    }

    @Test
    fun ignoresLineBreaks() {
        // base64 不带 -w0 时会折行，折出来的换行不该让解码失败。
        assertEquals("foobar", decode("Zm9v\nYmFy"))
        assertEquals("foobar", decode("Zm9v\r\nYmFy"))
        assertEquals("foobar", decode("  Zm9v YmFy  "))
    }

    @Test
    fun acceptsTheUrlSafeAlphabet() {
        val bytes = ByteArray(256) { it.toByte() }
        val urlSafe = java.util.Base64.getUrlEncoder().withoutPadding().encodeToString(bytes)
        assertEquals(bytes.toList(), Base64Codec.decode(urlSafe).toList())
    }

    /** 这份 JSON 就是真实配置的形状，含中文和缩进。 */
    @Test
    fun decodesARealSettingsFile() {
        val json = """
            {
              "language": "zh-Hans",
              "screen_off": 15,
              "thermal_limit": 46,
              "charge_mode": "fastest"
            }
        """.trimIndent()
        val encoded = java.util.Base64.getEncoder().encodeToString(json.toByteArray())
        val back = decode(encoded)
        assertEquals(json, back)

        val settings = SkywardSettings.json.decodeFromString<SkywardSettings>(back)
        assertEquals(15L, settings.screenOff)
        assertEquals(46, settings.thermalLimit)
        assertEquals("fastest", settings.chargeMode)
    }

    @Test
    fun rejectsCharactersOutsideTheAlphabet() {
        assertThrows(IllegalArgumentException::class.java) { Base64Codec.decode("Zm9v!!!") }
        assertThrows(IllegalArgumentException::class.java) { Base64Codec.decode("中文") }
    }

    /**
     * 空输入解出空串，不是崩溃。
     *
     * 文件读不到时走的就是这条路，调用方据此报「文件是空的」。
     */
    @Test
    fun emptyInputDecodesToNothing() {
        assertEquals("", decode(""))
        assertEquals("", decode("   \n  "))
    }
}
