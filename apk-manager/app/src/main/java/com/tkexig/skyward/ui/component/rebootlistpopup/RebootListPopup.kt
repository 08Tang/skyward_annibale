package com.tkexig.skyward.ui.component.rebootlistpopup

import androidx.annotation.StringRes
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import com.tkexig.skyward.Natives
import com.tkexig.skyward.R
import com.tkexig.skyward.ui.LocalUiMode
import com.tkexig.skyward.ui.UiMode
import com.tkexig.skyward.ui.component.dialog.rememberConfirmDialog
import com.tkexig.skyward.ui.util.reboot

data class RebootListOption(
    @param:StringRes val labelRes: Int,
    val reason: String,
)

/**
 * 重启菜单里的选项。
 *
 * 原来这里是七项：普通重启、userspace、软重启、Recovery、bootloader、
 * download、EDL。这一版只留「重启到 Recovery」——其余几项要么是 SukiSU
 * 交接内核模块用的（软重启、userspace），要么是刷机时才用得上的危险入口
 * （bootloader、download、EDL），这个 APK 不该给。
 */
@Composable
fun getRebootListOption(): List<RebootListOption> =
    listOf(RebootListOption(R.string.reboot_recovery, "recovery"))


@Composable
fun rememberRebootAction(): (String) -> Unit {
    val title = stringResource(R.string.reboot)
    val message = stringResource(R.string.jailbreak_reboot_warning)
    val confirmDialog = rememberConfirmDialog(onConfirm = { reboot() })

    return remember(title, message, confirmDialog) {
        { reason ->
            if (Natives.isLateLoadMode && reason.isEmpty()) {
                confirmDialog.showConfirm(title = title, content = message)
            } else {
                reboot(reason)
            }
        }
    }
}

@Composable
fun RebootListPopup() {
    when (LocalUiMode.current) {
        UiMode.Miuix -> RebootListPopupMiuix()
        UiMode.Material -> RebootListPopupMaterial()
    }
}
