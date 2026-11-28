package com.tkexig.skyward.ui.screen.skyward

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.add
import androidx.compose.foundation.layout.displayCutout
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Schedule
import androidx.compose.material.icons.rounded.Brightness4
import androidx.compose.material.icons.rounded.DarkMode
import androidx.compose.material.icons.rounded.FlashOn
import androidx.compose.material.icons.rounded.Language
import androidx.compose.material.icons.rounded.CheckCircle
import androidx.compose.material.icons.rounded.Save
import androidx.compose.material.icons.rounded.BatteryFull
import androidx.compose.material.icons.rounded.Wifi
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.tkexig.skyward.data.skyward.SkywardConfig
import com.tkexig.skyward.data.skyward.SkywardSettings
import com.tkexig.skyward.ui.component.miuix.AppCard
import com.tkexig.skyward.ui.component.miuix.AppScaffold
import top.yukonga.miuix.kmp.basic.CircularProgressIndicator
import top.yukonga.miuix.kmp.basic.Icon
import top.yukonga.miuix.kmp.basic.Text
import top.yukonga.miuix.kmp.basic.TextButton
import top.yukonga.miuix.kmp.basic.TextField
import top.yukonga.miuix.kmp.preference.OverlayDropdownPreference
import top.yukonga.miuix.kmp.preference.SwitchPreference
import top.yukonga.miuix.kmp.theme.MiuixTheme

/**
 * Skyward 配置页（Miuix 皮肤）。
 *
 * 每一项都只改内存里的 [SkywardSettings]，点「保存到 Skyward」才写回
 * /metadata/skyward/settings.json——拖滑块不该一路往分区里写。
 *
 * 用的都是 Miuix 自己的 preference 组件，和设置页长得一样：这一页在
 * 底栏里和「设置」并排，样式对不上会很扎眼。
 */
