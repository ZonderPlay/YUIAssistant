package com.yui.assistant.ui

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.yui.assistant.ui.components.VoiceButton

@Composable
fun MainScreen(
    paddingValues: PaddingValues = PaddingValues(0.dp),
    vm: MainViewModel = viewModel(),
) {
    val state by vm.state.collectAsState()
    val listState = rememberLazyListState()

    LaunchedEffect(state.messages.size) {
        if (state.messages.isNotEmpty()) listState.animateScrollToItem(state.messages.size - 1)
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(paddingValues)
            .padding(horizontal = 24.dp, vertical = 12.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text("Юи — голосовой ассистент", style = MaterialTheme.typography.headlineSmall)
        Spacer(Modifier.height(8.dp))
        Text(
            when (state.phase) {
                AssistantPhase.Idle -> "Нажмите кнопку и говорите"
                AssistantPhase.Recording -> "Слушаю… нажмите ещё раз, чтобы остановить"
                AssistantPhase.Sending -> "Думаю…"
                AssistantPhase.Speaking -> "Отвечаю…"
            },
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.height(16.dp))

        LazyColumn(
            state = listState,
            modifier = Modifier
                .weight(1f)
                .fillMaxWidth(),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            items(state.messages) { msg -> ChatBubble(msg) }
        }

        Spacer(Modifier.height(16.dp))
        VoiceButton(
            phase = state.phase,
            onToggleRecord = { vm.onMicClick() },
        )
    }
}

@Composable
private fun ChatBubble(msg: ChatMessage) {
    Surface(
        shape = MaterialTheme.shapes.medium,
        color = if (msg.fromUser) MaterialTheme.colorScheme.secondaryContainer
        else MaterialTheme.colorScheme.primaryContainer,
        modifier = Modifier.fillMaxWidth(),
    ) {
        Text(
            text = msg.text,
            modifier = Modifier.padding(12.dp),
            style = MaterialTheme.typography.bodyLarge,
        )
    }
}
