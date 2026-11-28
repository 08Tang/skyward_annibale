package com.tkexig.skyward.ui.screen.skyward

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.tkexig.skyward.data.skyward.SkywardConfig
import com.tkexig.skyward.data.skyward.SkywardSettings
import com.tkexig.skyward.ui.component.miuix.AppCard
import com.tkexig.skyward.ui.component.miuix.AppScaffold
import top.yukonga.miuix.kmp.basic.CircularProgressIndicator
import top.yukonga.miuix.kmp.basic.Text
import top.yukonga.miuix.kmp.basic.TextButton
import top.yukonga.miuix.kmp.basic.TextField
import top.yukonga.miuix.kmp.preference.SwitchPreference
import top.yukonga.miuix.kmp.theme.MiuixTheme

/**
 * Skyward 配置页（Miuix 皮肤）。
 *
 * 每个开关和加减号都只改内存里的 [SkywardSettings]，点「保存到 Skyward」才
 * 写回 /metadata/skyward/settings.json——调数值时不该一路往分区里写。
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

    AppScaffold {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .verticalScroll(scrollState)
                .padding(horizontal = 16.dp),
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

            state.error?.let { message ->
                AppCard {
                    Text(
                        text = message,
                        style = MiuixTheme.textStyles.body2,
                        color = colorScheme.onSurfaceVariantSummary,
                    )
                }
            }

            if (!state.hasRoot) {
                AppCard {
                    Text(
                        text = "Skyward Manager 需要 root 才能读写 ${SkywardConfig.FILE}。",
                        style = MiuixTheme.textStyles.body2,
                        color = colorScheme.onSurfaceVariantSummary,
                    )
                }
                Spacer(Modifier.height(8.dp))
                return@Column
            }

            val settings = state.settings ?: SkywardSettings()

            // ---- 充电 ----
            AppCard {
                SectionTitle("充电")
                ChargeModeRow(
                    current = settings.chargeMode,
                    onSelect = { mode -> actions.onEdit { it.copy(chargeMode = mode) } },
                )
                SwitchPreference(
                    title = "温控保护",
                    summary = "温度过高时自动降速充电",
                    checked = settings.thermalGuard,
                    onCheckedChange = { on -> actions.onEdit { it.copy(thermalGuard = on) } },
                )
                if (settings.thermalGuard) {
                    StepperRow(
                        title = "温控上限",
                        value = settings.thermalLimit,
                        unit = "°C",
                        range = 35..50,
                        onChange = { v -> actions.onEdit { it.copy(thermalLimit = v) } },
                    )
                }
            }

            // ---- 显示 ----
            AppCard {
                SectionTitle("显示")
                SwitchPreference(
                    title = "深色模式",
                    summary = "Skyward Recovery 界面用深色",
                    checked = settings.dark,
                    onCheckedChange = { on -> actions.onEdit { it.copy(dark = on) } },
                )
                StepperRow(
                    title = "亮度",
                    value = (settings.brightness * 100).toInt(),
                    unit = "%",
                    range = 10..100,
                    step = 5,
                    onChange = { v -> actions.onEdit { it.copy(brightness = v / 100f) } },
                )
                StepperRow(
                    title = "自动灭屏",
                    value = settings.screenOff.toInt(),
                    unit = "秒",
                    range = 0..300,
                    step = 15,
                    onChange = { v -> actions.onEdit { it.copy(screenOff = v.toLong()) } },
                )
            }

            // ---- 时间 ----
            AppCard {
                SectionTitle("时间")
                SwitchPreference(
                    title = "跟随系统时区",
                    summary = "关闭后用手动指定的时区",
                    checked = settings.followSystemTimeZone,
                    onCheckedChange = { on -> actions.onEdit { it.copy(followSystemTimeZone = on) } },
                )
                if (!settings.followSystemTimeZone) {
                    ValueRow("时区", settings.timeZone)
                }
            }

            // ---- 网络 ----
            AppCard {
                SectionTitle("静态 IP")
                val wlan = settings.wlanStatic
                if (wlan == null) {
                    Text(
                        text = "当前用 DHCP。要固定地址的话，在 Skyward Recovery 里设一次，这里就会显示出来。",
                        style = MiuixTheme.textStyles.body2,
                        color = colorScheme.onSurfaceVariantSummary,
                    )
                } else {
                    ValueRow("地址", wlan.address)
                    ValueRow("掩码", wlan.mask)
                    ValueRow("网关", wlan.gateway)
                    ValueRow("DNS", wlan.dns)
                }
            }

            // ---- 备份 ----
            AppCard {
                SectionTitle("配置备份")
                Text(
                    text = if (state.exists) {
                        "配置文件在 ${SkywardConfig.FILE}"
                    } else {
                        "还没有配置文件。保存一次就会创建。"
                    },
                    style = MiuixTheme.textStyles.body2,
                    color = colorScheme.onSurfaceVariantSummary,
                )
                Spacer(Modifier.height(8.dp))
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    TextButton(text = "刷新", onClick = actions.onRefresh)
                    TextButton(
                        text = if (showImport) "收起导入" else "导入 JSON",
                        onClick = { showImport = !showImport },
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
                    TextButton(text = "读入", onClick = { actions.onStageImport(importText) })
                }
            }

            state.message?.let { msg ->
                AppCard {
                    Text(
                        text = msg,
                        style = MiuixTheme.textStyles.body2,
                        color = colorScheme.primary,
                    )
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
private fun SectionTitle(text: String) {
    Text(
        text = text,
        style = MiuixTheme.textStyles.title4,
        fontWeight = FontWeight.SemiBold,
        modifier = Modifier.padding(bottom = 8.dp),
    )
}

@Composable
private fun ValueRow(label: String, value: String) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(text = label, style = MiuixTheme.textStyles.body2)
        Text(
            text = value.ifBlank { "—" },
            style = MiuixTheme.textStyles.body2,
            color = MiuixTheme.colorScheme.onSurfaceVariantSummary,
        )
    }
}

@Composable
private fun StepperRow(
    title: String,
    value: Int,
    unit: String,
    range: IntRange,
    step: Int = 1,
    onChange: (Int) -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(vertical = 6.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(text = title, style = MiuixTheme.textStyles.body1)
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            TextButton(
                text = "−",
                onClick = { onChange((value - step).coerceIn(range.first, range.last)) },
            )
            Text(text = "$value$unit", style = MiuixTheme.textStyles.body1)
            TextButton(
                text = "+",
                onClick = { onChange((value + step).coerceIn(range.first, range.last)) },
            )
        }
    }
}

@Composable
private fun ChargeModeRow(current: String, onSelect: (String) -> Unit) {
    val modes = listOf("plain" to "普通", "fast" to "快充", "fastest" to "最快")
    Column(modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp)) {
        Text(text = "充电档位", style = MiuixTheme.textStyles.body1)
        Spacer(Modifier.height(6.dp))
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            modes.forEach { (key, label) ->
                TextButton(
                    text = label,
                    onClick = { onSelect(key) },
                    modifier = Modifier.weight(1f),
                )
            }
        }
        Text(
            text = "当前：${modes.firstOrNull { it.first == current }?.second ?: current}",
            style = MiuixTheme.textStyles.body2,
            color = MiuixTheme.colorScheme.onSurfaceVariantSummary,
        )
    }
}
