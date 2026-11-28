package com.tkexig.skyward.data.skyward

import android.os.Build
import com.tkexig.skyward.data.log.AppLog
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

/**
 * 一台机器上的 Skyward 认哪些配置。
 *
 * 每台机器一份，写在 assets/devices/ 下的 JSON 里。加一台新机器就是加一个
 * 文件，不用动代码——这正是当初把它单独拆出来的原因。
 *
 * 作者和机型绑在一起：不同人做的适配，认的配置项可能不一样，出问题时得
 * 知道该找谁。
 */
@Serializable
data class SkywardDevice(
    /** 机型全名，显示在页面上。 */
    val name: String,
    /** 代号，和 `ro.product.device` 对得上。 */
    val codename: String,
    /**
     * 这台机器还会报的别的代号。
     *
     * 有的机器 `ro.product.device` 和适配时用的代号不一样（改过、或者厂商
     * 换过写法），列在这里就都能认出来。
     */
    val aliases: List<String> = emptyList(),
    /** 做这份适配的人。 */
    val author: String,
    /** 可以配哪些项，见 [Feature]。 */
    val features: List<String> = emptyList(),
    /** 这份适配针对的 Skyward 版本，给人看的。 */
    val skywardVersion: String = "",
    /** 备注，比如需要注意什么。 */
    val note: String = "",
) {
    /** 这台机器支持某一项吗。 */
    fun supports(feature: Feature): Boolean = feature.key in features

    /** 这个代号是这台机器吗。 */
    fun answersTo(codename: String): Boolean =
        this.codename.equals(codename, ignoreCase = true) ||
            aliases.any { it.equals(codename, ignoreCase = true) }
}

/**
 * 配置页上能出现的一项。
 *
 * 用字符串存在 JSON 里，而不是枚举序号：以后删一项、加一项都不会把别的
 * 项错位。
 */
enum class Feature(val key: String, val label: String) {
    ChargeMode("charge_mode", "快充档位"),
    ThermalGuard("thermal_guard", "温控保护"),
    Brightness("brightness", "亮度"),
    ScreenOff("screen_off", "自动灭屏"),
    Vibration("vibration", "振动"),
    FileTransfer("file_transfer", "文件传输"),
    TimeZone("time_zone", "时区"),
    WlanStatic("wlan_static", "静态 IP"),
    Install("install", "安装选项"),
    Backup("backup", "备份"),
    Logs("logs", "日志"),
    ;

    companion object {
        fun of(key: String): Feature? = entries.firstOrNull { it.key == key }
    }
}

/**
 * 这台机器是哪一个。
 *
 * 按代号匹配：`ro.product.device` 比 `Build.MODEL` 可靠，机型名各家会改，
 * 代号不会。认不出来就返回 null，页面上说明没适配，而不是猜一个。
 */
object SkywardDevices {

    private const val TAG = "SkywardDevices"
    private const val ASSET_DIR = "devices"
    private const val PREFS = "skyward_device"
    private const val KEY_SELECTED = "selected_codename"

    private val json = Json {
        ignoreUnknownKeys = true
        isLenient = true
    }

    /** 全部已知机型，按名字排。 */
    fun all(context: android.content.Context): List<SkywardDevice> {
        val names = runCatching {
            context.assets.list(ASSET_DIR)?.filter { it.endsWith(".json") }.orEmpty()
        }.getOrDefault(emptyList())
        return names.mapNotNull { fileName ->
            runCatching {
                val text = context.assets.open("$ASSET_DIR/$fileName").bufferedReader().use { it.readText() }
                json.decodeFromString<SkywardDevice>(text)
            }.onFailure {
                // 一份坏文件不该让整个页面打不开，记下来跳过。
                AppLog.w("device profile $fileName unreadable", it)
            }.getOrNull()
        }.sortedBy { it.name }
    }

    /**
     * 当前这台机器。
     *
     * 只看代号（含别名），不看机型名：机型名是给人看的，各家会改，代号才是
     * 机器的身份。按名字匹配还容易把「REDMI K90」这种名字相同的别的机器认成
     * 这一台。
     */
    fun current(context: android.content.Context): SkywardDevice? {
        val codename = deviceCodename()
        if (codename.isBlank()) return null
        return all(context).firstOrNull { it.answersTo(codename) }
    }

    /**
     * 用户手动指定的机型代号。
     *
     * 空串表示「自动」——按 `ro.product.device` 认。存的是代号而不是整份
     * 机型信息：机型文件会更新（作者换了、支持项变了），存代号才能跟着用上
     * 新的那份。
     */
    fun savedCodename(context: android.content.Context): String =
        prefs(context).getString(KEY_SELECTED, "").orEmpty()

    /** 记下用户选的机型；传空串回到自动。 */
    fun saveCodename(context: android.content.Context, codename: String) {
        prefs(context).edit().putString(KEY_SELECTED, codename).apply()
    }

    private fun prefs(context: android.content.Context) =
        context.getSharedPreferences(PREFS, android.content.Context.MODE_PRIVATE)

    /**
     * `ro.product.device`。
     *
     * 这个属性不用 root 就能读：它是只读的 system property，`Build.DEVICE`
     * 就是它的镜像。
     */
    fun deviceCodename(): String = Build.DEVICE.orEmpty()
}
