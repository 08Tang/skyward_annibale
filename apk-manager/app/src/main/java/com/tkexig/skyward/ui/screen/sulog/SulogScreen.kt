package com.tkexig.skyward.ui.screen.sulog

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel

/**
 * 运行日志。
 *
 * 显示的是 Skyward Manager 自己说的话：它调了什么命令、拿到什么结果、哪一
 * 步失败了。不是内核日志，也不是别的管理器的日志——那些东西这个应用已经
 * 不再读，留在这里只会让报告变长。
 */
@Composable
fun SulogScreen() {
    val viewModel = viewModel<LogViewModel>()
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    var query by remember { mutableStateOf("") }

    LaunchedEffect(Unit) { viewModel.refresh() }

    Column(modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp)) {
        Spacer(Modifier.height(8.dp))
        Text("运行日志", style = MaterialTheme.typography.headlineSmall)
        Text(
            text = state.summary,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )

        Spacer(Modifier.height(8.dp))
        OutlinedTextField(
            value = query,
            onValueChange = { query = it },
            label = { Text("搜索") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )

        Spacer(Modifier.height(8.dp))
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            OutlinedButton(onClick = viewModel::refresh, modifier = Modifier.weight(1f)) {
                Text("刷新")
            }
            OutlinedButton(onClick = { viewModel.clear(); }, modifier = Modifier.weight(1f)) {
                Text("清空")
            }
        }

        Spacer(Modifier.height(8.dp))
        val lines = state.lines.filter { query.isBlank() || it.contains(query, ignoreCase = true) }
        val listState = rememberLazyListState()
        LazyColumn(state = listState, modifier = Modifier.fillMaxWidth()) {
            items(lines) { line ->
                Text(
                    text = line,
                    style = MaterialTheme.typography.bodySmall,
                    fontFamily = FontFamily.Monospace,
                    modifier = Modifier.padding(vertical = 1.dp),
                )
            }
            item { Spacer(Modifier.height(24.dp)) }
        }
    }
}
