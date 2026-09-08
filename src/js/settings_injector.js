(function() {
    'use strict';

    let updateInProgress = false;
    let dismissedVersion = null;
    let installedShown = false;

    function api() {
        return window.electronAPI;
    }

    function injectUpdatesTab() {
        const settingsContainer = document.querySelector('[data-testid="settings-container"]') ||
                                   document.querySelector('.settings-container') ||
                                   document.querySelector('#settings');

        if (!settingsContainer) return false;

        if (document.querySelector('#qwen-updates-tab')) return false;

        const tab = document.createElement('div');
        tab.id = 'qwen-updates-tab';
        tab.innerHTML = `
            <div style="padding: 20px; border-top: 1px solid var(--border-color);">
                <h3 style="margin-bottom: 16px;">Updates</h3>
                <div id="qwen-update-status" style="margin-bottom: 16px;">
                    <p>Checking for updates...</p>
                </div>
                <button id="qwen-check-update" style="
                    padding: 8px 16px;
                    background: var(--primary-color, #007bff);
                    color: white;
                    border: none;
                    border-radius: 4px;
                    cursor: pointer;
                ">Check for Updates</button>
                <div id="qwen-update-progress" style="display: none; margin-top: 16px;">
                    <div style="
                        height: 4px;
                        background: var(--bg-secondary, #333);
                        border-radius: 2px;
                        overflow: hidden;
                    ">
                        <div id="qwen-progress-bar" style="
                            height: 100%;
                            background: var(--primary-color, #007bff);
                            width: 0%;
                            transition: width 0.3s;
                        "></div>
                    </div>
                    <p id="qwen-progress-text" style="margin-top: 8px; font-size: 12px; color: var(--text-secondary);"></p>
                </div>
            </div>
        `;

        settingsContainer.appendChild(tab);

        document.querySelector('#qwen-check-update')?.addEventListener('click', async () => {
            const status = document.querySelector('#qwen-update-status');
            if (status) status.innerHTML = '<p>Checking for updates...</p>';
            try {
                const info = await api().check_for_updates(false);
                renderStatus(info);
                if (info && info.available && info.download_url && !updateInProgress) {
                    startUpdate(info);
                }
            } catch (e) {
                if (status) status.innerHTML = '<p>Failed to check for updates.</p>';
            }
        });

        api().get_update_info?.()
            .then((info) => { if (info) renderStatus(info); })
            .catch(() => {});

        return true;
    }

    function renderStatus(info) {
        const status = document.querySelector('#qwen-update-status');
        if (!status || !info) return;
        if (info.available) {
            status.innerHTML = `<p>A new version (v${info.latest_version}) is available. ` +
                `Downloading and installing automatically...</p>`;
        } else {
            status.innerHTML = `<p>You are on the latest version (v${info.current_version}).</p>`;
        }
    }

    function tryInjectWithRetry(attempt) {
        if (injectUpdatesTab()) return;
        if (attempt >= 20) return;
        setTimeout(() => tryInjectWithRetry(attempt + 1), 100);
    }

    function maybeInject() {
        if (window.location.pathname.includes('settings')) {
            tryInjectWithRetry(0);
        }
    }

    function ensureBanner(id) {
        let banner = document.querySelector('#' + id);
        if (banner) return banner;
        banner = document.createElement('div');
        banner.id = id;
        banner.style.cssText = `
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            background: var(--primary-color, #007bff);
            color: white;
            padding: 12px 20px;
            display: flex;
            align-items: center;
            justify-content: space-between;
            z-index: 99999;
            font-size: 14px;
        `;
        document.body.prepend(banner);
        return banner;
    }

    function showAvailableBanner(info) {
        if (document.querySelector('#qwen-update-banner')) return;

        const banner = ensureBanner('qwen-update-banner');
        banner.innerHTML = `
            <span>New version v${info.latest_version} is available</span>
            <div>
                <button id="qwen-banner-update" style="
                    margin-right: 8px;
                    padding: 4px 12px;
                    background: rgba(255,255,255,0.2);
                    color: white;
                    border: 1px solid rgba(255,255,255,0.3);
                    border-radius: 4px;
                    cursor: pointer;
                ">Update now</button>
                <button id="qwen-banner-dismiss" style="
                    padding: 4px 12px;
                    background: rgba(255,255,255,0.1);
                    color: white;
                    border: 1px solid rgba(255,255,255,0.2);
                    border-radius: 4px;
                    cursor: pointer;
                ">Dismiss</button>
            </div>
        `;

        banner.querySelector('#qwen-banner-dismiss')?.addEventListener('click', () => {
            dismissedVersion = info.latest_version;
            banner.remove();
        });

        banner.querySelector('#qwen-banner-update')?.addEventListener('click', () => {
            banner.remove();
            startUpdate(info);
        });
    }

    function showProgressBanner() {
        const banner = ensureBanner('qwen-update-banner');
        banner.innerHTML = `
            <span id="qwen-progress-label">Downloading update...</span>
            <div style="
                flex: 1;
                margin: 0 16px;
                height: 6px;
                background: rgba(255,255,255,0.25);
                border-radius: 3px;
                overflow: hidden;
            ">
                <div id="qwen-banner-progress" style="
                    height: 100%;
                    width: 0%;
                    background: white;
                    transition: width 0.3s;
                "></div>
            </div>
            <span id="qwen-progress-pct" style="min-width: 40px; text-align: right;">0%</span>
        `;
    }

    function updateProgress(progress, downloaded, total) {
        let bar = document.querySelector('#qwen-banner-progress');
        let tabBar = document.querySelector('#qwen-progress-bar');
        if (!bar && !tabBar) {
            showProgressBanner();
            bar = document.querySelector('#qwen-banner-progress');
        }
        const pct = document.querySelector('#qwen-progress-pct');
        const label = document.querySelector('#qwen-progress-label');
        const tabText = document.querySelector('#qwen-progress-text');

        const p = Math.max(0, Math.min(100, progress || 0));
        if (bar) bar.style.width = p + '%';
        if (tabBar) tabBar.style.width = p + '%';
        if (pct) pct.textContent = p + '%';
        if (label) label.textContent = 'Downloading update...';
        if (tabText) {
            const mb = (downloaded || 0) / (1024 * 1024);
            const totalMb = (total || 0) / (1024 * 1024);
            tabText.textContent = totalMb > 0
                ? `${p}% (${mb.toFixed(1)} / ${totalMb.toFixed(1)} MB)`
                : `${p}%`;
        }
    }

    function showInstalledBanner() {
        if (installedShown) return;
        installedShown = true;
        const banner = ensureBanner('qwen-update-banner');
        banner.innerHTML = `
            <span id="qwen-restart-text">Update installed. Restarting in 60s...</span>
            <button id="qwen-restart-now" style="
                margin-left: 16px;
                padding: 4px 12px;
                background: rgba(255,255,255,0.2);
                color: white;
                border: 1px solid rgba(255,255,255,0.3);
                border-radius: 4px;
                cursor: pointer;
            ">Restart now</button>
        `;

        let remaining = 60;
        const txt = banner.querySelector('#qwen-restart-text');
        const timer = setInterval(() => {
            remaining--;
            if (txt) txt.textContent = `Update installed. Restarting in ${remaining}s...`;
            if (remaining <= 0) {
                clearInterval(timer);
                doRestart();
            }
        }, 1000);

        banner.querySelector('#qwen-restart-now')?.addEventListener('click', () => {
            clearInterval(timer);
            doRestart();
        });
    }

    function doRestart() {
        try {
            api().restart_app();
        } catch (e) { /* app will restart regardless */ }
    }

    async function startUpdate(info) {
        if (updateInProgress) return;
        if (!info || !info.download_url) {
            const banner = ensureBanner('qwen-update-banner');
            banner.innerHTML = '<span>Update available but download URL is missing.</span>';
            return;
        }

        updateInProgress = true;
        showProgressBanner();
        const progressEl = document.querySelector('#qwen-update-progress');
        if (progressEl) progressEl.style.display = 'block';

        try {
            const result = await api().install_update_with_progress(info.download_url);
            if (result === 'already-updating') return; // another instance is handling it
            updateProgress(100, 0, 0);
            showInstalledBanner();
        } catch (e) {
            updateInProgress = false;
            const banner = ensureBanner('qwen-update-banner');
            banner.innerHTML = `<span>Update failed: ${(e && e.message) || e}</span>`;
        }
    }

    if (window.__TAURI__?.event) {
        window.__TAURI__.event.listen('event_from_main', (event) => {
            const payload = event.payload;
            if (!payload || !payload.type) return;

            if (payload.type === 'update-available') {
                const info = payload.payload;
                if (info && info.available && info.latest_version !== dismissedVersion) {
                    showAvailableBanner(info);
                    // Auto-install is triggered from Rust (single source) to avoid
                    // parallel downloads across the multiple injected webviews.
                }
            } else if (payload.type === 'update-progress') {
                const p = payload.payload || {};
                updateProgress(p.progress, p.downloaded, p.total);
            } else if (payload.type === 'update-installed') {
                showInstalledBanner();
            }
        });
    }

    // Intercept SPA navigation instead of observing the whole DOM (perf).
    const _pushState = history.pushState;
    history.pushState = function() {
        const r = _pushState.apply(this, arguments);
        maybeInject();
        return r;
    };
    const _replaceState = history.replaceState;
    history.replaceState = function() {
        const r = _replaceState.apply(this, arguments);
        maybeInject();
        return r;
    };
    window.addEventListener('popstate', maybeInject);
    window.addEventListener('hashchange', maybeInject);

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', maybeInject);
    } else {
        maybeInject();
    }

    if(window.__QWEN_DEBUG) console.log('[Qwen Studio] Settings injector loaded');
})();
