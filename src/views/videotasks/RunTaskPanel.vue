<template>
	<div class="relative h-full w-full overflow-hidden isolate bg-[radial-gradient(ellipse_at_top_left,_var(--tw-gradient-stops))] from-[#0d1024] via-[#0a0e24] to-[#050812]">
		<!-- 装饰背景 -->
		<div class="pointer-events-none absolute -top-12 -left-12 h-36 w-36 rounded-full bg-gradient-to-br from-indigo-500/15 via-fuchsia-500/15 to-cyan-500/15 blur-3xl" />
		<div class="pointer-events-none absolute -bottom-20 -right-12 h-44 w-44 rounded-full bg-gradient-to-tr from-sky-500/12 via-purple-500/12 to-emerald-500/12 blur-3xl" />

		<div class="relative z-10 flex h-full items-start p-4">
			<n-card class="w-full h-full flex flex-col backdrop-blur-xl bg-white/[0.06] border border-white/10 shadow-[0_10px_30px_rgba(2,8,23,0.45)]"
							:segmented="{ content: true, footer: 'soft' }" embedded>
				<template #header>
					<div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
						<div class="flex items-center gap-3">
							<svg class="h-6 w-6 text-indigo-300" viewBox="0 0 24 24" fill="none">
								<path d="M12 2L20 6V10C20 15.52 16.42 20.74 12 22C7.58 20.74 4 15.52 4 10V6L12 2Z" stroke="currentColor" stroke-width="1.5" />
								<path d="M9 12L11 14L15 10" stroke="currentColor" stroke-width="1.5" class="text-emerald-300" />
							</svg>
							<n-gradient-text type="info" size="20">任务运行面板 · 聚宝盆</n-gradient-text>
						</div>
						<div class="flex items-center gap-2 text-xs text-white/70">
							<span class="hidden sm:inline">后台执行 · 事件/轮询汇报进度</span>
							<n-tag size="small" type="info" round>{{ lastRunId ? `Run: ${lastRunId}` : 'Idle' }}</n-tag>
						</div>
					</div>
				</template>

				<div class="flex-1 overflow-auto">
					<div class="space-y-5 pb-4 max-w-[720px]">
						<!-- 提示 -->
						<div class="rounded-2xl bg-gradient-to-r from-indigo-500/15 via-fuchsia-500/15 to-cyan-500/15 p-4 border border-white/10">
							<p class="text-[13px] text-white/80 leading-relaxed">
								任务在后台运行，前端<strong>不会等待</strong>完成。可通过事件/轮询查看进度，支持取消与导出日志。
							</p>
						</div>

						<!-- 表单 -->
						<div class="rounded-2xl border border-white/10 bg-[#0b1220]/70 p-4">
							<n-form ref="formRef" :model="form" :rules="rules" label-placement="left" label-width="96" size="large">
								<n-form-item label="用户ID" path="user_id">
									<n-input v-model:value="form.user_id" placeholder="请输入用户ID" clearable round maxlength="64">
										<template #prefix><i class="i-heroicons-user-20-solid text-white/60" /></template>
									</n-input>
								</n-form-item>
								<n-form-item label="用户令牌" path="user_token">
									<n-input v-model:value="form.user_token" placeholder="请输入用户令牌" type="password"
													 show-password-on="mousedown" clearable round maxlength="128">
										<template #prefix><i class="i-heroicons-key-20-solid text-white/60" /></template>
									</n-input>
								</n-form-item>

								<div class="mt-2 flex flex-wrap items-center justify-between gap-3">
									<div class="flex flex-wrap gap-2">
										<n-button size="large" strong round type="primary" :loading="isRunning" :disabled="isRunning"
															@click="handleRun" class="!h-12 !px-6 text-base shadow-[0_0_24px_rgba(99,102,241,0.45)]">
											<template #icon><i class="i-heroicons-bolt-20-solid" /></template> 启动任务
										</n-button>
										<n-button size="large" strong round secondary :disabled="!lastRunId || !isRunning"
															@click="confirmCancel" class="!h-12 !px-6 text-base">
											取消任务
										</n-button>
										<n-button size="large" strong round tertiary @click="openStatusModal" class="!h-12 !px-6 text-base">
											查看状态
										</n-button>
									</div>
									<div class="text-right text-xs text-white/70">
										<div v-if="isRunning" class="flex items-center gap-2">
											<n-spin size="small" /><span>运行中…</span>
										</div>
										<div v-else-if="lastRunAt">上次：{{ lastRunAt }}</div>
									</div>
								</div>
							</n-form>
						</div>

						<!-- 控制条 -->
						<div class="rounded-2xl border border-white/10 bg-[#0b1220]/70 p-4">
							<div class="flex flex-wrap items-center gap-2">
								<n-dropdown :options="levelOptions" @select="changeLevel">
									<n-button size="large" strong round tertiary class="!h-11 !px-5">日志级别：{{ levelLabel }}</n-button>
								</n-dropdown>
								<n-switch v-model:value="autoScroll" size="large" class="!h-11">
									<template #checked>自动滚动</template><template #unchecked>手动</template>
								</n-switch>
								<n-checkbox v-model:checked="onlyCurrentRun" size="large">只看当前 Run</n-checkbox>
								<n-button size="large" strong round secondary @click="clearAllLogs" class="!h-11 !px-5">清空</n-button>
								<n-button size="large" strong round tertiary @click="pullLogs()" class="!h-11 !px-5">拉取增量</n-button>
								<n-button size="large" strong round tertiary @click="exportLogs" class="!h-11 !px-5">导出</n-button>
							</div>
						</div>

						<!-- 日志 -->
						<div class="rounded-2xl border border-white/10 bg-black/50 overflow-hidden">
							<div class="flex items-center justify-between px-3 py-2 border-b border-white/10">
								<div class="text-xs text-white/80">实时日志（仅渲染最新 500 行）</div>
								<n-tag :type="isRunning ? 'success' : 'default'" size="small" round>{{ isRunning ? '运行中' : '空闲' }}</n-tag>
							</div>
							<div ref="logScrollRef" class="log-scroll h-[240px] md:h-[300px] overflow-auto">
								<ol class="divide-y divide-white/5">
									<li v-for="(row, idx) in visibleLogRows" :key="`${row.id}-${idx}`"
											class="px-3 py-1 font-mono text-[11px] leading-5 whitespace-pre-wrap">
										<span class="text-white/40">{{ formatShanghai(row.ts) }}</span>
										<span class="mx-1" :class="levelClass(row.level)">[{{ row.level }}]</span>
										<span class="text-white/80">{{ row.message }}</span>
									</li>
								</ol>
							</div>
						</div>
					</div>
				</div>

				<template #footer>
					<div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between text-xs text-white/60">
						<span>• 后台执行：start_run_task + 事件 + 轮询</span>
						<span>• 导出：Tauri dialog + fs（需能力授权）</span>
					</div>
				</template>
			</n-card>
		</div>

		<!-- 运行状态弹窗 -->
		<n-modal v-model:show="showStatus" preset="card" class="max-w-3xl w-[92vw]">
			<template #header>任务运行状态</template>
			<div class="space-y-4">
				<n-empty v-if="!lastStatus" description="暂无状态" />
				<div v-else class="space-y-2 text-white/80 text-sm">
					<div>Run ID：<code>{{ lastStatus.run_id }}</code></div>
					<div>状态：{{ lastStatus.state }} · 进度：{{ lastStatus.progress }}%</div>
					<div>开始：{{ formatShanghai(lastStatus.started_at) }}</div>
					<div v-if="lastStatus.finished_at">结束：{{ formatShanghai(lastStatus.finished_at) }}</div>
					<div v-if="lastStatus.last_message">最后信息：{{ lastStatus.last_message }}</div>
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
import { ref, computed, onMounted, onBeforeUnmount, nextTick, watch } from 'vue'
import {
	NCard, NTag, NSpin, NSpace, NButton, NDropdown,
	NSwitch, NCheckbox, NModal, NEmpty,
	NForm, NFormItem, NInput, useMessage, useDialog
} from 'naive-ui'
import type { FormInst, FormRules } from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'

