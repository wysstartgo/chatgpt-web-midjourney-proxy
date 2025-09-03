<template>
	<div class="relative min-h-screen overflow-hidden bg-[radial-gradient(ellipse_at_top_left,_var(--tw-gradient-stops))] from-[#0e1028] via-[#0b0f2e] to-[#020617]">
		<!-- 动态霓虹背景粒子/光斑 -->
		<div class="pointer-events-none absolute -top-24 -left-24 h-72 w-72 rounded-full bg-gradient-to-br from-indigo-500/30 via-fuchsia-500/30 to-cyan-500/30 blur-3xl animate-pulse" />
		<div class="pointer-events-none absolute -bottom-32 -right-24 h-80 w-80 rounded-full bg-gradient-to-tr from-sky-500/20 via-purple-500/20 to-emerald-500/20 blur-3xl animate-[float_10s_ease-in-out_infinite]" />

		<div class="relative z-10 flex min-h-screen items-center justify-center p-4">
			<n-card
				class="w-full max-w-5xl backdrop-blur-xl bg-white/5 border border-white/10 shadow-2xl shadow-indigo-500/10"
				:segmented="{ content: true, footer: 'soft' }"
				embedded
			>
				<template #header>
					<div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
						<div class="flex items-center gap-3">
							<svg class="h-6 w-6" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
								<path d="M12 2L20 6V10C20 15.52 16.42 20.74 12 22C7.58 20.74 4 15.52 4 10V6L12 2Z" stroke="currentColor" stroke-width="1.5" class="text-indigo-300" />
								<path d="M9 12L11 14L15 10" stroke="currentColor" stroke-width="1.5" class="text-emerald-300" />
							</svg>
							<n-gradient-text type="info" size="20">任务运行面板 · Tauri × Vue3 × Naive UI</n-gradient-text>
						</div>
						<div class="flex items-center gap-2 text-xs text-white/60">
							<span class="hidden sm:inline">本地安全执行 · 零网络暴露（Tauri）</span>
							<n-tag size="small" type="info" round>v1.1 日志增强</n-tag>
						</div>
					</div>
				</template>

				<div class="grid grid-cols-1 gap-6 xl:grid-cols-2">
					<!-- 左：表单区 -->
					<div class="space-y-6">
						<div class="rounded-2xl bg-gradient-to-r from-indigo-500/10 via-fuchsia-500/10 to-cyan-500/10 p-4 border border-white/10">
							<p class="text-sm text-white/70 leading-relaxed">
								填写 <span class="text-white/90">用户ID</span> 与 <span class="text-white/90">用户令牌</span>，点击
								<span class="text-indigo-300">运行</span> 即可通过 <span class="text-fuchsia-300">Tauri</span> 调用后端执行任务。
							</p>
						</div>

						<n-form ref="formRef" :model="form" :rules="rules" label-placement="left" label-width="96">
							<n-form-item label="用户ID" path="user_id">
								<n-input v-model:value="form.user_id" placeholder="请输入用户ID" clearable maxlength="64">
									<template #prefix>
										<i class="i-heroicons-user-20-solid text-white/50" />
									</template>
								</n-input>
							</n-form-item>

							<n-form-item label="用户令牌" path="user_token">
								<n-input v-model:value="form.user_token" placeholder="请输入用户令牌" clearable type="password" show-password-on="mousedown" maxlength="128">
									<template #prefix>
										<i class="i-heroicons-key-20-solid text-white/50" />
									</template>
								</n-input>
							</n-form-item>

							<div class="mt-2 flex flex-wrap items-center justify-between gap-3">
								<n-space>
									<n-button size="large" type="primary" strong round :loading="isRunning" :disabled="isRunning" @click="handleRun">
										<template #icon><i class="i-heroicons-bolt-20-solid" /></template>
										运行
									</n-button>
									<n-button quaternary size="large" @click="openStatusModal" :disabled="!lastRunId && !isRunning">
										<template #icon><i class="i-heroicons-eye-20-solid" /></template>
										查看运行状态
									</n-button>
								</n-space>

								<div class="text-right text-xs text-white/60">
									<div class="flex items-center gap-2" v-if="isRunning">
										<n-spin size="small" />
										<span>运行中… 请稍候</span>
									</div>
									<div v-else-if="lastRunAt">上次运行：{{ lastRunAt }}</div>
								</div>
							</div>
						</n-form>

						<!-- 日志控制条（轻量版） -->
						<div class="rounded-xl border border-white/10 bg-white/5 p-3">
							<div class="flex flex-wrap items-center gap-2">
								<n-dropdown :options="levelOptions" @select="changeLevel">
									<n-button size="small" quaternary>日志级别：{{ levelLabel }}</n-button>
								</n-dropdown>
								<n-switch v-model:value="autoScroll" size="small"> <template #checked>自动滚动</template><template #unchecked>手动</template> </n-switch>
								<n-checkbox v-model:checked="onlyCurrentRun" size="small">只看当前 Run</n-checkbox>
								<n-button size="small" secondary @click="clearAllLogs">清空</n-button>
								<n-button size="small" quaternary @click="pullLogs()">拉取增量</n-button>
								<n-button size="small" quaternary @click="exportLogs">导出</n-button>
							</div>
						</div>
					</div>

					<!-- 右：实时日志区（概览） -->
					<div class="rounded-2xl border border-white/10 bg-black/40 p-0 overflow-hidden">
						<div class="flex items-center justify-between px-4 py-3 border-b border-white/10">
							<div class="text-sm text-white/80">实时日志（最新 500 行）</div>
							<n-tag :type="isRunning ? 'success' : 'default'" size="small" round>
								{{ isRunning ? '运行中' : '空闲' }}
							</n-tag>
						</div>
						<div ref="logScrollRef" class="h-[420px] overflow-auto">
							<ol class="divide-y divide-white/5">
								<li v-for="(row, idx) in visibleLogRows" :key="idx" class="px-4 py-1 font-mono text-[12px] leading-5 whitespace-pre-wrap">
									<span class="text-white/40">{{ row.ts }}</span>
									<span class="mx-2" :class="levelClass(row.level)">[{{ row.level }}]</span>
									<span class="text-white/70">{{ row.message }}</span>
								</li>
							</ol>
						</div>
					</div>
				</div>

				<template #footer>
					<div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between text-xs text-white/50">
						<span>• UI 由 Naive UI + Tailwind 驱动</span>
						<span>• 日志：事件 <code>app_log</code> + 拉取 <code>get_logs</code></span>
					</div>
				</template>
			</n-card>
		</div>

		<!-- 运行状态弹窗（保留） -->
		<n-modal v-model:show="showStatus" preset="card" class="max-w-3xl w-[92vw]">
			<template #header>任务运行状态</template>
			<div class="space-y-4">
				<n-alert v-if="isRunning" type="info" :closable="false" title="执行中">正在执行任务（Run ID: {{ lastRunId || '—' }}）。</n-alert>
				<n-empty v-else-if="!runLogs.length" description="暂无运行日志" />
				<div v-else class="h-64 overflow-auto rounded-lg bg-black/50 p-3 border border-white/10">
					<ol class="space-y-2">
						<li v-for="(line, idx) in runLogs" :key="idx" class="font-mono text-xs text-emerald-200/90">
							<span class="text-white/40">#{{ idx + 1 }}:</span>
							<span class="ml-2">{{ line }}</span>
						</li>
					</ol>
				</div>
			</div>
			<template #footer>
				<n-space justify="end">
					<n-button quaternary @click="showStatus = false">关闭</n-button>
				</n-space>
			</template>
		</n-modal>
	</div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, nextTick } from 'vue'
