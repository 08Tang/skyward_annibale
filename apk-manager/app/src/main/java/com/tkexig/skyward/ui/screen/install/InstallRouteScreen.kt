package com.tkexig.skyward.ui.screen.install

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import com.tkexig.skyward.ui.util.reboot

/**
 * 刷机页的入口，把 [InstallViewModel] 和 [InstallScreen] 接起来。
 *
 * 选镜像用系统的文件选择器：刷机包可能在任何地方（下载目录、U 盘），不该
 * 让应用自己猜路径。
 */
@Composable
fun InstallRouteScreen(
    onBack: () -> Unit,
) {
    val viewModel = viewModel<InstallViewModel>()
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    val picker = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.OpenDocument(),
    ) { uri ->
        if (uri != null) viewModel.pickImage(uri)
    }

    InstallScreen(
        state = state,
        bottomInnerPadding = 0.dp,
        onBack = onBack,
        actions = InstallActions(
            onRefresh = viewModel::refresh,
            onPickImage = {
                // image/* 覆盖 .img；zip 的 MIME 各家报得不一致，一并列上。
                picker.launch(arrayOf("image/*", "application/zip", "application/octet-stream"))
            },
            onSetBackupFirst = viewModel::setBackupFirst,
            onFlash = viewModel::flash,
            onRebootRecovery = { reboot("recovery") },
        ),
    )
}