interface RunForm { user_id: string; user_token: string }
interface LogEntry { id: number; ts: string; level: string; target?: string; message: string; runId?: string }
interface TaskStatus { run_id: string; state: string; progress: number; started_at: string; finished_at?: string|null; last_message?: string|null }

const message = useMessage()
const dialog = useDialog()

const formRef = ref<FormInst | null>(null)
const form = ref<RunForm>({ user_id: '', user_token: '' })
const isRunning = ref(false)
const lastRunAt = ref<string | null>(null)
const lastRunId = ref<string | null>(null)
const showStatus = ref(false)
const lastStatus = ref<TaskStatus | null>(null)

const runLogs = ref<string[]>([])
const logRows = ref<LogEntry[]>([])
let lastLogId = 0
const autoScroll = ref(true)
const onlyCurrentRun = ref(false)
const levelLabel = ref('INFO')
const MAX_LOG_ROWS = 5000

// 滚动到底部预留 90px
const BOTTOM_GAP = 90
const AUTO_NEAR_GAP = BOTTOM_GAP + 24
const logScrollRef = ref<HTMLElement | null>(null)
let rafId: number | null = null

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

const isTauri =
	typeof window !== 'undefined' && ('__TAURI_IPC__' in window || '__TAURI_INTERNALS__' in window)

