package com.tkexig.skyward.ui.screen.install

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
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
import com.tkexig.skyward.data.skyward.SkywardRelease
import com.tkexig.skyward.ui.component.material.SegmentedColumn
import com.tkexig.skyward.ui.component.material.SegmentedColumnScope
import com.tkexig.skyward.ui.component.material.SegmentedListItem
import com.tkexig.skyward.ui.component.material.SegmentedSwitchItem

/**
 * 刷入 Skyward。
 *
 * 一屏讲清楚三件事：这一版是什么（版本、大小、时间、更新内容）、刷到哪
 * （槽位）、以及会不会先把现在这份备份下来。刷之前把这三件都摆出来，是
 * 因为写分区不可逆。
 */
@Composable
fun InstallScreen(
    state: InstallUiState,
    bottomInnerPadding: Dp,
    onBack: () -> Unit,
    actions: InstallActions,
) {
    val scroll = rememberScrollState()

    // 这一页从首页 push 进来，没有 topBar，所以状态栏和导航栏的区域都要自己
    // 让开：不然标题会钻到通知栏底下，按钮会压在手势条上。
    Box(
        modifier = Modifier
            .fillMaxSize()
            // 自定义背景图时整页是透明的，会直接看到桌面。给一个实心底色。
            .background(MaterialTheme.colorScheme.background),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .verticalScroll(scroll)
                .statusBarsPadding()
                .navigationBarsPadding()
                .padding(horizontal = 16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Spacer(Modifier.height(8.dp))

            Text(
                text = "刷入 Skyward",
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.SemiBold,
            )

            // ---- 这一版是什么 ----
            SegmentedColumn(title = "最新版本") {
                if (state.loadingRelease) {
                    item {
                        SegmentedListItem(
                            headlineContent = { Text("正在取发布信息…") },
                            leadingContent = { CircularProgressIndicator(Modifier.size(20.dp)) },
                        )
                    }
                } else if (state.release == null) {
                    item {
                        SegmentedListItem(
                            headlineContent = { Text("取不到发布信息") },
                            supportingContent = { Text(state.releaseError ?: "检查网络后重试") },
                            onClick = actions.onRefresh,
                        )
                    }
                } else {
                    ReleaseRows(state.release)
                }
            }

            // ---- 刷到哪 ----
            SegmentedColumn(title = "刷入位置") {
                item {
                    SegmentedListItem(
                        headlineContent = { Text("分区") },
                        supportingContent = { Text("recovery") },
                    )
                }
                item {
                    SlotPicker(
                        slots = state.slots,
                        selected = state.selectedSlot,
                        defaultSlot = state.slotSuffix,
                        onSelect = actions.onSelectSlot,
                    )
                }
            }

            // ---- 备份与镜像 ----
            SegmentedColumn(title = "刷前准备") {
                item {
                    SegmentedSwitchItem(
                        icon = null,
                        title = "先备份当前 recovery",
                        summary = "写分区不可逆，留一份原来的才能刷回去",
                        checked = state.backupFirst,
                        onCheckedChange = actions.onSetBackupFirst,
                    )
                }
                item {
                    SegmentedListItem(
                        headlineContent = { Text("选择已有镜像") },
                        supportingContent = { Text(state.pickedName.ifBlank { "不选就用上面的发布包" }) },
                        onClick = actions.onPickImage,
                        trailingContent = pickTrailing(state.pickedName),
                    )
                }
            }

            // ---- 进展 ----
            when (state.phase) {
                InstallPhase.Idle -> Unit
                else -> {
                    SegmentedColumn(title = "进展") {
                        item {
                            Column(Modifier.padding(16.dp)) {
                                Row(
                                    modifier = Modifier.fillMaxWidth(),
                                    horizontalArrangement = Arrangement.SpaceBetween,
                                    verticalAlignment = Alignment.CenterVertically,
                                ) {
                                    Text(state.status.ifBlank { "—" })
                                    if (state.phase == InstallPhase.Downloading) {
                                        Text("${state.progress}%")
                                    }
                                }
                                if (state.phase == InstallPhase.Downloading) {
                                    Spacer(Modifier.height(8.dp))
                                    LinearProgressIndicator(
                                        progress = { state.progress / 100f },
                                        modifier = Modifier.fillMaxWidth(),
                                    )
                                }
                                if (state.phase == InstallPhase.BackingUp ||
                                    state.phase == InstallPhase.Extracting ||
                                    state.phase == InstallPhase.Flashing
                                ) {
                                    Spacer(Modifier.height(8.dp))
                                    LinearProgressIndicator(Modifier.fillMaxWidth())
                                }
                                state.backupFile?.let { file ->
                                    Spacer(Modifier.height(8.dp))
                                    Text(
                                        text = "备份在 ${file.absolutePath}",
                                        style = MaterialTheme.typography.bodySmall,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                                    )
                                }
                            }
                        }
                    }
                }
            }

            state.error?.let { message ->
                SegmentedColumn(title = "出错") {
                    item {
                        SegmentedListItem(
                            headlineContent = { Text(message) },
                            supportingContent = { Text("没有写入任何东西，分区还是原来的") },
                        )
                    }
                }
            }

            // ---- 按钮 ----
            Spacer(Modifier.height(4.dp))
            val busy = state.phase == InstallPhase.Downloading ||
                state.phase == InstallPhase.Extracting ||
                state.phase == InstallPhase.BackingUp ||
                state.phase == InstallPhase.Flashing
            Button(
                onClick = actions.onFlash,
                enabled = !busy && state.hasRoot,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Text(if (state.phase == InstallPhase.Done) "再刷一次" else "刷入")
            }
            if (state.phase == InstallPhase.Done) {
                OutlinedButton(
                    onClick = actions.onRebootRecovery,
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Text("重启到 Recovery")
                }
            }
            OutlinedButton(
                onClick = onBack,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Text("返回")
            }
            Spacer(Modifier.height(bottomInnerPadding + 24.dp))
        }
    }
}

