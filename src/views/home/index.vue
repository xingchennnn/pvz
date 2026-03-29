<template>
    <div class="home-view" :class="themeModeClass">
        <div class="home-view__backdrop"></div>

        <section class="home-top">
            <div class="home-left-stack">
                <article class="panel profile-card">
                    <div class="profile-card__avatar">
                        <span>PV</span>
                        <i></i>
                    </div>
                    <div class="profile-card__content">
                        <div class="profile-card__title">PVZ 控制台</div>
                        <div class="profile-card__subtitle">桌面工具与实时链路中心</div>
                        <div class="profile-card__tags">
                            <span v-for="tag in profileTags" :key="tag">{{ tag }}</span>
                        </div>
                    </div>
                </article>

                <article class="panel system-card">
                    <div class="panel-title">
                        <InfoCircleOutlined />
                        <span>系统信息</span>
                    </div>
                    <div class="system-card__list">
                        <div v-for="item in systemInfoItems" :key="item.label" class="system-card__item">
                            <div class="system-card__item-label">{{ item.label }}</div>
                            <div class="system-card__item-value">{{ item.value }}</div>
                        </div>
                    </div>
                </article>
            </div>

            <article class="panel overview-card">
                <div class="overview-card__main">
                    <div class="overview-card__section">
                        <div class="panel-title">
                            <DesktopOutlined />
                            <span>设备概览</span>
                        </div>
                        <div class="overview-grid">
                            <div v-for="item in deviceOverviewItems" :key="item.label" class="overview-grid__item">
                                <div class="overview-grid__label">{{ item.label }}</div>
                                <div class="overview-grid__value">{{ item.value }}</div>
                            </div>
                        </div>
                    </div>

                    <div class="overview-card__section overview-card__section--secondary">
                        <div class="panel-title panel-title--muted">
                            <RadarChartOutlined />
                            <span>运行摘要</span>
                        </div>
                        <div class="overview-grid overview-grid--secondary">
                            <div v-for="item in runtimeOverviewItems" :key="item.label" class="overview-grid__item">
                                <div class="overview-grid__label">{{ item.label }}</div>
                                <div class="overview-grid__value">{{ item.value }}</div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="overview-card__meters">
                    <div class="meter-card">
                        <div class="meter-ring" :style="dayProgressRingStyle">
                            <div class="meter-ring__inner">
                                <span>{{ dayProgress }}%</span>
                                <small>今日进度</small>
                            </div>
                        </div>
                    </div>

                    <div class="meter-card">
                        <div class="meter-ring" :style="shortcutCoverageRingStyle">
                            <div class="meter-ring__inner">
                                <span>{{ shortcutCoverage }}%</span>
                                <small>快捷覆盖</small>
                            </div>
                        </div>
                    </div>
                </div>
            </article>
        </section>

        <section class="metrics-row">
            <article v-for="item in summaryMetrics" :key="item.label" class="panel metric-card">
                <div class="metric-card__value">{{ item.value }}</div>
                <div class="metric-card__label">{{ item.label }}</div>
                <div class="metric-card__hint">{{ item.hint }}</div>
            </article>
        </section>

        <!-- <section class="home-bottom">
            <article class="panel quote-card">
                <div class="quote-card__badge">控制台摘要</div>
                <div class="quote-card__mark">“</div>
                <div class="quote-card__text">{{ quote.text }}</div>
                <div class="quote-card__author">{{ quote.author }}</div>
                <div class="quote-card__caption">{{ quote.caption }}</div>

                <div class="quote-card__actions">
                    <button
                        v-for="item in quickLinks"
                        :key="item.path"
                        class="action-chip"
                        type="button"
                        @click="goTo(item.path)"
                    >
                        <component :is="item.icon" />
                        <span>{{ item.label }}</span>
                        <small>{{ item.caption }}</small>
                    </button>
                </div>
            </article>
        </section> -->
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import {
    AudioOutlined,
    BookOutlined,
    ControlOutlined,
    DesktopOutlined,
    InfoCircleOutlined,
    RadarChartOutlined,
    SettingOutlined,
} from "@ant-design/icons-vue";
import { routes } from "@/routers";
import { useThemeStore } from "@/store/theme";

type PerformanceMemory = {
    usedJSHeapSize: number;
    jsHeapSizeLimit: number;
};