function isNearBottom(el: HTMLElement, gap = AUTO_NEAR_GAP) {
	const targetTop = Math.max(0, el.scrollHeight - el.clientHeight - BOTTOM_GAP)
	return el.scrollTop >= targetTop - gap
}

function formatShanghai(input: string | number | Date | null | undefined) {
	if (input == null) return ''
	const d = input instanceof Date ? input : new Date(input)
	const parts = new Intl.DateTimeFormat('zh-CN', {
		timeZone: 'Asia/Shanghai',
		hour12: false,
		year: 'numeric', month: '2-digit', day: '2-digit',
		hour: '2-digit', minute: '2-digit', second: '2-digit'
	}).formatToParts(d)
	const m = Object.fromEntries(parts.map(p => [p.type, p.value]))
	return `${m.year}-${m.month}-${m.day} ${m.hour}:${m.minute}:${m.second}`
}


async function scrollToBottom(force = false) {
	const el = logScrollRef.value
	if (!el) return
	if (!force && !autoScroll.value && !isNearBottom(el)) return
	await nextTick()
	const targetTop = Math.max(0, el.scrollHeight - el.clientHeight - BOTTOM_GAP)
	requestAnimationFrame(() => { el.scrollTop = targetTop })
}
function scrollToBottomThrottled(force = false) {
	if (rafId) cancelAnimationFrame(rafId)
	rafId = requestAnimationFrame(() => scrollToBottom(force))
}
function trimLogRows() {
	const extra = logRows.value.length - MAX_LOG_ROWS
	if (extra > 0) logRows.value.splice(0, extra)
}
function levelClass(level: string) {
	const l = (level || '').toUpperCase()
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
	return rows.slice(-500) // 仅渲染最新 500
})
watch(() => visibleLogRows.value.length, () => scrollToBottomThrottled())

// 新增：快速拉取一次状态
async function fetchAndSetStatus() {
	if (!lastRunId.value) return
	try {
		const s = await invoke<TaskStatus | null>('get_task_status', { args:{ run_id: lastRunId.value }})
		if (s) lastStatus.value = s
	} catch {}
}

// 改：启动任务后立即设置初始状态，并立刻 fetch 一次
async function handleRun() {
	try { await formRef.value?.validate() } catch { return }
	if (isRunning.value) return
	isRunning.value = true
	runLogs.value = []
	lastStatus.value = null
	lastRunId.value = null

	try {
		const res = await invoke<{ run_id: string; message: string }>('start_run_task', {
			payload: { user_id: form.value.user_id, user_token: form.value.user_token }
		})
		lastRunId.value = res.run_id
		// 先给一个本地初始状态，防止面板空白
		lastStatus.value = {
			run_id: res.run_id,
			state: 'running',
			progress: 0,
			started_at: new Date().toISOString(),
			finished_at: null,
			last_message: '任务开始…'
		}
		message.success(res.message || '任务已启动')
		await fetchAndSetStatus()        // ⬅️ 立即拉一次
		startPollingStatus(res.run_id)   // 开始加速轮询
	} catch (e: any) {
		isRunning.value = false
		message.error(`启动失败：${e?.message || e}`)
	}
}

// 改：打开状态面板时，立刻拉一次
function openStatusModal() {
	showStatus.value = true
	fetchAndSetStatus()
}