/**
 * 槽位选择。
 *
 * 默认是当前启动的那个槽；A/B 机器上还能选另一个，刷进去之后从那边启动。
 * 非 A/B 机器只有一个分区，就不给选了。
 */
@Composable
private fun SlotPicker(
    slots: List<String>,
    selected: String,
    defaultSlot: String,
    onSelect: (String) -> Unit,
) {
    var expanded by remember { mutableStateOf(false) }
    val label = if (selected.isBlank()) "无（非 A/B）" else selected
    val isDefault = selected == defaultSlot

    Box {
        SegmentedListItem(
            headlineContent = { Text("槽位") },
            supportingContent = {
                Text(
                    when {
                        slots.isEmpty() -> "这台机器不是 A/B，只有一个分区"
                        isDefault -> "当前启动槽"
                        else -> "刷到另一槽，重启后从那边启动"
                    }
                )
            },
            trailingContent = {
                Text(
                    text = if (slots.isEmpty()) "—" else "$label ▾",
                    color = MaterialTheme.colorScheme.primary,
                )
            },
            onClick = { if (slots.isNotEmpty()) expanded = true },
        )
        DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            slots.forEach { slot ->
                DropdownMenuItem(
                    text = { Text(if (slot == defaultSlot) "$slot（当前启动）" else slot) },
                    onClick = {
                        onSelect(slot)
                        expanded = false
                    },
                )
            }
        }
    }
}

/** 版本号、大小、发布时间、更新内容。 */
private fun SegmentedColumnScope.ReleaseRows(release: SkywardRelease) {
    item {
        SegmentedListItem(
            headlineContent = { Text(release.tag) },
            supportingContent = {
                Text(
                    listOfNotNull(formatSize(release.sizeBytes), formatDate(release.publishedAt))
                        .joinToString(" · ")
                )
            },
        )
    }
    if (release.changelog.isNotBlank()) {
        item {
            SegmentedListItem(
                headlineContent = { Text("更新内容") },
                supportingContent = {
                    Text(release.changelog.trim(), style = MaterialTheme.typography.bodySmall)
                },
            )
        }
    }
}

@Composable
private fun pickTrailing(name: String): (@Composable () -> Unit)? {
    if (name.isBlank()) return null
    return { Text("已选", color = MaterialTheme.colorScheme.primary) }
}

private fun formatSize(bytes: Long): String? = when {
    bytes <= 0 -> null
    bytes >= 1024L * 1024 -> "%.1f MB".format(bytes / 1024.0 / 1024.0)
    bytes >= 1024 -> "%.0f KB".format(bytes / 1024.0)
    else -> "$bytes B"
}

private fun formatDate(iso: String): String? {
    if (iso.isBlank()) return null
    return runCatching {
        val formatter = java.time.format.DateTimeFormatter
            .ofPattern("yyyy-MM-dd HH:mm")
            .withZone(java.time.ZoneId.systemDefault())
        formatter.format(java.time.Instant.parse(iso))
    }.getOrNull()
}

data class InstallActions(
    val onRefresh: () -> Unit,
    val onPickImage: () -> Unit,
    val onSetBackupFirst: (Boolean) -> Unit,
    val onSelectSlot: (String) -> Unit,
    val onFlash: () -> Unit,
    val onRebootRecovery: () -> Unit,
)
