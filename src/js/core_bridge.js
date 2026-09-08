(function() {
    'use strict';

    const invoke = window.__TAURI__?.core?.invoke;

    if (!invoke) {
        console.warn('[Qwem Studio] Tauri invoke not available');
        return;
    }

    const eventListeners = {};

    window.__TAURI__?.event?.listen('event_from_main', (event) => {
        const data = event.payload;
        const type = data?.type;
        const payload = data?.payload;
        if (type && eventListeners[type]) {
            eventListeners[type].forEach(cb => cb(payload));
        }
    });

    window.electronAPI = {
        PRELOAD_FILE_PATH: '',
        minimize: () => invoke('minimize_window'),
        maximize: () => invoke('maximize_window'),
        close: () => invoke('close_window'),
        open_devtool: () => invoke('open_devtool'),
        toggle_hidden_devtools: () => invoke('toggle_hidden_devtools'),
        get_app_version: () => invoke('get_app_version'),
        get_platform_info: () => invoke('get_platform_info'),
        open_external_link: (url) => invoke('open_external_link', { url }),
        switch_theme: (theme) => invoke('switch_theme', { theme }),
        switch_ln: (lang) => invoke('switch_ln', { ln: lang }),
        update_title_bar_for_system_theme: (isDark) => invoke('update_title_bar_for_system_theme', { isDark }),
        handle_shortcut: (action) => invoke('handle_shortcut', { action }),
        export_chat: (title, messages, format) => invoke('export_chat', { title, messages, format }),
        get_setting: (key) => invoke('get_setting', { key }),
        set_setting: (key, value) => invoke('set_setting', { key, value }),
        show_native_dialog: (options) => invoke('show_native_dialog', { options }),
        request_file_access: (purpose, returnFile) => invoke('request_file_access', { purpose, returnFile }),
        read_clipboard_image: () => invoke('read_clipboard_image'),
        on_event: function(type, callback) {
            if (!eventListeners[type]) eventListeners[type] = [];
            eventListeners[type].push(callback);
        },
        send_event: function(data) {
            window.__TAURI__?.event?.emit('event_to_main', data);
        },
        mcp_client_connect: () => invoke('mcp_client_connect'),
        mcp_client_close: () => invoke('mcp_client_close'),
        mcp_client_tool_list: (serverName) => invoke('mcp_client_tool_list', { params: { serverName } }),
        mcp_client_tool_call: (params) => invoke('mcp_client_tool_call', { params }),
        mcp_client_get_config: () => invoke('mcp_client_get_config'),
        mcp_client_update_config: (config) => invoke('mcp_client_update_config', { config }),
        check_for_updates: (silent) => invoke('check_for_updates', { silent: silent ?? false }),
        install_update_with_progress: (url) => invoke('install_update_with_progress', { url }),
        restart_app: () => invoke('restart_app'),
        get_update_info: () => invoke('get_update_info'),
        get_language: () => invoke('get_language'),
        list_crash_logs: () => invoke('list_crash_logs'),
        read_crash_log: (filename) => invoke('read_crash_log', { filename }),
    };

    window.electron = {
        ipcRenderer: {
            on: (channel, callback) => {
                window.__TAURI__.event.listen('event_from_main', (event) => {
                    if (event.payload?.type === channel) {
                        callback(event.payload.payload);
                    }
                });
            },
            send: (channel, data) => {
                window.__TAURI__.event.emit('event_to_main', { type: channel, payload: data });
            },
            invoke: (command, ...args) => invoke(command, ...args),
        },
    };

    // Best-effort Electron clipboard shim so chat.qwen.ai's own Electron-API
    // calls resolve. `read_clipboard_image` returns base64 PNG, which we wrap in
    // a minimal NativeImage-like object (full emulation is impossible in JS).
    function __qwenNativeImage(b64) {
        return {
            toDataURL: () => 'data:image/png;base64,' + b64,
            toPNG:     () => b64,
            toBitmap:  () => b64,
            isEmpty:   () => !b64,
        };
    }
    window.electron.clipboard = {
        readText:  () => invoke('plugin:clipboard-manager|read_text'),
        writeText: (t) => invoke('plugin:clipboard-manager|write_text', { text: t }),
        readImage: async () => __qwenNativeImage(await invoke('read_clipboard_image')),
        writeImage: () => Promise.resolve(),
        read: async () => {
            try { return await navigator.clipboard.read(); }
            catch (_) {
                const items = [];
                try {
                    const t = await invoke('plugin:clipboard-manager|read_text');
                    if (t) items.push(new ClipboardItem({ 'text/plain': new Blob([t], { type: 'text/plain' }) }));
                } catch (_) {}
                try {
                    const b = await invoke('read_clipboard_image');
                    if (b) items.push(new ClipboardItem({ 'image/png': await (await fetch('data:image/png;base64,' + b)).blob() }));
                } catch (_) {}
                return items;
            }
        },
    };

    if(window.__QWEN_DEBUG) console.log('[Qwem Studio] Bridge initialized');
})();
