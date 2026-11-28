package com.tkexig.skyward.data.skyward

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

/**
 * Skyward Recovery 的 settings.json，逐字段对应 crates/skyward-ui/src/settings.rs
 * 里的 `Settings`。
 *
 * 字段名和类型必须和 Rust 那边一致：那份文件由 serde 读写，多一个字段不会
 * 出错（serde 默认忽略未知字段），但类型写错（比如把数字写成字符串）会让
 * 整份配置读不出来，recovery 会当作没有配置、退回默认值。所以这里的默认值
 * 也照着 `Settings::of(&State::default())` 抄。
 *
 * 路径：/metadata/skyward/settings.json
 */
@Serializable
data class SkywardSettings(
    /** 语言页选的语言，按 locale 名；没有就跟随系统。 */
    val language: String? = null,
    /** 上次说的语言，给命名不了语言的启动用（fastboot）。 */
    val spoken: String? = null,
    val dark: Boolean = false,
    /** 亮度滑块的位置，0 到 1。 */
    val brightness: Float = 0.6f,
    /** 无操作多久灭屏，秒；0 表示不灭。 */
    val screenOff: Long = 60L,
    /** 时钟跟随已安装系统设置的时区。 */
    @SerialName("follow_system_time_zone")
    val followSystemTimeZone: Boolean = true,
    /** 时区页选的时区，按 tz 数据库的名字。 */
    @SerialName("time_zone")
    val timeZone: String = "Asia/Shanghai",
    /** 上次看到的、已安装系统的时区。 */
    @SerialName("system_time_zone")
    val systemTimeZone: String? = null,
    val vibrate: Boolean = true,
    /** 振动强度的位置，0 到 1。 */
    val vibration: Float = 0.5f,
    /** 插上电脑时把内部存储给它。 */
    @SerialName("file_transfer")
    val fileTransfer: Boolean = true,
    /**
     * 充电快慢，按名字：plain、fast、fastest。
     *
     * 这是这个 APK 最常改的一项——它对应 Skyward 快充页的三档。
     */
    @SerialName("charge_mode")
    val chargeMode: String = "fast",
    /** 电池过热时降档充电，以及多少度算过热（摄氏整度）。 */
    @SerialName("thermal_guard")
    val thermalGuard: Boolean = true,
    @SerialName("thermal_limit")
    val thermalLimit: Int = 40,
    val install: Install = Install(),
    val backup: Backup = Backup(),
    val logs: Logs = Logs(),
    /** 手动指定的 IPv4；有它就不走 DHCP。 */
    @SerialName("wlan_static")
    val wlanStatic: StaticV4? = null,
) {
    @Serializable
    data class Install(
        /** zip 的安装脚本只在签名匹配时才跑。 */
        @SerialName("check_signature")
        val checkSignature: Boolean = true,
        /** 安装写掉的 recovery 分区，Skyward 会再进一次。 */
        @SerialName("keep_recovery")
        val keepRecovery: Boolean = true,
    )

    @Serializable
    data class Backup(
        /** 备份哪些分区：boot、system、data、modem。 */
        val parts: List<String> = listOf("boot", "system", "data", "modem"),
        /** 存到哪：storage 或 usb_drive。 */
        val to: String = "storage",
        val compress: Boolean = false,
        val verify: Boolean = true,
    )

    @Serializable
    data class Logs(
        /** 哪些日志：recovery、kernel。 */
        val kinds: List<String> = listOf("recovery", "kernel"),
        /** 存到哪：temporary 或 storage。 */
        val to: String = "storage",
    )

    @Serializable
    data class StaticV4(
        val address: String = "",
        val mask: String = "255.255.255.0",
        val gateway: String = "",
        val dns: String = "",
    )

    companion object {
        /**
         * `encodeDefaults` 打开：Rust 那边的字段没有 `#[serde(default)]`，缺一
         * 个就整份读不出来，所以写回去时要把每个字段都带上。
         */
        val json = Json {
            prettyPrint = true
            encodeDefaults = true
            ignoreUnknownKeys = true
        }
    }
}