import type { FormInst, FormRules } from 'naive-ui'
import { useMessage } from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

interface RunForm { user_id: string; user_token: string }
interface LogEntry { id: number; ts: string; level: string; target?: string; message: string; runId?: string }

const message = useMessage()
const formRef = ref<FormInst | null>(null)
const form = ref<RunForm>({ user_id: '', user_token: '' })
const isRunning = ref(false)
const lastRunAt = ref<string | null>(null)
const lastRunId = ref<string | null>(null)
const showStatus = ref(false)

// —— 日志状态 ——
const runLogs = ref<string[]>([])              // 旧的状态弹窗日志
const logRows = ref<LogEntry[]>([])            // 全局结构化日志（来自 app_log / get_logs）
let lastLogId = 0                              // 增量游标
const autoScroll = ref(true)
const onlyCurrentRun = ref(false)
const levelLabel = ref('INFO')

const levelOptions = [
	{ label: 'TRACE', key: 'trace' },
	{ label: 'DEBUG', key: 'debug' },
	{ label: 'INFO',  key: 'info'  },
	{ label: 'WARN',  key: 'warn'  },
	{ label: 'ERROR', key: 'error' },
	{ label: 'OFF',   key: 'off'   }
]

const rules: FormRules = {
	user_id: [{ required: true, message: '请输入用户ID', trigger: ['input', 'blur'] }],
	user_token: [{ required: true, message: '请输入用户令牌', trigger: ['input', 'blur'] }]
}

const logScrollRef = ref<HTMLElement | null>(null)

function levelClass(level: string) {
	const l = level.toUpperCase()
	if (l === 'ERROR') return 'text-red-400'
	if (l === 'WARN' || l === 'WARNING') return 'text-amber-300'
	if (l === 'DEBUG') return 'text-sky-300'
	if (l === 'TRACE') return 'text-fuchsia-300'
	return 'text-emerald-300'
}

const visibleLogRows = computed(() => {
	const rows = onlyCurrentRun.value && lastRunId.value
		? logRows.value.filter(r => !r.runId || r.runId === lastRunId.value)
		: logRows.value
	return rows.slice(-500) // 只显示最新 500 条
})