@Composable
fun SkywardScreenMiuix(
    state: SkywardUiState,
    bottomInnerPadding: Dp,
    actions: SkywardActions,
) {
    val colorScheme = MiuixTheme.colorScheme
    val scrollState = rememberScrollState()
    var importText by remember { mutableStateOf("") }
    var showImport by remember { mutableStateOf(false) }

    // 这一页没有 topBar，状态栏的区域就得自己让开：不给 windowInsets 的话
    // 第一张卡会钻到通知栏底下。
    AppScaffold(
        contentWindowInsets = WindowInsets.systemBars
            .add(WindowInsets.displayCutout)
            .only(WindowInsetsSides.Top + WindowInsetsSides.Horizontal),
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .verticalScroll(scrollState)
                .padding(horizontal = 16.dp)
                .padding(top = innerPadding.calculateTopPadding()),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Spacer(Modifier.height(8.dp))

            if (state.loading) {
                Row(
                    modifier = Modifier.fillMaxWidth().padding(vertical = 40.dp),
                    horizontalArrangement = Arrangement.Center,
                ) { CircularProgressIndicator() }
                return@Column
            }

            if (!state.hasRoot) {
                AppCard {
                    Text(
                        text = "读取 Skyward 配置需要 root。",
                        style = MiuixTheme.textStyles.body1,
                    )
                    Spacer(Modifier.height(4.dp))
                    Text(
                        text = "配置在 ${SkywardConfig.FILE}，只有 root 能挂上 /metadata。",
                        style = MiuixTheme.textStyles.body2,
                        color = colorScheme.onSurfaceVariantSummary,
                    )
                }
                Spacer(Modifier.height(bottomInnerPadding + 24.dp))
                return@Column
            }

            state.error?.let { message ->
                AppCard {
                    Text(
                        text = message,
                        style = MiuixTheme.textStyles.body1,
                        color = colorScheme.error,
                    )
                }
            }

            val settings = state.settings ?: SkywardSettings()

            // ---- 充电 ----
            AppCard {
                ChargeModePreference(
                    current = settings.chargeMode,
                    onSelect = { mode -> actions.onEdit { it.copy(chargeMode = mode) } },
                )
                SwitchPreference(
                    title = "温控保护",
                    summary = "电池过热时降档充电",
                    startAction = { PreferenceIcon(Icons.Rounded.BatteryFull) },
                    checked = settings.thermalGuard,
                    onCheckedChange = { on -> actions.onEdit { it.copy(thermalGuard = on) } },
                )
                if (settings.thermalGuard) {
                    // 40 到 55 是 Skyward 自己的上下限（THERMAL_LIMIT_LEAST/MOST），
                    // 超出这个范围 recovery 会当作没设过。
                    StepperPreference(
                        title = "温控上限",
                        value = settings.thermalLimit,
                        unit = "°C",
                        range = 40..55,
                        onChange = { v -> actions.onEdit { it.copy(thermalLimit = v) } },
                    )
                }
            }

            // ---- 显示 ----
            AppCard {
                SwitchPreference(
                    title = "深色模式",
                    summary = "Skyward Recovery 界面用深色",
                    startAction = { PreferenceIcon(Icons.Rounded.DarkMode) },
                    checked = settings.dark,
                    onCheckedChange = { on -> actions.onEdit { it.copy(dark = on) } },
                )
                StepperPreference(
                    title = "亮度",
                    value = (settings.brightness * 100).toInt(),
                    unit = "%",
                    range = 10..100,
                    step = 5,
                    onChange = { v -> actions.onEdit { it.copy(brightness = v / 100f) } },
                )
                ScreenOffPreference(
                    seconds = settings.screenOff,
                    onChange = { v -> actions.onEdit { it.copy(screenOff = v) } },
                )
            }

            // ---- 时间 ----
            AppCard {
                SwitchPreference(
                    title = "跟随系统时区",
                    summary = "关闭后用手动指定的时区",
                    startAction = { PreferenceIcon(Icons.Rounded.Schedule) },
                    checked = settings.followSystemTimeZone,
                    onCheckedChange = { on -> actions.onEdit { it.copy(followSystemTimeZone = on) } },
                )
                if (!settings.followSystemTimeZone) {
                    ArrowValuePreference(
                        title = "时区",
                        value = settings.timeZone,
                        icon = Icons.Rounded.Language,
                    )
                }
            }

            // ---- 网络 ----
            AppCard {
                val wlan = settings.wlanStatic
                if (wlan == null) {
                    ArrowValuePreference(
                        title = "静态 IP",
                        value = "使用 DHCP",
                        icon = Icons.Rounded.Wifi,
                    )
                } else {
                    ArrowValuePreference("静态 IP", wlan.address, Icons.Rounded.Wifi)
                    ArrowValuePreference("掩码", wlan.mask, null)
                    ArrowValuePreference("网关", wlan.gateway, null)
                    ArrowValuePreference("DNS", wlan.dns, null)
                }
            }

            // ---- 备份 ----
            AppCard {
                ArrowValuePreference(
                    title = "配置文件",
                    value = if (state.exists) "已存在" else "还没创建",
                    icon = Icons.Rounded.Save,
                )
                Row(
                    modifier = Modifier.fillMaxWidth().padding(top = 8.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    TextButton(
                        text = "重新读取",
                        onClick = actions.onRefresh,
                        modifier = Modifier.weight(1f),
                    )
                    TextButton(
                        text = if (showImport) "收起导入" else "导入 JSON",
                        onClick = { showImport = !showImport },
                        modifier = Modifier.weight(1f),
                    )
                }
                if (showImport) {
                    Spacer(Modifier.height(8.dp))
                    TextField(
                        value = importText,
                        onValueChange = { importText = it },
                        modifier = Modifier.fillMaxWidth(),
                        label = "把导出的 JSON 贴在这里",
                    )
                    Spacer(Modifier.height(8.dp))
                    TextButton(
                        text = "读入",
                        onClick = { actions.onStageImport(importText) },
                        modifier = Modifier.fillMaxWidth(),
                    )
                }
            }

            state.message?.let { msg ->
                AppCard {
                    Row(
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalAlignment = androidx.compose.ui.Alignment.CenterVertically,
                    ) {
                        Icon(
                            imageVector = Icons.Rounded.CheckCircle,
                            contentDescription = null,
                            modifier = Modifier.size(18.dp),
                            tint = colorScheme.primary,
                        )
                        Text(
                            text = msg,
                            style = MiuixTheme.textStyles.body2,
                            color = colorScheme.primary,
                        )
                    }
                }
            }

            Spacer(Modifier.height(8.dp))
            TextButton(
                text = "保存到 Skyward",
                onClick = actions.onSave,
                modifier = Modifier.fillMaxWidth(),
            )
            Spacer(Modifier.height(bottomInnerPadding + 24.dp))
        }
    }
}

