package com.tkexig.skyward.ui.component.choosekmidialog

import androidx.compose.runtime.Composable
import com.tkexig.skyward.ui.LocalUiMode
import com.tkexig.skyward.ui.UiMode

@Composable
fun ChooseKmiDialog(
    show: Boolean,
    onDismissRequest: () -> Unit,
    onSelected: (String?) -> Unit
) {
    when (LocalUiMode.current) {
        UiMode.Miuix -> ChooseKmiDialogMiuix(show, onDismissRequest, onSelected)
        UiMode.Material -> ChooseKmiDialogMaterial(show, onDismissRequest, onSelected)
    }
}