function scrollToBottomSoon() {
	if (!autoScroll.value) return
	nextTick(() => {
		const el = logScrollRef.value
		if (!el) return
		el.scrollTop = el.scrollHeight
	})
}

async function handleRun() {
	try { await formRef.value?.validate() } catch { return }
	if (isRunning.value) return
	isRunning.value = true
	runLogs.value = []
	showStatus.value = true
	lastRunId.value = `${Date.now()}`
	try {
		await invoke('run_task', {
			user_id: form.value.user_id,
			user_token: form.value.user_token,
			run_id: lastRunId.value
		})
		message.success('任务执行完成')
		runLogs.value.push('✅ 任务执行完成')
	} catch (err: any) {
		message.error(`执行失败：${err?.message || err}`)
		runLogs.value.push(`❌ 执行失败：${err?.message || String(err)}`)
	} finally {
		isRunning.value = false
		lastRunAt.value = new Date().toLocaleString()
	}
}

function openStatusModal() { showStatus.value = true }

// —— 日志：事件订阅 + 增量拉取 ——
let unlistenAppLog: (() => void) | null = null
let unlistenRunLog: (() => void) | null = null

async function startListenLogs() {
	// 结构化日志事件（来自自定义 logger）
	unlistenAppLog = await listen<LogEntry>('app_log', (e) => {
		const entry = e.payload
		logRows.value.push(entry)
		scrollToBottomSoon()
	})
	// 兼容旧的 run_log 文本事件
	unlistenRunLog = await listen<any>('run_log', (e) => {
		const p = e.payload as { runId?: string; line: string; ts: string }
		logRows.value.push({ id: ++lastLogId, ts: p.ts, level: 'INFO', message: p.line, runId: p.runId })
		scrollToBottomSoon()
	})
}

async function stopListenLogs() {
	unlistenAppLog?.(); unlistenAppLog = null
	unlistenRunLog?.(); unlistenRunLog = null
}

async function pullLogs(limit = 800) {
	try {
		const [entries, latest] = await invoke<[LogEntry[], number]>('get_logs', { args: { since_id: lastLogId || 0, limit } } as any)
		if (entries.length) {
			// 合并并去重（按 id 递增即可）
			for (const e of entries) {
				logRows.value.push(e)
				lastLogId = e.id
			}
			scrollToBottomSoon()
		} else {
			lastLogId = Math.max(latest || 0, lastLogId)
		}
	} catch (e) {
		// 忽略一次失败，控制台查看
		console.warn('pullLogs failed', e)
	}
}

async function clearAllLogs() {
	await invoke('clear_logs')
	logRows.value = []
	lastLogId = 0
}

async function changeLevel(levelKey: string) {
	try {
		await invoke('set_log_level', { level: levelKey })
		levelLabel.value = levelKey.toUpperCase()
		message.success(`日志级别已切换为 ${levelLabel.value}`)
	} catch (e: any) {
		message.error(`切换失败：${e?.message || e}`)
	}
}

function exportLogs() {
	// 简易导出为文本
	const lines = visibleLogRows.value.map(r => `${r.ts} [${r.level}] ${r.message}`)
	//const blob = new Blob([lines.join('\n')], { type: 'text/plain;charset=utf-8' })
	const bom = '\uFEFF'
	const blob = new Blob([bom + lines.join('\n')], { type: 'text/plain;charset=utf-8' })
	const url = URL.createObjectURL(blob)
	const a = document.createElement('a')
	a.href = url
	a.download = `logs_${Date.now()}.txt`
	a.click()
	URL.revokeObjectURL(url)
}

onMounted(async () => {
	await startListenLogs()
	await pullLogs(1000) // 启动时补一批历史
	scrollToBottomSoon()
})

onBeforeUnmount(async () => {
	await stopListenLogs()
})
</script>

<style scoped>
@keyframes float { 0%,100% { transform: translateY(0) } 50% { transform: translateY(-8px) } }

/* Naive UI 暗色微调 */
:deep(.n-card) { color: #e5e7eb; }
:deep(.n-form-item-label__text) { color: #cbd5e1; }
:deep(.n-input) {
	--n-color: rgba(255,255,255,0.04);
	--n-border: rgba(255,255,255,0.12);
	--n-text-color: #e5e7eb;
}
:deep(.n-input__placeholder) { color: rgba(255,255,255,0.45); }
:deep(.n-button--primary) { box-shadow: 0 8px 24px rgba(99,102,241,0.35); }

/* 图标占位（如未用 Iconify 可先用 emoji 代替） */
.i-heroicons-user-20-solid::before { content: "👤"; }
.i-heroicons-key-20-solid::before { content: "🔑"; }
.i-heroicons-bolt-20-solid::before { content: "⚡"; }
.i-heroicons-eye-20-solid::before { content: "👁️"; }
</style>