@Composable
private fun PreferenceIcon(icon: ImageVector) {
    Icon(
        imageVector = icon,
        contentDescription = null,
        modifier = Modifier.padding(end = 6.dp).size(22.dp),
        tint = MiuixTheme.colorScheme.onBackground,
    )
}

/**
 * 充电三档。
 *
 * 用下拉而不是并排三个按钮：档位名字长短不一，并排会挤成三行字；下拉和
 * 设置页里其它多选项长得一样。
 */
@Composable
private fun ChargeModePreference(current: String, onSelect: (String) -> Unit) {
    val modes = listOf("plain" to "普通", "fast" to "快充", "fastest" to "最快")
    OverlayDropdownPreference(
        title = "充电档位",
        summary = when (current) {
            "plain" -> "由内核按电池状态限流"
            "fast" -> "跳过电池校验，按普通快充供电"
            "fastest" -> "校验充电器，按充电器能给的最大功率"
            else -> current
        },
        startAction = { PreferenceIcon(Icons.Rounded.FlashOn) },
        items = modes.map { it.second },
        selectedIndex = modes.indexOfFirst { it.first == current }.coerceAtLeast(0),
        onSelectedIndexChange = { index -> onSelect(modes[index].first) },
    )
}

/**
 * 灭屏时间。
 *
 * 不是连续的秒数：Skyward 那边是七个固定档（15 秒到 10 分钟，加一个「不
 * 灭屏」），存的是秒。这里按档给，滑块停不到 recovery 不认的值上。
 */
@Composable
private fun ScreenOffPreference(seconds: Long, onChange: (Long) -> Unit) {
    val steps = listOf(15L to "15 秒", 30L to "30 秒", 60L to "1 分钟",
        120L to "2 分钟", 300L to "5 分钟", 600L to "10 分钟", 0L to "不灭屏")
    val index = steps.indexOfFirst { it.first == seconds }.let { if (it < 0) 2 else it }
    OverlayDropdownPreference(
        title = "自动灭屏",
        summary = steps[index].second,
        startAction = { PreferenceIcon(Icons.Rounded.Brightness4) },
        items = steps.map { it.second },
        selectedIndex = index,
        onSelectedIndexChange = { onChange(steps[it].first) },
    )
}

/** 一行只读的值，右边跟箭头的样子和设置页一致。 */
@Composable
private fun ArrowValuePreference(
    title: String,
    value: String,
    icon: ImageVector?,
) {
    top.yukonga.miuix.kmp.preference.ArrowPreference(
        title = title,
        summary = value.ifBlank { "—" },
        startAction = icon?.let { { PreferenceIcon(it) } },
        onClick = {},
    )
}

/**
 * 加减号调一个整数。
 *
 * Miuix 没有现成的数值组件，这里自己画一行：左边标题，右边 − 值 +。
 */
@Composable
private fun StepperPreference(
    title: String,
    value: Int,
    unit: String,
    range: IntRange,
    step: Int = 1,
    onChange: (Int) -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp, horizontal = 16.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = androidx.compose.ui.Alignment.CenterVertically,
    ) {
        Text(text = title, style = MiuixTheme.textStyles.body1)
        Row(
            verticalAlignment = androidx.compose.ui.Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            TextButton(
                text = "−",
                onClick = { onChange((value - step).coerceIn(range.first, range.last)) },
                enabled = value > range.first,
            )
            Text(
                text = "$value$unit",
                style = MiuixTheme.textStyles.body1,
                modifier = Modifier.padding(horizontal = 4.dp),
            )
            TextButton(
                text = "+",
                onClick = { onChange((value + step).coerceIn(range.first, range.last)) },
                enabled = value < range.last,
            )
        }
    }
}
