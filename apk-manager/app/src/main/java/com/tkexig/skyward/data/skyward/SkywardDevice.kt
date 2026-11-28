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
     * 先按代号找；代号对不上再看机型名。两个都试是为了那些 `ro.product.device`
     * 被改成通用值的机型。
     */
    fun current(context: android.content.Context): SkywardDevice? {
        val devices = all(context)
        val codename = deviceCodename()
        devices.firstOrNull { it.codename.equals(codename, ignoreCase = true) }?.let { return it }
        val model = Build.MODEL.orEmpty()
        return devices.firstOrNull { it.name.equals(model, ignoreCase = true) }
    }

    /**
     * `ro.product.device`。
     *
     * 这个属性不用 root 就能读：它是只读的 system property，`Build.DEVICE`
     * 就是它的镜像。
     */
    fun deviceCodename(): String = Build.DEVICE.orEmpty()
}