type PerformanceWithMemory = Performance & {
    memory?: PerformanceMemory;
};

type DashboardLink = {
    label: string;
    caption: string;
    path: string;
    icon: unknown;
};

const themeStore = useThemeStore();

const now = ref(new Date());
const heapUsedMb = ref<number | null>(null);
const heapLimitMb = ref<number | null>(null);
const viewport = ref({
    width: window.innerWidth,
    height: window.innerHeight,
});

let clockTimer: number | undefined;
let memoryTimer: number | undefined;

const quickLinks: DashboardLink[] = [
    {
        label: "降噪中心",
        caption: "实时音频链路",
        path: "/cpal",
        icon: AudioOutlined,
    },
    {
        label: "文章资料",
        caption: "记录与查阅",
        path: "/article",
        icon: BookOutlined,
    },
    {
        label: "窗口位置",
        caption: "桌面定位工具",
        path: "/windowPos",
        icon: ControlOutlined,
    },
    {
        label: "设置中心",
        caption: "主题与偏好",
        path: "/settings",
        icon: SettingOutlined,
    },
];

const visibleRoutes = computed(() => routes.filter((route) => !route.meta?.hideMenu));
const themeLabel = computed(() => (themeStore.isDarkMode ? "深色模式" : "浅色模式"));
const themeModeClass = computed(() =>
    themeStore.isDarkMode ? "home-view--dark" : "home-view--light",
);
const logicalThreads = computed(() => navigator.hardwareConcurrency || 0);
const routeCount = computed(() => visibleRoutes.value.length);
const shortcutCoverage = computed(() =>
    Math.round((quickLinks.length / Math.max(routeCount.value, 1)) * 100),
);

const themePalette = computed(() =>
    themeStore.isDarkMode
        ? {
              dayStart: "#ff4f9b",
              dayEnd: "#ff8f5c",
              shortcutStart: "#8e7dff",
              shortcutEnd: "#45d7ff",
              ringTrack: "rgba(255, 255, 255, 0.08)",
          }
        : {
              dayStart: "#ff6b7d",
              dayEnd: "#ffb14f",
              shortcutStart: "#4f6bff",
              shortcutEnd: "#2cb7ff",
              ringTrack: "rgba(24, 34, 54, 0.12)",
          },
);

const profileTags = computed(() => ["Rust", "Tauri", "实时音频", themeLabel.value]);

const dateLabel = computed(() => {
    const current = now.value;
    const weekNames = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
    return `${current.getFullYear()}/${String(current.getMonth() + 1).padStart(2, "0")}/${String(
        current.getDate(),
    ).padStart(2, "0")} ${weekNames[current.getDay()]}`;
});

const timeLabel = computed(() => {
    const current = now.value;
    return `${String(current.getHours()).padStart(2, "0")}:${String(current.getMinutes()).padStart(2, "0")}:${String(
        current.getSeconds(),
    ).padStart(2, "0")}`;
});

const dayProgress = computed(() => {
    const current = now.value;
    const minutes = current.getHours() * 60 + current.getMinutes();
    return Math.round((minutes / (24 * 60)) * 100);
});

const platformLabel = computed(() => {
    const ua = navigator.userAgent.toLowerCase();
    if (ua.includes("windows")) {
        return "Windows Desktop";
    }
    if (ua.includes("mac os")) {
        return "macOS Desktop";
    }
    if (ua.includes("linux")) {
        return "Linux Desktop";
    }
    return "Desktop Runtime";
});

const rendererLabel = computed(() => "Tauri 2 + Vue 3");

const viewportLabel = computed(
    () => `${viewport.value.width} × ${viewport.value.height}`,
);

const heapDisplay = computed(() => {
    if (heapUsedMb.value === null) {
        return "不可用";
    }
    if (heapLimitMb.value === null) {
        return `${heapUsedMb.value.toFixed(0)} MB`;
    }
    return `${heapUsedMb.value.toFixed(0)} / ${heapLimitMb.value.toFixed(0)} MB`;
});

const memoryUsagePercent = computed(() => {
    if (heapUsedMb.value === null || heapLimitMb.value === null || heapLimitMb.value <= 0) {
        return null;
    }
    return Math.round((heapUsedMb.value / heapLimitMb.value) * 100);
});

