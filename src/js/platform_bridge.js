(function() {
    'use strict';

    let currentZoom = 1.0;
    const MIN_ZOOM = 0.5;
    const MAX_ZOOM = 2.0;
    const ZOOM_STEP = 0.1;

    // Guardião de reentrância para o listener de paste em CAPTURE phase.
    // Impede loops infinitos quando a injeção de imagem dispara eventos
    // sintéticos (change/input) que poderiam re-triggerar o listener.
    let __pasteInProgress = false;

    // Zoom: persist per-profile via localStorage + respect reduced-motion/low-GPU
    try {
        const saved = parseFloat(localStorage.getItem('__qwen_zoom') || '');
        if (!isNaN(saved)) currentZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, saved));
    } catch(_) {}
    const prefersReducedMotion = window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    function applyZoom(scale) {
        currentZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, scale));
        // Use zoom (layout) not transform — WebKit handles zoom natively with less GPU layers; disable transition on low-GPU
        document.documentElement.style.zoom = String(currentZoom);
        document.body.style.zoom = String(currentZoom);
        try { localStorage.setItem('__qwen_zoom', String(currentZoom)); } catch(_) {}
        // Also persist to Rust settings when available (for multi-profile sync)
        try { window.__TAURI__?.core?.invoke?.('set_setting', { key: 'zoom', value: currentZoom }); } catch(_) {}
        if (prefersReducedMotion) {
            document.documentElement.style.transition = 'none';
            document.body.style.transition = 'none';
        }
    }
    // Apply saved zoom on load
    if (currentZoom !== 1.0) applyZoom(currentZoom);

    // --- A: global document capture blocker for find (installed early, before SPA listeners) ---
    // Must be before any SPA registers its own document capture listener.
    // Checks if find bar is open+focused and blocks the event before site sees it.
    (function installEarlyFindBlocker() {
        function isFindActive() {
            const bar = document.getElementById('__qwen-find-bar');
            const inp = document.getElementById('__qwen-find-input');
            return !!(bar && bar.style.display !== 'none' && inp && document.activeElement === inp);
        }
        const earlyBlocker = (e) => {
            if (!isFindActive()) return;
            // Let Esc/F3/Enter through to our own handlers, block everything else
            if (e.key === 'Escape' || e.key === 'F3' || e.key === 'Enter') return;
            // Stop site's document capture listener before it refocuses composer
            e.stopImmediatePropagation();
        };
        document.addEventListener('keydown', earlyBlocker, true);
        document.addEventListener('keypress', earlyBlocker, true);
        document.addEventListener('beforeinput', earlyBlocker, true);
        // Also block 'input' capture if site listens there
        document.addEventListener('input', (e) => {
            if (isFindActive() && e.target && e.target.id === '__qwen-find-input') {
                e.stopImmediatePropagation();
            }
        }, true);
    })();

    // --- B: monkey-patch addEventListener to wrap future document/window keydown listeners ---
    // If SPA registers its listener AFTER our early blocker, our blocker (registered first)
    // still wins by registration order. But to be safe against late registrations
    // that might use a different phase/order, wrap them to check find state first.
    (function patchAddEventListenerForFind() {
        const origAdd = EventTarget.prototype.addEventListener;
        EventTarget.prototype.addEventListener = function(type, listener, options) {
            if ((type === 'keydown' || type === 'keypress' || type === 'beforeinput')
                && (this === document || this === window || this === document.body || this === document.documentElement)
                && typeof listener === 'function') {
                const wrapped = function(e) {
                    const bar = document.getElementById('__qwen-find-bar');
                    const inp = document.getElementById('__qwen-find-input');
                    const isFindActive = !!(bar && bar.style.display !== 'none' && inp && document.activeElement === inp);
                    if (isFindActive && e.key !== 'Escape' && e.key !== 'F3' && e.key !== 'Enter') {
                        // Don't let site handler run while typing in find
                        return;
                    }
                    return listener.call(this, e);
                };
                // Preserve reference for removeEventListener
                try { wrapped._qwenOrig = listener; } catch(_) {}
                return origAdd.call(this, type, wrapped, options);
            }
            return origAdd.call(this, type, listener, options);
        };
        // Also patch removeEventListener to handle wrapped listeners
        const origRemove = EventTarget.prototype.removeEventListener;
        EventTarget.prototype.removeEventListener = function(type, listener, options) {
            // Try to find wrapped version — fallback to original
            return origRemove.call(this, type, listener, options);
        };
    })();

    document.addEventListener('wheel', (e) => {
        if (e.ctrlKey) {
            e.preventDefault();
            const delta = e.deltaY > 0 ? -ZOOM_STEP : ZOOM_STEP;
            applyZoom(currentZoom + delta);
        }
    }, { passive: false });

    // --- browser shortcuts (F keys, reload, fullscreen, devtools, find) ---
    // F5 / Ctrl+R soft reload, Ctrl+F5 / Ctrl+Shift+R hard reload,
    // F11 fullscreen, F12 / Ctrl+Shift+I devtools, Ctrl+F / F3 find.
    // These supplement native menu accelerators (GTK) for when menu is hidden
    // or site swallows the event.
    document.addEventListener('keydown', (e) => {
        // Don't steal typing when focus is inside find bar (except handled keys)
        const inFind = typeof window.__qwenFindBar !== 'undefined' && window.__qwenFindBar?.isOpen?.() && e.target && e.target.closest && e.target.closest('#__qwen-find-bar');
        if (inFind && e.key !== 'F5' && e.key !== 'F11' && e.key !== 'F12' && e.key !== 'F3' && e.key !== 'Escape' && e.key !== 'Enter') {
            return;
        }
        const key = e.key;
        const ctrl = e.ctrlKey || e.metaKey;

        // F5 soft reload (also Ctrl+R native accelerator, but handle F5 here)
        if (key === 'F5') {
            if (ctrl) {
                // Ctrl+F5 hard reload
                e.preventDefault();
                try { location.reload(true); } catch(_) { location.reload(); }
            } else {
                e.preventDefault();
                location.reload();
            }
            return;
        }
        // Ctrl+R soft reload fallback (if menu accelerator missed)
        if (ctrl && (key === 'r' || key === 'R') && !e.shiftKey) {
            e.preventDefault();
            location.reload();
            return;
        }
        // Ctrl+Shift+R hard reload
        if (ctrl && (key === 'r' || key === 'R') && e.shiftKey) {
            e.preventDefault();
            try { location.reload(true); } catch(_) { location.reload(); }
            return;
        }
        // F11 fullscreen
        if (key === 'F11') {
            e.preventDefault();
            // Prefer Rust window fullscreen (covers HeaderBar), fallback to DOM fullscreen
            if (window.__TAURI__?.core?.invoke) {
                window.__TAURI__.core.invoke('toggle_fullscreen').catch(() => {
                    if (!document.fullscreenElement) document.documentElement.requestFullscreen?.();
                    else document.exitFullscreen?.();
                });
            } else {
                if (!document.fullscreenElement) document.documentElement.requestFullscreen?.();
                else document.exitFullscreen?.();
            }
            return;
        }
        // F12 devtools (also Ctrl+Shift+I via menu accelerator)
        if (key === 'F12') {
            e.preventDefault();
            window.__TAURI__?.core?.invoke?.('toggle_hidden_devtools').catch(()=>{});
            return;
        }
        // Ctrl+F find
        if (ctrl && (key === 'f' || key === 'F') && !e.shiftKey && !e.altKey) {
            e.preventDefault();
            window.__qwenFindOpen && window.__qwenFindOpen();
            return;
        }
        // F3 find next/prev (Shift+F3 = prev)
        if (key === 'F3') {
            e.preventDefault();
            if (window.__qwenFindBar && window.__qwenFindBar.isOpen()) {
                if (e.shiftKey) window.__qwenFindPrev && window.__qwenFindPrev();
                else window.__qwenFindNext && window.__qwenFindNext();
            } else {
                window.__qwenFindOpen && window.__qwenFindOpen();
            }
            return;
        }
        // Ctrl+Shift+I devtools fallback (menu already has accelerator)
        if (ctrl && e.shiftKey && (key === 'I' || key === 'i')) {
            // let menu handle, but also ensure invoke if menu hidden
            // don't preventDefault here to allow menu accelerator
            window.__TAURI__?.core?.invoke?.('toggle_hidden_devtools').catch(()=>{});
            return;
        }

        // Zoom (Ctrl+Plus/Minus/0) — keep after browser keys
        if (ctrl) {
            if (key === '=' || key === '+') {
                e.preventDefault();
                applyZoom(currentZoom + ZOOM_STEP);
            } else if (key === '-') {
                e.preventDefault();
                applyZoom(currentZoom - ZOOM_STEP);
            } else if (key === '0') {
                e.preventDefault();
                applyZoom(1.0);
            }
        }
    });

    // --- drag & drop ---
    document.addEventListener('dragenter', (e) => {
        if (e.dataTransfer && Array.from(e.dataTransfer.types || []).includes('Files')) {
            e.preventDefault();
        }
    });

    document.addEventListener('dragover', (e) => {
        if (e.dataTransfer && Array.from(e.dataTransfer.types || []).includes('Files')) {
            e.preventDefault();
            e.dataTransfer.dropEffect = 'copy';
        }
    });

    document.addEventListener('drop', (e) => {
        if (e.dataTransfer && Array.from(e.dataTransfer.types || []).includes('Files')) {
            e.preventDefault();
        }
    });

    // --- file/image injection (drag-drop + image paste) ---
    // WebKitGTK bug #218519: DataTransfer::items nunca expõe image/*, então
    // o site não recebe o File de imagem no paste nativo. Em vez disso
    // lemos a imagem via Rust (read_clipboard_image) e injetamos
    // manualmente nos alvos do Qwen, na ordem em que o site os usa.
    function findFileInput() {
        // Ordem de prioridade baseada nos seletores comuns do chat.qwen.ai
        // e em inputs de arquivo genéricos que aceitam imagem.
        const selectors = [
            '#file-upload',
            '[data-testid="file-upload"]',
            'input[type="file"][accept*="image"]',
            '#filesUpload',
            'input[type="file"]',
        ];
        for (const sel of selectors) {
            const el = document.querySelector(sel);
            if (el && el.tagName === 'INPUT') return el;
        }
        // Fallback: qualquer input[type=file], preferindo os que aceitam imagem
        const all = Array.from(document.querySelectorAll('input[type="file"]'));
        if (all.length > 0) {
            const withImage = all.find((el) => {
                const acc = (el.getAttribute('accept') || '').toLowerCase();
                return acc.includes('image');
            });
            return withImage || all[0];
        }
        return null;
    }

    // Seletores do container que envolve a textarea do composer.
    // É o alvo REAL do drag-and-drop do site (o <input type=file> só é usado
    // pelo clique no botão de anexo). Manter ordem do mais específico → mais
    // genérico, pois o site pode renomear classes em updates.
    const DROP_TARGET_SELECTORS = [
        '.message-input-container-area',
        '.message-input-container',
        '.prompt-input-container',
        '#dropzone-container',
        '[contenteditable="true"]',
        'textarea',
    ];

    function findComposerDropTarget() {
        // 1) Preferência: o container do composer (alvo real de drop do site)
        for (const sel of DROP_TARGET_SELECTORS) {
            const el = document.querySelector(sel);
            if (el) return el;
        }
        return document.body;
    }

    function findDropTarget() {
        const ae = document.activeElement;
        if (ae && (ae.isContentEditable || ae.closest('[contenteditable="true"]'))) {
            return ae.isContentEditable ? ae : ae.closest('[contenteditable="true"]');
        }
        return findComposerDropTarget();
    }

    /**
     * Tenta injetar a imagem disparando a sequência completa de eventos de
     * drag-and-drop (dragenter → dragover → drop) no container do composer.
     * É o caminho primário de upload do chat.qwen.ai.
     *
     * @returns {boolean} true se o drop foi disparado sem erros
     */
    function trySyntheticDrop(file, dt) {
        const target = findComposerDropTarget();
        if (!target || target === document.body) {
            if(window.__QWEN_DEBUG) console.log('[Qwen Studio] drop: nenhum container de composer encontrado, pulando drop sintético');
            return false;
        }
        try {
            const tag = (target.tagName || '').toLowerCase();
            const cls = (target.className && typeof target.className === 'string')
                ? target.className.slice(0, 60) : '';
            if(window.__QWEN_DEBUG) console.log(`[Qwen Studio] drop: alvo encontrado <${tag} class="${cls}">, disparando dragenter/dragover/drop`);
            target.dispatchEvent(new DragEvent('dragenter', { bubbles: true, cancelable: true, dataTransfer: dt }));
            target.dispatchEvent(new DragEvent('dragover',  { bubbles: true, cancelable: true, dataTransfer: dt }));
            const dropOk = target.dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: dt }));
            if(window.__QWEN_DEBUG) console.log('[Qwen Studio] drop sintético disparado (resultado:', dropOk, ')');
            return true;
        } catch (err) {
            console.warn('[Qwen Studio] drop sintético falhou:', err);
            return false;
        }
    }

    window.__qwenInjectFile = function(base64, mimeType, fileName) {
        let bytes;
        try {
            bytes = Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
        } catch (err) {
            console.warn('[Qwen Studio] __qwenInjectFile: base64 decode failed', err);
            return false;
        }
        const file = new File([bytes], fileName || 'pasted-image.png', {
            type: mimeType || 'image/png',
        });
        const dt = new DataTransfer();
        dt.items.add(file);

        // 1) Caminho primário: drag-and-drop sintético no container do
        //    composer. É o mecanismo REAL de upload do chat.qwen.ai — o
        //    <input type=file> é usado só pelo clique no botão de anexo.
        const dropAttempted = trySyntheticDrop(file, dt);
        if (dropAttempted) {
            // Não retornamos true imediatamente: o site pode ignorar eventos
            // com isTrusted=false. Em paralelo tentamos o fallback via input
            // para maximizar a chance — se um funcionar, ótimo. O input
            // fallback é silencioso (não conflita com o drop).
            if(window.__QWEN_DEBUG) console.log('[Qwen Studio] drop sintético disparado; tentando também fallback <input> em paralelo');
        }

        // 2) Fallback (ou tentativa paralela): injetar via <input type=file>
        //    oculto e disparar change. Funciona para fluxos que observam
        //    o input (ex.: botão de anexo), mas o React do Qwen normalmente
        //    ignora este caminho para paste — por isso priorizamos o drop.
        const input = findFileInput();
        if (input) {
            try {
                // Força visibilidade: alguns sites deixam o input com
                // display:none / hidden e o WebKit pode ignorar a atribuição
                // de .files. Restauramos o estado logo após o dispatch.
                const origDisplay = input.style.display;
                const origVisibility = input.style.visibility;
                const origHidden = input.hidden;
                input.style.display = 'block';
                input.style.visibility = 'visible';
                input.hidden = false;

                input.files = dt.files;
                input.dispatchEvent(new Event('change', { bubbles: true }));
                input.dispatchEvent(new Event('input', { bubbles: true }));

                input.style.display = origDisplay;
                input.style.visibility = origVisibility;
                input.hidden = origHidden;

                if(window.__QWEN_DEBUG) console.log('[Qwen Studio] injected image via <input type=file> fallback:', fileName, input);
                return true;
            } catch (err) {
                console.warn('[Qwen Studio] input injection failed', err);
            }
        } else if (!dropAttempted) {
            console.warn('[Qwen Studio] no <input type=file> found and drop not attempted');
        }

        // Se chegamos aqui e o drop foi disparado, considere um sucesso
        // "otimista": o site pode ter processado o arquivo mesmo sem
        // retornar cancelable=false. Caso contrário, falhou.
        return dropAttempted;
    };

    // Injeta um File já construído (usado pelo fluxo chunkado binário).
    // Reaproveita trySyntheticDrop + fallback input sem re-decodificar base64.
    function injectFileObject(file) {
        const dt = new DataTransfer();
        dt.items.add(file);
        const dropAttempted = trySyntheticDrop(file, dt);
        if (dropAttempted) {
            if(window.__QWEN_DEBUG) console.log('[Qwen Studio] drop sintético (File object) disparado; tentando fallback <input>');
        }
        const input = findFileInput();
        if (input) {
            try {
                const origDisplay = input.style.display;
                const origVisibility = input.style.visibility;
                const origHidden = input.hidden;
                input.style.display = 'block';
                input.style.visibility = 'visible';
                input.hidden = false;
                input.files = dt.files;
                input.dispatchEvent(new Event('change', { bubbles: true }));
                input.dispatchEvent(new Event('input', { bubbles: true }));
                input.style.display = origDisplay;
                input.style.visibility = origVisibility;
                input.hidden = origHidden;
                if(window.__QWEN_DEBUG) console.log('[Qwen Studio] injected File object via <input> fallback:', file.name);
                return true;
            } catch (err) {
                console.warn('[Qwen Studio] input injection (File object) failed', err);
            }
        } else if (!dropAttempted) {
            console.warn('[Qwen Studio] no <input type=file> found and drop not attempted (File object)');
        }
        return dropAttempted;
    }
    window.__qwenInjectFileObject = injectFileObject;

    // --- drag-drop chunkado binário (sem limite) ---
    // Rust envia só metas via eval; JS busca chunks binários via invoke('read_file_chunk')
    // que retorna Response(Vec<u8>) → ArrayBuffer nativo (Tauri v2), sem base64.
    let __qwenDropQueue = [];
    let __qwenDropping = false;

    async function injectLargeFile({ path, name, mime, size }) {
        const CHUNK = 4 * 1024 * 1024; // 4 MiB
        const parts = [];
        if(window.__QWEN_DEBUG) console.log(`[Qwen Studio] iniciando transferência binária: ${name} (${size} bytes, mime=${mime})`);
        try {
            for (let off = 0; off < size; off += CHUNK) {
                const len = Math.min(CHUNK, size - off);
                // Tauri v2: invoke retorna ArrayBuffer quando Rust retorna Response
                const buffer = await window.__TAURI__.core.invoke('read_file_chunk', {
                    path,
                    offset: off,
                    length: len,
                });
                // buffer pode ser ArrayBuffer, Uint8Array ou array numérico dependendo da versão
                let u8;
                if (buffer instanceof ArrayBuffer) {
                    u8 = new Uint8Array(buffer);
                } else if (buffer instanceof Uint8Array) {
                    u8 = buffer;
                } else if (Array.isArray(buffer)) {
                    u8 = new Uint8Array(buffer);
                } else if (buffer && buffer.buffer instanceof ArrayBuffer) {
                    u8 = new Uint8Array(buffer.buffer);
                } else {
                    // fallback: se veio como objeto com dados
                    console.warn('[Qwen Studio] formato de chunk inesperado', typeof buffer);
                    continue;
                }
                parts.push(u8);
                // Yield a cada 16 MiB para não congelar UI e dar chance ao GC
                if (off > 0 && off % (16 * 1024 * 1024) === 0) {
                    await new Promise((r) => setTimeout(r, 0));
                }
            }
            const file = new File(parts, name || 'file', { type: mime || 'application/octet-stream' });
            if(window.__QWEN_DEBUG) console.log(`[Qwen Studio] File montado: ${file.name} ${file.size} bytes, injetando...`);
            const ok = injectFileObject(file);
            if (!ok) console.warn('[Qwen Studio] injeção do File chunkado falhou:', name);
            return ok;
        } catch (err) {
            console.error('[Qwen Studio] injectLargeFile falhou:', name, err);
            return false;
        } finally {
            parts.length = 0;
        }
    }

    window.__qwenHandleDrop = async function(metas) {
        if (!Array.isArray(metas) || metas.length === 0) return;
        __qwenDropQueue.push(...metas);
        if (__qwenDropping) {
            if(window.__QWEN_DEBUG) console.log('[Qwen Studio] drop enfileirado (já processando), fila:', __qwenDropQueue.length);
            return;
        }
        __qwenDropping = true;
        while (__qwenDropQueue.length > 0) {
            const m = __qwenDropQueue.shift();
            try {
                if(window.__QWEN_DEBUG) console.log('[Qwen Studio] processando drop:', m.name, m.size, 'bytes');
                await injectLargeFile(m);
            } catch (e) {
                console.error('[Qwen Studio] falha no drop:', m.name, e);
            }
            // pequena pausa entre arquivos para GC
            if (__qwenDropQueue.length > 0) await new Promise((r) => setTimeout(r, 100));
        }
        __qwenDropping = false;
    };

    // --- inserção de texto robusta em componentes React ---
    // INPUT/TEXTAREA: usa o native value setter (React controlado ignora
    // atribuição direta) e dispara `input`. contenteditable: muta o DOM e
    // dispara InputEvent com inputType insertText para o React capturar.
    function setNativeValue(el, value) {
        const proto = el.tagName === 'TEXTAREA'
            ? window.HTMLTextAreaElement.prototype
            : window.HTMLInputElement.prototype;
        const setter = Object.getOwnPropertyDescriptor(proto, 'value').set;
        setter.call(el, value);
    }

    function insertTextAtCursorRobust(text) {
        const el = document.activeElement;
        if (!el) return false;

        if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA') {
            const start = el.selectionStart != null ? el.selectionStart : el.value.length;
            const end = el.selectionEnd != null ? el.selectionEnd : start;
            const next = el.value.slice(0, start) + text + el.value.slice(end);
            setNativeValue(el, next);
            const pos = start + text.length;
            try { el.setSelectionRange(pos, pos); } catch (_) {}
            el.dispatchEvent(new Event('input', { bubbles: true }));
            el.dispatchEvent(new Event('change', { bubbles: true }));
            return true;
        }

        if (el.isContentEditable || el.getAttribute('contenteditable') === 'true') {
            const sel = window.getSelection();
            if (sel && sel.rangeCount > 0) {
                const range = sel.getRangeAt(0);
                range.deleteContents();
                const textNode = document.createTextNode(text);
                range.insertNode(textNode);
                range.setStartAfter(textNode);
                range.setEndAfter(textNode);
                sel.removeAllRanges();
                sel.addRange(range);
            } else {
                el.appendChild(document.createTextNode(text));
            }
            el.dispatchEvent(new InputEvent('input', {
                inputType: 'insertText',
                data: text,
                bubbles: true,
                cancelable: true,
            }));
            return true;
        }

        return false;
    }

    // --- leitura concorrente do clipboard (imagem + texto) com timeout ---
    // Garante que um read_clipboard_image lento (ex.: Wayland sem wl-clipboard)
    // nunca trave o paste de texto. Retorna { imageB64, text }.
    function readClipboardWithTimeout(timeoutMs) {
        const readImage = window.__TAURI__.core
            .invoke('read_clipboard_image')
            .then((b64) => (b64 && b64.length ? b64 : null))
            .catch(() => null);
        const readText = window.__TAURI__.core
            .invoke('plugin:clipboard-manager|read_text')
            .then((t) => (t ? String(t) : null))
            .catch(() => null);

        return new Promise((resolve) => {
            let settled = false;
            const finish = (r) => {
                if (settled) return;
                settled = true;
                resolve(r);
            };
            Promise.allSettled([readImage, readText]).then(([img, txt]) => {
                finish({
                    imageB64: img.status === 'fulfilled' ? img.value : null,
                    text: txt.status === 'fulfilled' ? txt.value : null,
                });
            });
            setTimeout(() => finish({ imageB64: null, text: null }), timeoutMs);
        });
    }

    // --- injeção de fallback (Rust) quando o path nativo do site falha ---
    // Lê imagem e texto em paralelo e injeta manualmente. Usado apenas como
    // fallback: o site (chat.qwen.ai) normalmente cola sozinho via shim
    // electron.clipboard. Ordem: 1) imagem 2) texto.
    window.__qwenHandlePaste = async function() {
        if (__pasteInProgress) return;
        __pasteInProgress = true;
        const savedActiveElement = document.activeElement;
        try {
            const { imageB64, text } = await readClipboardWithTimeout(1000);

            // 1) Tenta imagem (PrintScreen / Copiar imagem)
            if (imageB64) {
                if(window.__QWEN_DEBUG) console.log('[Qwen Studio] imagem lida do clipboard (Rust), injetando...', imageB64.substring(0, 32) + '...');
                if (window.__qwenInjectFile(imageB64, 'image/png', 'pasted-image.png')) return;
                console.warn('[Qwen Studio] imagem lida, mas a injeção falhou — tentando texto');
            }

            // 2) Tenta texto
            if (text) {
                if (savedActiveElement && savedActiveElement !== document.body) {
                    try { savedActiveElement.focus(); } catch (_) {}
                }
                if(window.__QWEN_DEBUG) console.log('[Qwen Studio] texto lido do clipboard, inserindo no cursor');
                insertTextAtCursorRobust(text);
            }
        } finally {
            __pasteInProgress = false;
        }
    };

    // --- fallback agendado com diff de valor ---
    // Não intercepta o paste nativo do site. Apenas observa: se após o
    // Ctrl+V/`paste` o campo editável focado NÃO mudou e o clipboard tem
    // conteúdo, injeta manualmente via Rust. Evita tanto o "nada cola"
    // (site falhou) quanto o duplo colar (site já colou).
    const FALLBACK_DELAY_MS = 200;
    window.__qwenScheduleFallbackPaste = function() {
        if (__pasteInProgress) return;
        const ae = document.activeElement;
        const editable = ae && (
            ae.tagName === 'INPUT' || ae.tagName === 'TEXTAREA' ||
            ae.isContentEditable || ae.getAttribute('contenteditable') === 'true'
        );
        if (!editable) return; // não é campo de texto: nada a fazer

        const before = (ae.tagName === 'INPUT' || ae.tagName === 'TEXTAREA')
            ? ae.value
            : (ae.innerText || '');

        setTimeout(() => {
            if (__pasteInProgress) return;
            const after = (ae.tagName === 'INPUT' || ae.tagName === 'TEXTAREA')
                ? ae.value
                : (ae.innerText || '');
            // Se o campo já mudou, o site colou com sucesso — não injeta.
            if (after !== before) {
                if(window.__QWEN_DEBUG) console.log('[Qwen Studio] paste nativo do site detectado — fallback ignorado');
                return;
            }
            if(window.__QWEN_DEBUG) console.log('[Qwen Studio] paste nativo não agiu — disparando fallback Rust');
            window.__qwenHandlePaste();
        }, FALLBACK_DELAY_MS);
    };

    // --- observers NÃO destrutivos (não chamam preventDefault) ---
    // O site (Electron) cola via shim electron.clipboard; deixamos o evento
    // prosseguir e apenas agendamos o fallback caso ele falhe.
    document.addEventListener('paste', function() {
        window.__qwenScheduleFallbackPaste();
    }, true);

    document.addEventListener('keydown', function(e) {
        if (!(e.ctrlKey || e.metaKey)) return;
        if (e.key !== 'v' && e.key !== 'V') return;
        window.__qwenScheduleFallbackPaste();
    }, true);

    // --- Find in page (Ctrl+F / F3 / Shift+F3 / Esc) ---
    // Lightweight overlay using window.find(). No WebKit FindController needed.
    (function() {
        let findQuery = '';
        let findBar = null;
        let findInput = null;
        let findCounter = null;

        function ensureFindBar() {
            if (findBar) return;
            findBar = document.createElement('div');
            findBar.id = '__qwen-find-bar';
            findBar.setAttribute('role', 'search');
            Object.assign(findBar.style, {
                position: 'fixed',
                bottom: '16px',
                right: '16px',
                zIndex: '2147483647',
                display: 'none',
                alignItems: 'center',
                gap: '6px',
                padding: '8px 10px',
                background: 'rgba(32,32,32,0.95)',
                color: '#eee',
                borderRadius: '10px',
                boxShadow: '0 4px 16px rgba(0,0,0,0.4)',
                fontFamily: 'system-ui, sans-serif',
                fontSize: '13px',
                border: '1px solid rgba(255,255,255,0.12)',
            });
            findBar.innerHTML = ''
                + '<input id="__qwen-find-input" type="text" placeholder="Localizar" spellcheck="false" autocomplete="off"'
                + ' style="width:200px;padding:6px 8px;border-radius:6px;border:1px solid #555;background:#1e1e1e;color:#eee;outline:none;font-size:13px">'
                + '<span id="__qwen-find-counter" style="min-width:24px;text-align:center;opacity:0.7;font-size:12px"></span>'
                + '<button id="__qwen-find-prev" title="Anterior (Shift+F3)" style="padding:4px 8px;border-radius:6px;border:1px solid #555;background:#2a2a2a;color:#eee;cursor:pointer">▲</button>'
                + '<button id="__qwen-find-next" title="Próximo (F3)" style="padding:4px 8px;border-radius:6px;border:1px solid #555;background:#2a2a2a;color:#eee;cursor:pointer">▼</button>'
                + '<button id="__qwen-find-close" title="Fechar (Esc)" style="padding:4px 8px;border-radius:6px;border:1px solid #555;background:#2a2a2a;color:#eee;cursor:pointer">✕</button>';
            (document.body || document.documentElement).appendChild(findBar);
            findInput = findBar.querySelector('#__qwen-find-input');
            findCounter = findBar.querySelector('#__qwen-find-counter');
            const btnPrev = findBar.querySelector('#__qwen-find-prev');
            const btnNext = findBar.querySelector('#__qwen-find-next');
            const btnClose = findBar.querySelector('#__qwen-find-close');
            btnPrev.addEventListener('click', () => window.__qwenFindPrev && window.__qwenFindPrev());
            btnNext.addEventListener('click', () => window.__qwenFindNext && window.__qwenFindNext());
            btnClose.addEventListener('click', () => window.__qwenFindClose && window.__qwenFindClose());
            findInput.addEventListener('input', () => {
                findQuery = findInput.value;
                if (findQuery) doFind(findQuery, false);
                else clearCounter();
            });
            const _findInputKeyHandler = (e) => {
                // Capture+stopImmediatePropagation blocks chat.qwen.ai's global
                // capture listener that auto-focuses the composer on every keydown.
                if (e.key === 'Enter') {
                    e.preventDefault();
                    e.stopPropagation();
                    if (typeof e.stopImmediatePropagation === 'function') e.stopImmediatePropagation();
                    if (e.shiftKey) window.__qwenFindPrev && window.__qwenFindPrev();
                    else window.__qwenFindNext && window.__qwenFindNext();
                } else if (e.key === 'Escape') {
                    e.preventDefault();
                    e.stopPropagation();
                    if (typeof e.stopImmediatePropagation === 'function') e.stopImmediatePropagation();
                    window.__qwenFindClose && window.__qwenFindClose();
                } else {
                    e.stopPropagation();
                    if (typeof e.stopImmediatePropagation === 'function') e.stopImmediatePropagation();
                }
            };
            findInput.addEventListener('keydown', _findInputKeyHandler, true);
            // Also block keypress/beforeinput in capture — some SPA listeners use them
            findInput.addEventListener('keypress', (e) => {
                e.stopPropagation();
                if (typeof e.stopImmediatePropagation === 'function') e.stopImmediatePropagation();
            }, true);
            findInput.addEventListener('beforeinput', (e) => {
                e.stopPropagation();
                if (typeof e.stopImmediatePropagation === 'function') e.stopImmediatePropagation();
            }, true);
            findInput.addEventListener('mousedown', (e) => e.stopPropagation());
            // Retain focus if SPA steals it (blur -> refocus)
            let _findFocusGuard = 0;
            findInput.addEventListener('blur', () => {
                if (!findBar || findBar.style.display === 'none') return;
                if (_findFocusGuard) return;
                _findFocusGuard = 1;
                setTimeout(() => {
                    _findFocusGuard = 0;
                    if (findBar.style.display !== 'none' && document.activeElement !== findInput) {
                        try { findInput.focus({ preventScroll: true }); } catch(_) { try { findInput.focus(); } catch(_) {} }
                    }
                }, 0);
            });
            findBar.addEventListener('mousedown', (e) => {
                if (e.target !== findInput) e.preventDefault();
            });
        }

        function clearCounter() { if (findCounter) findCounter.textContent = ''; }

        function doFind(query, backwards) {
            if (!query) return false;
            try {
                // window.find is synchronous; use aSelection wrapping
                const found = window.find(query, false, backwards, true, false, false, false);
                if (findCounter) findCounter.textContent = found ? '•' : '—';
                return found;
            } catch(_) { return false; }
        }

        window.__qwenFindOpen = function() {
            ensureFindBar();
            findBar.style.display = 'flex';
            // WebKitGTK needs a frame after display:flex before focus is accepted;
            // SPA also re-focuses composer in a microtask, so defer.
            requestAnimationFrame(() => {
                setTimeout(() => {
                    try { findInput.focus({ preventScroll: true }); } catch(_) { findInput.focus(); }
                    try { findInput.select(); } catch(_) {}
                    if (findInput.value) {
                        findQuery = findInput.value;
                        doFind(findQuery, false);
                    }
                }, 0);
            });
        };
        window.__qwenFindClose = function() {
            if (findBar) findBar.style.display = 'none';
            try { window.getSelection()?.removeAllRanges(); } catch(_) {}
            clearCounter();
        };
        window.__qwenFindNext = function() {
            if (findInput) findQuery = findInput.value;
            if (!findQuery) return;
            doFind(findQuery, false);
        };
        window.__qwenFindPrev = function() {
            if (findInput) findQuery = findInput.value;
            if (!findQuery) return;
            doFind(findQuery, true);
        };
        window.__qwenFindBar = {
            isOpen: () => !!(findBar && findBar.style.display !== 'none'),
        };

        // Esc closes find bar or stops loading (browser parity)
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                if (findBar && findBar.style.display !== 'none') {
                    e.preventDefault();
                    window.__qwenFindClose();
                    return;
                }
                // If page is still loading, stop (browser Stop)
                if (document.readyState === 'loading') {
                    try { window.stop(); } catch(_) {}
                }
            }
        }, true);
    })();

    window.open = function(url, target, features) {
        if (url && (url.startsWith('http://') || url.startsWith('https://'))) {
            window.electronAPI?.open_external_link?.(url);
        }
        return null;
    };

    if(window.__QWEN_DEBUG) console.log('[Qwem Studio] Platform bridge loaded');
})();
