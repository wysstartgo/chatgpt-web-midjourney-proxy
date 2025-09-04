<script setup lang="ts">
import { computed, nextTick } from 'vue'
import { NLayout, NLayoutContent } from 'naive-ui'
import { useRouter, useRoute } from 'vue-router'
import Permission from '../chat/layout/Permission.vue'
import { useBasicLayout } from '@/hooks/useBasicLayout'
import { homeStore, useAppStore, useAuthStore, useChatStore } from '@/store'
import { aiSider, aiFooter } from '@/views/mj'
import aiMobileMenu from '@/views/mj/aiMobileMenu.vue'
import { mlog } from '@/api'

const router = useRouter()
const route = useRoute()
const appStore = useAppStore()
const chatStore = useChatStore()
const authStore = useAuthStore()

// —— 路由与状态初始化（避免死循环重定向，做保护）——
mlog('layout.enter', route.name)
nextTick(() => {
	const targetName = (route.name as string) || 'video'
	// 仅当缺少 uuid 或路由名为空时才补齐，避免重复 replace
	if (!route.params?.uuid || !route.name) {
		router.replace({ name: targetName, params: { uuid: chatStore.active } })
	}
	homeStore.setMyData({ local: targetName })
})

const { isMobile } = useBasicLayout()

// 折叠状态（由全局设置控制）
const collapsed = computed(() => appStore.siderCollapsed)

// 是否需要权限弹窗
const needPermission = computed(() => !!authStore.session?.auth && !authStore.token)

// 外层卡片在移动端去圆角/阴影
const getMobileClass = computed(() => {
	return isMobile.value ? ['rounded-none', 'shadow-none'] : ['shadow-md', 'dark:border-neutral-800']
})

// 容器尺寸/动画
const getContainerClass = computed(() => [
	'h-full',
	'transition-all',
	{ 'layout-with-sider-expanded': !isMobile.value && !collapsed.value },
])

// 侧边栏位置：桌面放右侧，移动端放左侧
const siderPlacement = computed(() => (isMobile.value ? 'left' : 'right'))
</script>

<template>
	<!-- 外层自适应高度；移动端预留底部菜单高度（h55） -->
	<div class="dark:bg-[#24272e] transition-all p-0" :class="[isMobile ? 'h55' : 'h-full']">
		<div class="h-full overflow-hidden" :class="getMobileClass">
			<NLayout
				class="z-40 transition-all"
				:class="getContainerClass"
				has-sider
				:sider-placement="siderPlacement"
			>
				<!-- 桌面侧边栏（随窗口自适应宽度，右侧吸附） -->
				<aiSider v-if="!isMobile" />

				<!-- 主内容：始终占满剩余空间，独立滚动 -->
				<NLayoutContent class="h-full">
					<RouterView v-slot="{ Component, route }">
						<component :is="Component" :key="route.fullPath" />
					</RouterView>
				</NLayoutContent>
			</NLayout>
		</div>

		<!-- 权限弹窗 -->
		<Permission :visible="needPermission" />
	</div>

	<!-- 移动端底部菜单（固定高度 55px，与 .h55 同步） -->
	<aiMobileMenu v-if="isMobile" />
	<!-- 全局页脚（按需保留） -->
	<aiFooter />
</template>

<style>
/* 移动端时主体高度扣除底部菜单高度，防止被遮挡 */
:root {
	--mobile-menu-height: 55px;
}
.h55 {
	height: calc(100% - var(--mobile-menu-height));
}

/* 桌面下，未折叠侧边时可选的额外样式钩子（留给你做渐变/阴影联动） */
.layout-with-sider-expanded {
	/* 例如：
	box-shadow: inset 1px 0 0 rgba(255,255,255,0.06);
	*/
}
</style>
