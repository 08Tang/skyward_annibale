package com.tkexig.skyward.ui.screen.install

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
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

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .verticalScroll(scroll)
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
                SegmentedListItem(
                    headlineContent = { Text("槽位") },
                    // 非 A/B 机器没有槽位，说清楚，而不是显示空白。
                    supportingContent = {
                        Text(
                            if (state.slotSuffix.isBlank()) {
                                "这台机器不是 A/B，只有一个分区"
                            } else {
                                "当前启动槽 ${state.slotSuffix}"
                            }
                        )
                    },
                )
            }
            if (!state.hasRecoveryPartition && state.hasRoot) {
                item {
                    SegmentedListItem(
                        headlineContent = { Text("没有独立的 recovery 分区") },
                        supportingContent = {
                            Text("A/B 机型把 recovery 合在 boot 里，这个页面刷不了")
                        },
                    )
                }
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
                    supportingContent = {
                        Text(
                            state.pickedName.ifBlank {
                                "不选就用上面的发布包"
                            }
                        )
                    },
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
            enabled = !busy && state.hasRoot && state.hasRecoveryPartition,
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

/** 版本号、大小、发布时间、更新内容。 */
@Composable
private fun SegmentedColumnScope.ReleaseRows(release: SkywardRelease) {
    item {
        SegmentedListItem(
            headlineContent = { Text(release.tag) },
            supportingContent = {
                Text(
                    listOfNotNull(
                        formatSize(release.sizeBytes),
                        formatDate(release.publishedAt),
                    ).joinToString(" · ")
                )
            },
        )
    }
    if (release.body.isNotBlank()) {
        item {
            SegmentedListItem(
                headlineContent = { Text("更新内容") },
                supportingContent = {
                    Text(
                        text = release.body.trim(),
                        style = MaterialTheme.typography.bodySmall,
                    )
                },
            )
        }
    }
}

/**
 * 选了镜像时右边显示「已选」。
 *
 * 写成函数而不是在调用处用 if 表达式：`@Composable (() -> Unit)?` 这种可空
 * 的函数类型在条件表达式里容易被推断成别的类型，编译器会报错。
 */
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

/** GitHub 给的是 ISO 8601，按本地时区显示成 2026-11-24 20:45。 */
private fun formatDate(iso: String): String? {
    if (iso.isBlank()) return null
    return runCatching {
        val parser = java.time.Instant.parse(iso)
        val formatter = java.time.format.DateTimeFormatter
            .ofPattern("yyyy-MM-dd HH:mm")
            .withZone(java.time.ZoneId.systemDefault())
        formatter.format(parser)
    }.getOrNull()
}

data class InstallActions(
    val onRefresh: () -> Unit,
    val onPickImage: () -> Unit,
    val onSetBackupFirst: (Boolean) -> Unit,
    val onFlash: () -> Unit,
    val onRebootRecovery: () -> Unit,
)
