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

    console.log('[Qwem Studio] Bridge initialized');
})();
