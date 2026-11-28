package com.tkexig.skyward.data.skyward

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * 解码一份真实的 Skyward settings.json。
 *
 * 这份 JSON 是从 K90 上导出来的，`screen_off` 是 15、`thermal_limit` 是 46、
 * 备份只勾了 modem。断言逐个字段，而不是「解得出来就行」——字段名写错时
 * `ignoreUnknownKeys` 不会报错，只会静悄悄按默认值补上，那正是之前灭屏和
 * 温控读错的样子。
 */
class SkywardSettingsDecodeTest {

    private val real = """
        {
          "language": "zh-Hans",
          "spoken": "zh-Hans",
          "dark": false,
          "brightness": 0.14069021,
          "screen_off": 15,
          "follow_system_time_zone": true,
          "time_zone": "Asia/Shanghai",
          "system_time_zone": null,
          "vibrate": true,
          "vibration": 0.39777744,
          "file_transfer": false,
          "charge_mode": "fastest",
          "thermal_guard": true,
          "thermal_limit": 46,
          "install": { "check_signature": false, "keep_recovery": true },
          "backup": { "parts": ["modem"], "to": "storage", "compress": false, "verify": true },
          "logs": { "kinds": ["recovery", "kernel"], "to": "temporary" }
        }
    """.trimIndent()

    private fun decode(text: String): SkywardSettings =
        SkywardSettings.json.decodeFromString(text)

    @Test
    fun readsTheValuesTheFileHolds() {
        val s = decode(real)

        assertEquals(15L, s.screenOff)
        assertEquals(46, s.thermalLimit)
        assertEquals("fastest", s.chargeMode)
        assertEquals(0.14069021f, s.brightness, 1e-6f)
        assertEquals(0.39777744f, s.vibration, 1e-6f)
        assertEquals("zh-Hans", s.language)
        assertEquals("Asia/Shanghai", s.timeZone)
        assertTrue(s.followSystemTimeZone)
        assertTrue(s.vibrate)
        assertEquals(false, s.fileTransfer)
        assertNull(s.systemTimeZone)
    }

    @Test
    fun readsTheNestedObjects() {
        val s = decode(real)

        assertEquals(false, s.install.checkSignature)
        assertEquals(true, s.install.keepRecovery)

        assertEquals(listOf("modem"), s.backup.parts)
        assertEquals("storage", s.backup.to)
        assertEquals(false, s.backup.compress)
        assertEquals(true, s.backup.verify)

        assertEquals(listOf("recovery", "kernel"), s.logs.kinds)
        assertEquals("temporary", s.logs.to)
    }

    /**
     * 每一项都得是文件里的值，不能有一项落回默认值。
     *
     * 上面那几条断言是逐项写的；这一条用默认值反过来查：如果哪次改字段名
     * 忘了 `@SerialName`，解码不会失败，但值会变成默认值，这里就会红。
     */
    @Test
    fun nothingFallsBackToADefault() {
        val s = decode(real)
        val d = SkywardSettings()

        // 这几项在文件里的值和默认值恰好不同，任何一个相等就说明没读到。
        assertTrue("screenOff", s.screenOff != d.screenOff)
        assertTrue("thermalLimit", s.thermalLimit != d.thermalLimit)
        assertTrue("chargeMode", s.chargeMode != d.chargeMode)
        assertTrue("brightness", s.brightness != d.brightness)
        assertTrue("vibration", s.vibration != d.vibration)
        assertTrue("fileTransfer", s.fileTransfer != d.fileTransfer)
        assertTrue("backup.parts", s.backup.parts != d.backup.parts)
        assertTrue("logs.to", s.logs.to != d.logs.to)
        assertTrue("install.checkSignature", s.install.checkSignature != d.install.checkSignature)
    }

    /**
     * 每一项的 JSON 名都对得上 Rust 那边。
     *
     * 写回去再看键名：`encodeDefaults` 让每个字段都出现，所以缺一个键就是
     * 名字错了。
     */
    @Test
    fun writesEveryNameTheRustSideReads() {
        val text = SkywardSettings.json.encodeToString(SkywardSettings.serializer(), SkywardSettings())
        val expected = listOf(
            "language", "spoken", "dark", "brightness", "screen_off",
            "follow_system_time_zone", "time_zone", "system_time_zone",
            "vibrate", "vibration", "file_transfer", "charge_mode",
            "thermal_guard", "thermal_limit", "auto_root", "install",
            "backup", "logs", "wlan_static",
        )
        for (key in expected) {
            assertTrue("missing $key in $text", text.contains("\"$key\""))
        }
        for (key in listOf("check_signature", "keep_recovery", "parts", "kinds")) {
            assertTrue("missing $key in $text", text.contains("\"$key\""))
        }
    }

    /**
     * 缺字段的旧文件照样能读。
     *
     * Rust 那边整个结构体带 `#[serde(default)]`，缺的按 recovery 的默认值补，
     * 不是整份作废。
     */
    @Test
    fun readsAFileMissingFields() {
        val s = decode("""{"dark": true}""")
        assertTrue(s.dark)
        // 灭屏默认是 2 分钟（SCREEN_OFF[3]），不是 0。
        assertEquals(120L, s.screenOff)
    }

    @Test
    fun screenOffDefaultMatchesTheRecovery() {
        // State::default() 里 screen_off: 3，即 SCREEN_OFF[3] = 120_000ms = 2 分钟。
        assertEquals(120L, SkywardSettings().screenOff)
    }

    @Test
    fun autoRootDefaultMatchesTheRecovery() {
        // AutoRoot::None 的名字是 "none"，不是空串。
        assertEquals("none", SkywardSettings().autoRoot)
    }

    @Test
    fun sliderDefaultsMatchTheRecovery() {
        // State::default() 里 brightness 和 vibration 都是 SLIDER_START = 0.1。
        assertEquals(0.1f, SkywardSettings().brightness, 1e-6f)
        assertEquals(0.1f, SkywardSettings().vibration, 1e-6f)
    }

    @Test
    fun logsToDefaultMatchesTheRecovery() {
        // State::default() 里 logs_to: 0，即 LOG_PLACES[0] = "temporary"。
        assertEquals("temporary", SkywardSettings().logs.to)
    }

    @Test
    fun thermalLimitDefaultMatchesTheRecovery() {
        assertEquals(45, SkywardSettings().thermalLimit)
    }
}
