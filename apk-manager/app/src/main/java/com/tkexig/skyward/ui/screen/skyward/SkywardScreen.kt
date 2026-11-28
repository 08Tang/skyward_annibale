package com.tkexig.skyward.ui.screen.skyward

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import com.tkexig.skyward.ui.LocalUiMode
import com.tkexig.skyward.ui.UiMode

/**
 * Skyward 的配置页。
 *
 * 这一页是「把 SukiSU 那些用不了的功能换掉」的落点：快充档位、温控、亮度、
 * 灭屏时间、时区、静态 IP，以及把整份配置导出/导入成 JSON。读写的都是
 * /metadata/skyward/settings.json——Skyward Recovery 自己那份。
 */
@Composable
fun SkywardPager(
    bottomInnerPadding: androidx.compose.ui.unit.Dp,
    isCurrentPage: Boolean,
) {
    val viewModel = viewModel<SkywardViewModel>()
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    when (LocalUiMode.current) {
        UiMode.Miuix -> SkywardScreenMiuix(
            state = state,
            bottomInnerPadding = bottomInnerPadding,
            actions = skywardActions(viewModel),
        )

        UiMode.Material -> SkywardScreenMaterial(
            state = state,
            bottomInnerPadding = bottomInnerPadding,
            actions = skywardActions(viewModel),
        )
    }
}

@Composable
private fun skywardActions(viewModel: SkywardViewModel) = SkywardActions(
    onRefresh = viewModel::refresh,
    onSave = viewModel::save,
    onEdit = viewModel::edit,
    onSelectDevice = viewModel::selectDevice,
    onStageImport = viewModel::stageImport,
    onDismissMessage = viewModel::clearMessage,
)

data class SkywardActions(
    val onRefresh: () -> Unit,
    val onSave: () -> Unit,
    val onEdit: ((com.tkexig.skyward.data.skyward.SkywardSettings) -> com.tkexig.skyward.data.skyward.SkywardSettings) -> Unit,
    val onStageImport: (String) -> Unit,
    val onDismissMessage: () -> Unit,
    /** 手动指定机型；null 表示回到按代号自动识别。 */
    val onSelectDevice: (com.tkexig.skyward.data.skyward.SkywardDevice?) -> Unit,
)