/** 新增：取消前确认，防误操作 */
function confirmCancel() {
	if (!lastRunId.value || !isRunning.value) return
	dialog.warning({
		title: '确认取消当前任务？',
		content: '取消后本次运行将立即尝试停止，已完成的步骤不会回滚。',
		positiveText: '确认取消',
		negativeText: '继续运行',
		autoFocus: false,
		onPositiveClick: async () => {
			await doCancel()
		}
	})
}

async function doCancel() {
	if (!lastRunId.value) return
	try {
		const ok = await invoke<boolean>('cancel_task', {args: { run_id: lastRunId.value }})
		message.info(ok ? '已请求取消' : '取消失败/任务不存在')
	} catch (e: any) {
		message.error(`取消失败：${e?.message || e}`)
	}
}

let unlistenAppLog: (() => void) | null = null
let unlistenRunLog: (() => void) | null = null
let unlistenProgress: (() => void) | null = null
let unlistenDone: (() => void) | null = null
let listening = false

async function startListenLogs() {
	if (!isTauri || listening) return
	listening = true

	unlistenAppLog = await listen<LogEntry>('app_log', (e) => {
		const p = e.payload || ({} as any)
		const id = typeof p.id === 'number' ? p.id : ++lastLogId
		logRows.value.push({ ...p, id })
		lastLogId = Math.max(lastLogId, id)
		trimLogRows()
		scrollToBottomThrottled()
	})

	unlistenRunLog = await listen<any>('run_log', (e) => {
		const p = (e.payload || {}) as { runId?: string; line?: string; ts?: string }
		logRows.value.push({
			id: ++lastLogId,
			ts: p.ts || new Date().toISOString(),
			level: 'INFO',
			message: p.line || '',
			runId: p.runId
		})
		trimLogRows()
		scrollToBottomThrottled()
	})

	// 进度事件：刷新 lastStatus
	unlistenProgress = await listen<any>('run_progress', (e) => {
		const p = e.payload as { runId: string; progress: number; message: string; ts: string }
		if (lastRunId.value && p.runId !== lastRunId.value) return

		logRows.value.push({ id: ++lastLogId, ts: p.ts, level: 'INFO', message: `${p.progress}% ${p.message}`, runId: p.runId })
		trimLogRows()
		scrollToBottomThrottled()

		// ⬇️ 同步更新状态
		lastStatus.value = {
			run_id: p.runId,
			state: 'running',
			progress: Math.min(100, Math.max(0, p.progress)) as number,
			started_at: lastStatus.value?.started_at ?? new Date().toISOString(),
			finished_at: null,
			last_message: p.message
		}
	})

	// 完成/取消事件：刷新 lastStatus
	unlistenDone = await listen<any>('run_done', (e) => {
		const p = e.payload as { runId: string; state: string; ts: string }
		if (lastRunId.value && p.runId !== lastRunId.value) return

		isRunning.value = false
		lastRunAt.value = new Date().toLocaleString()
		stopPollingStatus()

		// ⬇️ 同步更新状态（若没 100% 则补满）
		lastStatus.value = {
			run_id: p.runId,
			state: p.state,
			progress: 100,
			started_at: lastStatus.value?.started_at ?? new Date().toISOString(),
			finished_at: p.ts,
			last_message: p.state === 'success' ? '任务完成 ✅' : (p.state === 'canceled' ? '任务已取消' : '任务结束')
		}

		message.success(p.state === 'success' ? '任务完成' : p.state === 'canceled' ? '已取消' : '已结束')
	})

}

async function stopListenLogs() {
	unlistenAppLog?.(); unlistenAppLog = null
	unlistenRunLog?.(); unlistenRunLog = null
	unlistenProgress?.(); unlistenProgress = null
	unlistenDone?.(); unlistenDone = null
	listening = false
}

