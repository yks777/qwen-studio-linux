(function() {
    'use strict';

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

        return true;
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

    function injectUpdateBanner(info) {
        if (document.querySelector('#qwen-update-banner')) return;

        const banner = document.createElement('div');
        banner.id = 'qwen-update-banner';
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
        banner.innerHTML = `
            <span>New version v${info.latest_version} is available</span>
            <div>
                <button id="qwen-banner-settings" style="
                    margin-right: 8px;
                    padding: 4px 12px;
                    background: rgba(255,255,255,0.2);
                    color: white;
                    border: 1px solid rgba(255,255,255,0.3);
                    border-radius: 4px;
                    cursor: pointer;
                ">View in Settings</button>
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

        document.body.prepend(banner);

        document.querySelector('#qwen-banner-dismiss')?.addEventListener('click', () => {
            banner.remove();
        });

        document.querySelector('#qwen-banner-settings')?.addEventListener('click', () => {
            banner.remove();
            window.location.href = '/settings';
        });
    }

    if (window.__TAURI__?.event) {
        window.__TAURI__.event.listen('event_from_main', (event) => {
            if (event.payload?.type === 'update-available') {
                injectUpdateBanner(event.payload.payload);
            }
        });
    }

    // Intercept SPA navigation instead of observing the whole DOM (perf).
    // A global MutationObserver with subtree:true fires on every DOM mutation of
    // the heavy chat.qwen.ai page, wasting CPU during load and use.
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

    console.log('[Qwen Studio] Settings injector loaded');
})();
