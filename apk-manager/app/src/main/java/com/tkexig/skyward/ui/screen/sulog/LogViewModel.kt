package com.tkexig.skyward.ui.screen.sulog

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.tkexig.skyward.data.log.AppLog
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

data class LogUiState(
    val lines: List<String> = emptyList(),
    val summary: String = "",
)

/** 读 [AppLog] 写下来的那份运行日志。 */
class LogViewModel : ViewModel() {

    private val _uiState = MutableStateFlow(LogUiState())
    val uiState: StateFlow<LogUiState> = _uiState.asStateFlow()

    init {
        refresh()
    }

    fun refresh() {
        viewModelScope.launch {
            val text = withContext(Dispatchers.IO) { AppLog.read() }
            val lines = text.lines().filter { it.isNotBlank() }
            _uiState.update {
                LogUiState(
                    lines = lines,
                    summary = if (lines.isEmpty()) {
                        "还没有日志。用过一次之后这里会有内容。"
                    } else {
                        "${lines.size} 行 · ${text.toByteArray().size / 1024} KB"
                    },
                )
            }
        }
    }

    fun clear() {
        viewModelScope.launch {
            withContext(Dispatchers.IO) { AppLog.clear() }
            refresh()
        }
    }
}