const systemInfoItems = computed(() => [
    { label: "客户端", value: "桌面控制台" },
    { label: "渲染器", value: rendererLabel.value },
    { label: "主题状态", value: themeLabel.value },
    { label: "运行模式", value: import.meta.env.PROD ? "production" : "development" },
    { label: "当前日期", value: dateLabel.value },
]);

const deviceOverviewItems = computed(() => [
    { label: "平台", value: platformLabel.value },
    { label: "逻辑线程", value: `${logicalThreads.value}` },
    { label: "视窗尺寸", value: viewportLabel.value },
    { label: "当前时间", value: timeLabel.value },
]);

const runtimeOverviewItems = computed(() => [
    { label: "可用页面", value: `${routeCount.value}` },
    { label: "快捷入口", value: `${quickLinks.length}` },
    { label: "JS 堆", value: heapDisplay.value },
    {
        label: "内存占用",
        value: memoryUsagePercent.value === null ? "不可用" : `${memoryUsagePercent.value}%`,
    },
]);

const summaryMetrics = computed(() => [
    {
        label: "页面总数",
        value: `${routeCount.value}`,
        hint: "主导航入口",
    },
    {
        label: "快捷入口",
        value: `${quickLinks.length}`,
        hint: "首页直达",
    },
    {
        label: "逻辑线程",
        value: `${logicalThreads.value}`,
        hint: "浏览器环境",
    },
    {
        label: "JS 堆占用",
        value: heapUsedMb.value === null ? "--" : `${heapUsedMb.value.toFixed(0)}`,
        hint: "单位 MB",
    },
    {
        label: "今日进度",
        value: `${dayProgress.value}%`,
        hint: "时间刻度",
    },
]);

const dayProgressRingStyle = computed(() => ({
    background: `conic-gradient(from 210deg, ${themePalette.value.dayStart} 0%, ${themePalette.value.dayEnd} ${dayProgress.value}%, ${themePalette.value.ringTrack} ${dayProgress.value}%, ${themePalette.value.ringTrack} 100%)`,
}));

const shortcutCoverageRingStyle = computed(() => ({
    background: `conic-gradient(from 210deg, ${themePalette.value.shortcutStart} 0%, ${themePalette.value.shortcutEnd} ${shortcutCoverage.value}%, ${themePalette.value.ringTrack} ${shortcutCoverage.value}%, ${themePalette.value.ringTrack} 100%)`,
}));

const refreshMemory = () => {
    const memory = (performance as PerformanceWithMemory).memory;
    if (!memory) {
        heapUsedMb.value = null;
        heapLimitMb.value = null;
        return;
    }

    heapUsedMb.value = memory.usedJSHeapSize / 1024 / 1024;
    heapLimitMb.value = memory.jsHeapSizeLimit / 1024 / 1024;
};

const handleResize = () => {
    viewport.value = {
        width: window.innerWidth,
        height: window.innerHeight,
    };
};

onMounted(() => {
    refreshMemory();
    handleResize();
    clockTimer = window.setInterval(() => {
        now.value = new Date();
    }, 1000);
    memoryTimer = window.setInterval(() => {
        refreshMemory();
    }, 4000);
    window.addEventListener("resize", handleResize);
});

onUnmounted(() => {
    if (clockTimer) {
        window.clearInterval(clockTimer);
    }
    if (memoryTimer) {
        window.clearInterval(memoryTimer);
    }
    window.removeEventListener("resize", handleResize);
});
</script>

