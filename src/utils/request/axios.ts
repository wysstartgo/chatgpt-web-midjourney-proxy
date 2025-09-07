// axios.ts
import axios, { AxiosError, type AxiosResponse } from 'axios'
import { useAuthStore } from '@/store'

// 是否在 Tauri 环境运行
const isTauri = typeof window !== 'undefined' &&
	(('__TAURI__' in window) || ('__TAURI_INTERNALS__' in window))

/**
 * 生产(Tauri)必须是绝对地址；开发可用 /api（走 Vite 代理）
 * 推荐在 .env.tauri 或 CI 注入 VITE_TAURI_API_BASE_URL
 */
const TAURI_BASE = import.meta.env.VITE_TAURI_API_BASE_URL as string | undefined
const WEB_BASE   = import.meta.env.VITE_GLOB_API_URL as string | undefined

// 计算最终 baseURL
let baseURL = isTauri ? (TAURI_BASE || WEB_BASE) : (WEB_BASE || '/api')

// 防御：Tauri 下绝对地址必需
if (isTauri && baseURL && baseURL.startsWith('/')) {
	console.error('[axios] 在 Tauri 生产环境不能使用相对路径作为 baseURL：', baseURL)
	// 兜底：可改成你的后端地址，避免直奔 tauri://localhost
	baseURL = 'http://127.0.0.1:3002'
}

console.log('[axios baseURL]', baseURL)

const service = axios.create({
	baseURL,
	timeout: 20000,
	// 仅把 2xx 当成功；其余走错误分支，便于统一处理
	validateStatus: s => s >= 200 && s < 300,
})

// 在上面“方案A”的 axios.ts 基础上，追加这段：
// axios.ts 里（只在 Tauri 环境启用这个适配器）
import { fetch as tFetch } from '@tauri-apps/plugin-http'

if (isTauri) {
	(service.defaults as any).adapter = async (config) => {
		const url = (config.baseURL || '') + (config.url || '')
		const method = (config.method || 'GET').toUpperCase()

		const resp = await tFetch(url, {
			method: method as any,
			headers: config.headers as Record<string, string>,
			body: config.data,           // 若是 JSON，确保上游已设置 'Content-Type: application/json'
			// 其他可选项：timeout、proxy、danger 等见官方文档
		})

		// 选择解析方式：优先按 axios 的 responseType，再看 content-type
		const ct = resp.headers.get('content-type') || ''
		let data: any

		if (config.responseType === 'arraybuffer') {
			data = await resp.arrayBuffer()
		} else if (config.responseType === 'blob') {
			// WebView 环境未必有原生 Blob，这里退化为 ArrayBuffer
			data = await resp.arrayBuffer()
		} else if (ct.includes('application/json')) {
			data = await resp.json()
		} else {
			data = await resp.text()
		}

		// 调试日志（可保留几天排查）：
		console.log('[HTTP]', resp.status, url, ct, typeof data === 'string' ? data.slice(0, 200) : data)

		// 返回 axios 期望的对象结构
		return {
			data,
			status: resp.status,
			statusText: resp.statusText,
			headers: Object.fromEntries(resp.headers.entries()),
			config,
			request: {} as any,
		}
	}
}



service.interceptors.request.use(
	(config) => {
		// 如果历史代码里写了 url 以 /api 开头，Tauri 下可去掉前缀（可选）
		if (isTauri && typeof config.url === 'string') {
			config.url = config.url.replace(/^\/api\//, '/')
		}
		const token = useAuthStore().token
		if (token) config.headers.Authorization = `Bearer ${token}`
		// 常见预检需要的头
		if (!config.headers['Content-Type'] && config.method !== 'get') {
			config.headers['Content-Type'] = 'application/json'
		}
		return config
	},
	(error) => Promise.reject(error)
)

service.interceptors.response.use(
	(response: AxiosResponse) => response,
	(error: AxiosError<any>) => {
		const status = error.response?.status
		const url    = error.response?.config?.url
		const data   = error.response?.data
		console.error('[HTTP ERR]', status, url, typeof data === 'string' ? data.slice(0,200) : data)
		return Promise.reject(error)
	},
)

export default service
