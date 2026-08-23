(function() {
    'use strict';

    let currentZoom = 1.0;
    const MIN_ZOOM = 0.5;
    const MAX_ZOOM = 2.0;
    const ZOOM_STEP = 0.1;

    function applyZoom(scale) {
        currentZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, scale));
        document.body.style.zoom = currentZoom;
    }

    document.addEventListener('wheel', (e) => {
        if (e.ctrlKey) {
            e.preventDefault();
            const delta = e.deltaY > 0 ? -ZOOM_STEP : ZOOM_STEP;
            applyZoom(currentZoom + delta);
        }
    }, { passive: false });

    document.addEventListener('keydown', (e) => {
        if (e.ctrlKey) {
            if (e.key === '=' || e.key === '+') {
                e.preventDefault();
                applyZoom(currentZoom + ZOOM_STEP);
            } else if (e.key === '-') {
                e.preventDefault();
                applyZoom(currentZoom - ZOOM_STEP);
            } else if (e.key === '0') {
                e.preventDefault();
                applyZoom(1.0);
            }
        }
    });

    let dragCounter = 0;

    document.addEventListener('dragenter', (e) => {
        e.preventDefault();
        dragCounter++;
    });

    document.addEventListener('dragleave', (e) => {
        e.preventDefault();
        dragCounter--;
        if (dragCounter === 0) {
            const zone = document.querySelector('#dropzone-container');
            if (zone) zone.style.display = 'none';
        }
    });

    document.addEventListener('dragover', (e) => {
        e.preventDefault();
    });

    document.addEventListener('drop', async (e) => {
        e.preventDefault();
        dragCounter = 0;

        const zone = document.querySelector('#dropzone-container');
        if (zone) zone.style.display = 'none';

        if (!e.dataTransfer?.files?.length) return;

        const files = [];
        for (const file of e.dataTransfer.files) {
            try {
                const arrayBuffer = await file.arrayBuffer();
                const uint8Array = new Uint8Array(arrayBuffer);
                const base64 = btoa(String.fromCharCode.apply(null, uint8Array));
                files.push({
                    name: file.name,
                    type: file.type,
                    size: file.size,
                    data: base64,
                });
            } catch (err) {
                console.error('[Qwem Studio] File read error:', err);
            }
        }

        if (files.length > 0) {
            window.electronAPI?.handleFiles?.(files) ||
                window.__TAURI__?.event?.emit('files-dropped', { files });
        }
    });

    const originalPaste = document.addEventListener;

    document.addEventListener('paste', async (e) => {
        if (e.clipboardData?.items) {
            for (const item of e.clipboardData.items) {
                if (item.type.startsWith('image/')) {
                    e.preventDefault();
                    try {
                        const blob = item.getAsFile();
                        const arrayBuffer = await blob.arrayBuffer();
                        const uint8Array = new Uint8Array(arrayBuffer);
                        const base64 = btoa(String.fromCharCode.apply(null, uint8Array));
                        window.__TAURI__?.event?.emit('clipboard-image-pasted', {
                            data: base64,
                            type: blob.type,
                        });
                    } catch (err) {
                        console.error('[Qwem Studio] Paste image error:', err);
                    }
                    return;
                }
            }
        }
    }, true);

    window.open = function(url, target, features) {
        if (url && (url.startsWith('http://') || url.startsWith('https://'))) {
            window.electronAPI?.open_external_link?.(url);
        }
        return null;
    };

    console.log('[Qwem Studio] Platform bridge loaded');
})();