async function pullLogs(limit = 800) {
	if (!isTauri) return
	try {
		const res = await invoke<any>('get_logs', { args: { since_id: lastLogId || 0, limit } } as any)
		let entries: LogEntry[] = []
		let latest = lastLogId
		if (Array.isArray(res) && res.length === 2) {
			entries = Array.isArray(res[0]) ? res[0] : []
			latest = typeof res[1] === 'number' ? res[1] : lastLogId
		}
		if (entries.length) {
			for (const e of entries) {
				const id = typeof e.id === 'number' ? e.id : ++lastLogId
				logRows.value.push({ ...e, id })
				lastLogId = Math.max(lastLogId, id)
			}
			trimLogRows()
			scrollToBottomThrottled()
		} else {
			lastLogId = Math.max(latest || 0, lastLogId)
		}
	} catch (e) {
		console.warn('pullLogs failed', e)
	}
}

async function clearAllLogs() {
	if (!isTauri) { logRows.value = []; lastLogId = 0; return }
	await invoke('clear_logs')
	logRows.value = []
	lastLogId = 0
}

async function changeLevel(levelKey: string) {
	if (!isTauri) { levelLabel.value = levelKey.toUpperCase(); return }
	try {
		await invoke('set_log_level', { level: levelKey })
		levelLabel.value = levelKey.toUpperCase()
		message.success(`日志级别已切换为 ${levelLabel.value}`)
	} catch (e: any) {
		message.error(`切换失败：${e?.message || e}`)
	}
}

async function exportLogs() {
	const lines = visibleLogRows.value.map(r => `${r.ts} [${r.level}] ${r.message}`)
	const content = '\uFEFF' + lines.join('\n')
	const fileName = `logs_${Date.now()}.txt`

	if (isTauri) {
		try {
			const path = await save({
				defaultPath: fileName,
				title: '保存日志为',
				filters: [{ name: 'Text Log', extensions: ['txt'] }]
			})
			if (!path) {
				message.info('已取消保存')
				return
			}
			await writeTextFile(path, content)
			message.success(`已导出：${path}`)
		} catch (e: any) {
			message.error(`导出失败：${e?.message || e}`)
		}
	} else {
		// 浏览器降级方案
		const blob = new Blob([content], { type: 'text/plain;charset=utf-8' })
		const url = URL.createObjectURL(blob)
		const a = document.createElement('a')
		a.href = url
		a.download = fileName
		document.body.appendChild(a)
		a.click()
		document.body.removeChild(a)
		URL.revokeObjectURL(url)
	}
}

// 状态轮询（兜底）
// 改：轮询频率 3s
let pollTimer: number | null = null
function startPollingStatus(runId: string) {
	stopPollingStatus()
	pollTimer = window.setInterval(async () => {
		try {
			const s = await invoke<TaskStatus | null>('get_task_status', {args:{ run_id: runId }})
			if (s) lastStatus.value = s
		} catch {}
	}, 3000) // 3s
}
function stopPollingStatus() {
	if (pollTimer) clearInterval(pollTimer)
	pollTimer = null
}


onMounted(async () => {
	await stopListenLogs()
	await startListenLogs()
	await pullLogs(600)
	await scrollToBottom(true)
})

onBeforeUnmount(async () => {
	await stopListenLogs()
	stopPollingStatus()
})
</script>

<style scoped>
.log-scroll {
	scroll-behavior: smooth;
	overscroll-behavior: contain;
}

/* Naive UI 暗色微调 */
:deep(.n-card) { color: #e5e7eb; }
:deep(.n-form-item-label__text) { color: #d1d5db; font-weight: 500; }
:deep(.n-input) {
	--n-height-large: 44px;
	--n-border-radius: 9999px;
	--n-color: rgba(15, 23, 42, 0.7);
	--n-color-focus: rgba(15, 23, 42, 0.9);
	--n-border: rgba(203, 213, 225, 0.18);
	--n-border-hover: rgba(129, 140, 248, 0.45);
	--n-border-focus: rgba(129, 140, 248, 0.85);
	--n-text-color: #e5e7eb;
}
:deep(.n-input__placeholder) { color: rgba(226, 232, 240, 0.6); }
:deep(.n-button--primary) { box-shadow: 0 8px 24px rgba(99,102,241,0.35); }

/* 图标占位（如未接 Iconify 可先用 emoji） */
.i-heroicons-user-20-solid::before { content: "👤"; }
.i-heroicons-key-20-solid::before { content: "🔑"; }
.i-heroicons-bolt-20-solid::before { content: "⚡"; }
.i-heroicons-eye-20-solid::before { content: "👁️"; }
</style>