<style scoped>
.home-view {
    --home-shell-shadow: 0 22px 48px rgba(7, 12, 21, 0.24);
    --home-grid-line-1: rgba(255, 255, 255, 0.035);
    --home-grid-line-2: rgba(255, 255, 255, 0.03);
    --home-panel-bg: rgba(11, 18, 29, 0.82);
    --home-panel-border: rgba(194, 210, 255, 0.08);
    --home-panel-top-highlight: rgba(255, 255, 255, 0.04);
    --home-panel-glow: rgba(255, 94, 143, 0.2);
    --home-title: rgba(239, 244, 255, 0.94);
    --home-title-muted: rgba(207, 217, 255, 0.82);
    --home-text-strong: #f3f6ff;
    --home-text-primary: #eff4ff;
    --home-text-secondary: rgba(198, 210, 230, 0.72);
    --home-text-tertiary: rgba(184, 197, 223, 0.66);
    --home-text-soft: rgba(171, 185, 212, 0.56);
    --home-tag-bg: rgba(255, 255, 255, 0.05);
    --home-tag-border: rgba(255, 255, 255, 0.08);
    --home-item-bg: rgba(255, 255, 255, 0.04);
    --home-meter-inner-bg: rgba(11, 18, 29, 0.92);
    --home-meter-ring-border: rgba(255, 255, 255, 0.06);
    --home-meter-inner-border: rgba(255, 255, 255, 0.05);
    --home-accent: #ff6e9f;
    --home-accent-soft: rgba(255, 78, 157, 0.58);
    --home-chip-hover-border: rgba(143, 128, 255, 0.42);
    --home-chip-hover-bg: rgba(127, 113, 255, 0.12);
    position: relative;
    min-height: calc(100vh - 92px);
    overflow: hidden;
    border-radius: 22px;
    padding: 14px;
    color: var(--home-text-primary);
    box-shadow: inset 0 1px 0 var(--home-panel-top-highlight), var(--home-shell-shadow);
    transition: background 0.28s ease, color 0.28s ease, box-shadow 0.28s ease;
}

