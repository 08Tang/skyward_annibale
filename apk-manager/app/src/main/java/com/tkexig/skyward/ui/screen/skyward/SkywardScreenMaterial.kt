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
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
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
import com.tkexig.skyward.ui.component.material.SegmentedColumn
import com.tkexig.skyward.ui.component.material.SegmentedListItem
import com.tkexig.skyward.ui.component.material.SegmentedSwitchItem

/**
 * Skyward 配置页（Material 皮肤）。
 *
 * 和 Miuix 版共用 [SkywardViewModel]：改动先落在内存里，点「保存到 Skyward」
 * 才写 /metadata/skyward/settings.json。
 */
@Composable
fun SkywardScreenMaterial(
    state: SkywardUiState,
    bottomInnerPadding: Dp,
    actions: SkywardActions,
) {
    val scrollState = rememberScrollState()
    var importText by remember { mutableStateOf("") }
    var showImport by remember { mutableStateOf(false) }

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
            SegmentedColumn {
                item {
                    SegmentedListItem(
                        headlineContent = { Text(message) },
                    )
                }
            }
        }

        if (!state.hasRoot) {
            SegmentedColumn {
                item {
                    SegmentedListItem(
                        headlineContent = { Text("Skyward Manager 需要 root") },
                        supportingContent = { Text("没有 root 读不到 ${SkywardConfig.FILE}。") },
                    )
                }
            }
            Spacer(Modifier.height(8.dp))
            return@Column
        }

        val settings = state.settings ?: SkywardSettings()

        // ---- 充电 ----
        SegmentedColumn(title = "充电") {
            item {
                SegmentedListItem(
                    headlineContent = { Text("充电档位") },
                    supportingContent = {
                        Text("当前：${chargeModeLabel(settings.chargeMode)}")
                    },
                )
            }
            item {
                Row(
                    modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 4.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    listOf("plain" to "普通", "fast" to "快充", "fastest" to "最快").forEach { (key, label) ->
                        OutlinedButton(
                            onClick = { actions.onEdit { it.copy(chargeMode = key) } },
                            modifier = Modifier.weight(1f),
                        ) { Text(label) }
                    }
                }
            }
            item {
                SegmentedSwitchItem(
                    title = "温控保护",
                    summary = "温度过高时自动降速充电",
                    checked = settings.thermalGuard,
                    onCheckedChange = { on -> actions.onEdit { it.copy(thermalGuard = on) } },
                )
            }
            if (settings.thermalGuard) {
                item {
                    StepperItem(
                        title = "温控上限",
                        value = settings.thermalLimit,
                        unit = "°C",
                        range = 35..50,
                        onChange = { v -> actions.onEdit { it.copy(thermalLimit = v) } },
                    )
                }
            }
        }

        // ---- 显示 ----
        SegmentedColumn(title = "显示") {
            item {
                SegmentedSwitchItem(
                    title = "深色模式",
                    summary = "Skyward Recovery 界面用深色",
                    checked = settings.dark,
                    onCheckedChange = { on -> actions.onEdit { it.copy(dark = on) } },
                )
            }
            item {
                StepperItem(
                    title = "亮度",
                    value = (settings.brightness * 100).toInt(),
                    unit = "%",
                    range = 10..100,
                    step = 5,
                    onChange = { v -> actions.onEdit { it.copy(brightness = v / 100f) } },
                )
            }
            item {
                StepperItem(
                    title = "自动灭屏",
                    value = settings.screenOff.toInt(),
                    unit = "秒",
                    range = 0..300,
                    step = 15,
                    onChange = { v -> actions.onEdit { it.copy(screenOff = v.toLong()) } },
                )
            }
        }

        // ---- 时间 ----
        SegmentedColumn(title = "时间") {
            item {
                SegmentedSwitchItem(
                    title = "跟随系统时区",
                    summary = "关闭后用手动指定的时区",
                    checked = settings.followSystemTimeZone,
                    onCheckedChange = { on -> actions.onEdit { it.copy(followSystemTimeZone = on) } },
                )
            }
            if (!settings.followSystemTimeZone) {
                item {
                    SegmentedListItem(
                        headlineContent = { Text("时区") },
                        supportingContent = { Text(settings.timeZone.ifBlank { "—" }) },
                    )
                }
            }
        }

        // ---- 网络 ----
        SegmentedColumn(title = "静态 IP") {
            val wlan = settings.wlanStatic
            if (wlan == null) {
                item {
                    SegmentedListItem(
                        headlineContent = { Text("当前用 DHCP") },
                        supportingContent = {
                            Text("要固定地址的话，在 Skyward Recovery 里设一次，这里就会显示出来。")
                        },
                    )
                }
            } else {
                item {
                    SegmentedListItem(
                        headlineContent = { Text(wlan.address.ifBlank { "—" }) },
                        supportingContent = { Text("地址") },
                    )
                }
                item {
                    SegmentedListItem(
                        headlineContent = { Text(wlan.gateway.ifBlank { "—" }) },
                        supportingContent = { Text("网关") },
                    )
                }
            }
        }

        // ---- 备份 ----
        SegmentedColumn(title = "配置备份") {
            item {
                SegmentedListItem(
                    headlineContent = {
                        Text(if (state.exists) SkywardConfig.FILE else "还没有配置文件")
                    },
                    supportingContent = {
                        Text(if (state.exists) "配置文件在这个路径" else "保存一次就会创建")
                    },
                )
            }
            item {
                Row(
                    modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 4.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    OutlinedButton(onClick = actions.onRefresh, modifier = Modifier.weight(1f)) {
                        Text("刷新")
                    }
                    OutlinedButton(
                        onClick = { showImport = !showImport },
                        modifier = Modifier.weight(1f),
                    ) { Text(if (showImport) "收起导入" else "导入 JSON") }
                }
            }
            if (showImport) {
                item {
                    Column(modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp)) {
                        OutlinedTextField(
                            value = importText,
                            onValueChange = { importText = it },
                            label = { Text("把导出的 JSON 贴在这里") },
                            modifier = Modifier.fillMaxWidth(),
                        )
                        Spacer(Modifier.height(8.dp))
                        Button(onClick = { actions.onStageImport(importText) }) {
                            Text("读入")
                        }
                    }
                }
            }
        }

        state.message?.let { msg ->
            Text(
                text = msg,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.primary,
            )
        }

        Spacer(Modifier.height(8.dp))
        Button(
            onClick = actions.onSave,
            modifier = Modifier.fillMaxWidth(),
        ) { Text("保存到 Skyward") }
        Spacer(Modifier.height(bottomInnerPadding + 24.dp))
    }
}

private fun chargeModeLabel(mode: String): String = when (mode) {
    "plain" -> "普通"
    "fast" -> "快充"
    "fastest" -> "最快"
    else -> mode
}

@Composable
private fun StepperItem(
    title: String,
    value: Int,
    unit: String,
    range: IntRange,
    step: Int = 1,
    onChange: (Int) -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column {
            Text(text = title, style = MaterialTheme.typography.bodyLarge)
            Text(
                text = "$value$unit",
                style = MaterialTheme.typography.bodyMedium,
                fontWeight = FontWeight.Medium,
            )
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            OutlinedButton(
                onClick = { onChange((value - step).coerceIn(range.first, range.last)) },
            ) { Text("−") }
            OutlinedButton(
                onClick = { onChange((value + step).coerceIn(range.first, range.last)) },
            ) { Text("+") }
        }
    }
}