.home-view--dark {
    background:
        radial-gradient(circle at top left, rgba(255, 146, 109, 0.16), transparent 26%),
        radial-gradient(circle at top right, rgba(120, 119, 255, 0.16), transparent 24%),
        linear-gradient(180deg, #131b29 0%, #111826 50%, #0e1622 100%);
}

.home-view--light {
    --home-shell-shadow: 0 22px 48px rgba(86, 103, 139, 0.18);
    --home-grid-line-1: rgba(56, 78, 117, 0.08);
    --home-grid-line-2: rgba(56, 78, 117, 0.05);
    --home-panel-bg: rgba(255, 255, 255, 0.76);
    --home-panel-border: rgba(89, 113, 154, 0.14);
    --home-panel-top-highlight: rgba(255, 255, 255, 0.72);
    --home-panel-glow: rgba(255, 124, 142, 0.18);
    --home-title: #243043;
    --home-title-muted: rgba(52, 68, 95, 0.82);
    --home-text-strong: #1d293a;
    --home-text-primary: #263246;
    --home-text-secondary: rgba(77, 92, 120, 0.82);
    --home-text-tertiary: rgba(93, 107, 132, 0.78);
    --home-text-soft: rgba(102, 114, 136, 0.66);
    --home-tag-bg: rgba(59, 83, 120, 0.08);
    --home-tag-border: rgba(89, 113, 154, 0.14);
    --home-item-bg: rgba(250, 252, 255, 0.82);
    --home-meter-inner-bg: rgba(255, 255, 255, 0.94);
    --home-meter-ring-border: rgba(89, 113, 154, 0.16);
    --home-meter-inner-border: rgba(108, 129, 166, 0.14);
    --home-accent: #ff6a87;
    --home-accent-soft: rgba(255, 98, 132, 0.46);
    --home-chip-hover-border: rgba(79, 107, 255, 0.32);
    --home-chip-hover-bg: rgba(79, 107, 255, 0.1);
    background:
        radial-gradient(circle at top left, rgba(255, 166, 119, 0.28), transparent 28%),
        radial-gradient(circle at top right, rgba(97, 191, 255, 0.22), transparent 26%),
        linear-gradient(180deg, #f8fbff 0%, #edf3fb 52%, #e7eef8 100%);
}

.home-view__backdrop {
    position: absolute;
    inset: 0;
    background-image:
        linear-gradient(var(--home-grid-line-1) 1px, transparent 1px),
        linear-gradient(90deg, var(--home-grid-line-2) 1px, transparent 1px);
    background-size: 32px 32px;
    mask-image: linear-gradient(180deg, rgba(0, 0, 0, 0.75), transparent 100%);
    pointer-events: none;
}

.home-top,
.metrics-row,
.home-bottom {
    position: relative;
    z-index: 1;
}

.home-top {
    display: grid;
    gap: 14px;
    grid-template-columns: minmax(250px, 272px) minmax(0, 1fr);
}

.home-left-stack {
    display: grid;
    gap: 14px;
}

.panel {
    position: relative;
    overflow: hidden;
    border: 1px solid var(--home-panel-border);
    border-radius: 18px;
    background: var(--home-panel-bg);
    box-shadow: inset 0 1px 0 var(--home-panel-top-highlight);
    backdrop-filter: blur(14px);
    transition: background 0.28s ease, border-color 0.28s ease, box-shadow 0.28s ease;
}

.panel::after {
    position: absolute;
    inset: auto -40px -70px auto;
    width: 180px;
    height: 180px;
    border-radius: 50%;
    background: radial-gradient(circle, var(--home-panel-glow), transparent 70%);
    content: "";
    pointer-events: none;
}

.panel-title {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: var(--home-title);
    font-size: 15px;
    font-weight: 700;
    letter-spacing: 0.02em;
}

.panel-title--muted {
    color: var(--home-title-muted);
}

.profile-card {
    display: flex;
    gap: 14px;
    align-items: center;
    min-height: 96px;
    padding: 16px;
}

.profile-card__avatar {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 60px;
    height: 60px;
    border-radius: 18px;
    background:
        linear-gradient(140deg, rgba(125, 111, 255, 0.95), rgba(255, 120, 135, 0.95));
    color: #fff;
    font-family: "DIN Alternate", "Bahnschrift", "Segoe UI", sans-serif;
    font-size: 24px;
    font-weight: 700;
    box-shadow: 0 14px 30px rgba(96, 92, 230, 0.28);
}

.profile-card__avatar i {
    position: absolute;
    right: -2px;
    bottom: -2px;
    width: 14px;
    height: 14px;
    border: 2px solid #101827;
    border-radius: 50%;
    background: #4ade80;
}

.profile-card__content {
    min-width: 0;
}

.profile-card__title {
    font-family: "Bahnschrift", "Segoe UI Variable Display", "PingFang SC", sans-serif;
    font-size: 24px;
    font-weight: 700;
    letter-spacing: 0.02em;
}

.profile-card__subtitle {
    margin-top: 4px;
    color: var(--home-text-secondary);
    font-size: 12px;
}

.profile-card__tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 10px;
}

.profile-card__tags span {
    padding: 5px 9px;
    border: 1px solid var(--home-tag-border);
    border-radius: 999px;
    background: var(--home-tag-bg);
    color: var(--home-title-muted);
    font-size: 11px;
    line-height: 1;
}

.system-card {
    padding: 16px 16px 12px;
}

.system-card__list {
    display: grid;
    gap: 10px;
    margin-top: 14px;
}

.system-card__item {
    padding: 10px 12px;
    border-radius: 14px;
    background: var(--home-item-bg);
}

.system-card__item-label {
    color: var(--home-text-tertiary);
    font-size: 12px;
    letter-spacing: 0.03em;
}

.system-card__item-value {
    margin-top: 6px;
    color: var(--home-text-strong);
    font-size: 13px;
    font-weight: 600;
    line-height: 1.45;
    word-break: break-word;
}

.overview-card {
    display: grid;
    gap: 14px;
    grid-template-columns: minmax(0, 1fr) 236px;
    padding: 18px;
}

.overview-card__main {
    display: grid;
    gap: 18px;
}

.overview-card__section {
    display: grid;
    gap: 12px;
}

.overview-card__section--secondary {
    padding-top: 2px;
    border-top: 1px solid var(--home-tag-border);
}

.overview-grid {
    display: grid;
    gap: 10px;
    grid-template-columns: repeat(2, minmax(0, 1fr));
}

.overview-grid--secondary {
    gap: 10px;
}

.overview-grid__item {
    padding: 12px 14px;
    border-radius: 14px;
    background: var(--home-item-bg);
}

.overview-grid__label {
    color: var(--home-text-tertiary);
    font-size: 12px;
}

.overview-grid__value {
    margin-top: 8px;
    font-family: "Bahnschrift", "DIN Alternate", "Segoe UI", sans-serif;
    font-size: 17px;
    font-weight: 600;
    letter-spacing: 0.02em;
}

.overview-card__meters {
    display: grid;
    gap: 14px;
    align-content: center;
}

.meter-card {
    display: flex;
    justify-content: center;
}

.meter-ring {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 156px;
    height: 156px;
    padding: 14px;
    border-radius: 50%;
    box-shadow: inset 0 0 0 1px var(--home-meter-ring-border);
}

.meter-ring::before {
    position: absolute;
    inset: 16px;
    border-radius: 50%;
    background: var(--home-meter-inner-bg);
    box-shadow: inset 0 0 0 1px var(--home-meter-inner-border);
    content: "";
}

.meter-ring__inner {
    position: relative;
    z-index: 1;
    display: grid;
    justify-items: center;
    gap: 6px;
    text-align: center;
}

.meter-ring__inner span {
    font-family: "DIN Alternate", "Bahnschrift", sans-serif;
    font-size: 30px;
    font-weight: 700;
}

.meter-ring__inner small {
    color: var(--home-text-secondary);
    font-size: 11px;
    letter-spacing: 0.04em;
}

.metrics-row {
    display: grid;
    gap: 12px;
    grid-template-columns: repeat(5, minmax(0, 1fr));
    margin-top: 14px;
}

.metric-card {
    padding: 14px 10px;
    text-align: center;
}

.metric-card__value {
    font-family: "DIN Alternate", "Bahnschrift", sans-serif;
    font-size: 30px;
    font-weight: 700;
    line-height: 1;
}

.metric-card__label {
    margin-top: 8px;
    color: var(--home-title);
    font-size: 13px;
    font-weight: 600;
}

.metric-card__hint {
    margin-top: 4px;
    color: var(--home-text-soft);
    font-size: 11px;
}

.home-bottom {
    margin-top: 14px;
}

.quote-card {
    min-height: 216px;
    padding: 20px 20px 18px;
    text-align: center;
}

.quote-card__badge {
    display: inline-flex;
    padding: 6px 12px;
    border-radius: 999px;
    background: var(--home-tag-bg);
    color: var(--home-text-secondary);
    font-size: 12px;
    letter-spacing: 0.08em;
}

.quote-card__mark {
    margin-top: 14px;
    color: var(--home-accent-soft);
    font-size: 48px;
    line-height: 1;
    font-weight: 700;
}

.quote-card__text {
    max-width: 620px;
    margin: 6px auto 0;
    font-family: "Bahnschrift", "Segoe UI Variable Display", "PingFang SC", sans-serif;
    font-size: clamp(20px, 2.4vw, 32px);
    font-weight: 700;
    line-height: 1.35;
    letter-spacing: 0.02em;
}

.quote-card__author {
    margin-top: 10px;
    color: var(--home-accent);
    font-size: 16px;
    font-weight: 700;
}

.quote-card__caption {
    margin-top: 4px;
    color: var(--home-text-tertiary);
    font-size: 12px;
}

.quote-card__actions {
    display: grid;
    gap: 10px;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    margin-top: 18px;
}

.action-chip {
    display: grid;
    gap: 4px;
    justify-items: start;
    padding: 12px 14px;
    border: 1px solid var(--home-tag-border);
    border-radius: 14px;
    background: var(--home-item-bg);
    color: var(--home-text-strong);
    text-align: left;
    transition: transform 0.18s ease, border-color 0.18s ease, background 0.18s ease;
    cursor: pointer;
}

.action-chip:hover {
    transform: translateY(-2px);
    border-color: var(--home-chip-hover-border);
    background: var(--home-chip-hover-bg);
}

.action-chip :deep(svg) {
    font-size: 16px;
}

.action-chip span {
    font-size: 13px;
    font-weight: 700;
}

.action-chip small {
    color: var(--home-text-tertiary);
    font-size: 11px;
}

@media (max-width: 1220px) {
    .home-top {
        grid-template-columns: 1fr;
    }

    .home-left-stack {
        grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .metrics-row {
        grid-template-columns: repeat(3, minmax(0, 1fr));
    }
}

@media (max-width: 980px) {
    .overview-card {
        grid-template-columns: 1fr;
    }

    .overview-card__meters {
        grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .quote-card__actions {
        grid-template-columns: repeat(2, minmax(0, 1fr));
    }
}

@media (max-width: 720px) {
    .home-view {
        min-height: auto;
        padding: 12px;
        border-radius: 18px;
    }

    .home-left-stack,
    .metrics-row,
    .overview-grid,
    .overview-card__meters,
    .quote-card__actions {
        grid-template-columns: 1fr;
    }

    .profile-card {
        align-items: flex-start;
    }

    .profile-card__title {
        font-size: 22px;
    }

    .metric-card__value {
        font-size: 28px;
    }
}
</style>