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

    function applyZoom(scale) {
        currentZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, scale));
<<<<<<< HEAD
<<<<<<< HEAD
        document.documentElement.style.zoom = currentZoom;
        document.body.style.zoom = currentZoom;
        // Persiste via Rust para paridade navegador
        try { window.__TAURI__?.core?.invoke?.('handle_shortcut', { action: currentZoom === 1.0 ? 'zoom_reset' : (scale > currentZoom ? 'zoom_in' : 'zoom_out') }); } catch(_) {}
    }
    function applyZoomViaRust(action) {
        try { window.__TAURI__?.core?.invoke?.('handle_shortcut', { action }); } catch(_) {}
        // Optimistic local update for instant feedback
        const cur = parseFloat(document.documentElement.style.zoom || '1') || 1.0;
        if (action === 'zoom_in') {
            const z = Math.min(MAX_ZOOM, cur + ZOOM_STEP);
            document.documentElement.style.zoom = z; document.body.style.zoom = z; currentZoom = z;
        } else if (action === 'zoom_out') {
            const z = Math.max(MIN_ZOOM, cur - ZOOM_STEP);
            document.documentElement.style.zoom = z; document.body.style.zoom = z; currentZoom = z;
        } else if (action === 'zoom_reset') {
            document.documentElement.style.zoom = 1.0; document.body.style.zoom = 1.0; currentZoom = 1.0;
        }
=======
        window.__qwenCurrentZoom = currentZoom;
        if (__zoomTimer) clearTimeout(__zoomTimer);
        __zoomTimer = setTimeout(() => {
            // Native WebView zoom via Rust (wry set_zoom_level) — no reflow lag
            const invoke = window.__TAURI__?.core?.invoke;
            if (invoke) {
                invoke('set_zoom', { factor: currentZoom }).catch(() => {
                    // Fallback to plugin directly if custom command unavailable
                    invoke('plugin:webview|set_webview_zoom', { value: currentZoom }).catch(() => {});
                });
            }
        }, 50);
>>>>>>> ce2f600 (optimization)
=======
        document.body.style.zoom = currentZoom;
>>>>>>> 5877c22 (restore)
    }

    document.addEventListener('wheel', (e) => {
        if (e.ctrlKey) {
            e.preventDefault();
            const delta = e.deltaY > 0 ? 'zoom_out' : 'zoom_in';
            applyZoomViaRust(delta);
        }
    }, { passive: false });

    document.addEventListener('keydown', (e) => {
        if (e.ctrlKey || e.metaKey) {
            if (e.key === '=' || e.key === '+') {
                e.preventDefault();
                applyZoomViaRust('zoom_in');
            } else if (e.key === '-') {
                e.preventDefault();
                applyZoomViaRust('zoom_out');
            } else if (e.key === '0') {
                e.preventDefault();
                applyZoomViaRust('zoom_reset');
            } else if (e.key === 'f' || e.key === 'F') {
                // Deixa handle_shortcut cuidar, mas abre find bar como fallback
                if (!e.shiftKey && !e.altKey) {
                    // não previne, apenas garante fallback
                    setTimeout(() => { window.__qwenFindOpen && window.__qwenFindOpen(); }, 50);
                }
            }
        }
        // Alt+Left / Alt+Right para back/forward (paridade navegador)
        if (e.altKey && !e.ctrlKey && !e.metaKey) {
            if (e.key === 'ArrowLeft') {
                e.preventDefault();
                try { window.__TAURI__?.core?.invoke?.('handle_shortcut', { action: 'go_back' }); } catch(_) { history.back(); }
            } else if (e.key === 'ArrowRight') {
                e.preventDefault();
                try { window.__TAURI__?.core?.invoke?.('handle_shortcut', { action: 'go_forward' }); } catch(_) { history.forward(); }
            }
        }
    });

    // Middle-click (auxclick) em links → abrir externo/nova janela (paridade navegador)
    document.addEventListener('auxclick', (e) => {
        if (e.button === 1) {
            const a = e.target.closest && e.target.closest('a[href]');
            if (a && a.href) {
                const href = a.href;
                if (href.startsWith('http://') || href.startsWith('https://')) {
                    e.preventDefault();
                    window.electronAPI?.open_external_link?.(href);
                }
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
<<<<<<< HEAD
            if(window.__QWEN_DEBUG) console.log('[Qwen Studio] drop sintético (File object) disparado; tentando fallback <input>');
=======
            console.log('[Qwen Studio] drop sintético (File object) disparado; tentando fallback <input>');
>>>>>>> c0c2f30 (Fix: Upload medias e username)
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
<<<<<<< HEAD
                if(window.__QWEN_DEBUG) console.log('[Qwen Studio] injected File object via <input> fallback:', file.name);
=======
                console.log('[Qwen Studio] injected File object via <input> fallback:', file.name);
>>>>>>> c0c2f30 (Fix: Upload medias e username)
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
<<<<<<< HEAD
        const MAX_SIZE = 100 * 1024 * 1024; // 100 MiB limite (economia RAM) — window_utils.rs também filtra
        if (size > MAX_SIZE) {
            console.warn(`[Qwen Studio] arquivo ignorado, excede 100 MiB: ${name} (${size} bytes)`);
            return false;
        }
        const CHUNK = 4 * 1024 * 1024; // 4 MiB
        const parts = [];
        if(window.__QWEN_DEBUG) console.log(`[Qwen Studio] iniciando transferência binária: ${name} (${size} bytes, mime=${mime})`);
=======
        const CHUNK = 4 * 1024 * 1024; // 4 MiB
        const parts = [];
        console.log(`[Qwen Studio] iniciando transferência binária: ${name} (${size} bytes, mime=${mime})`);
>>>>>>> c0c2f30 (Fix: Upload medias e username)
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
<<<<<<< HEAD
            if(window.__QWEN_DEBUG) console.log(`[Qwen Studio] File montado: ${file.name} ${file.size} bytes, injetando...`);
=======
            console.log(`[Qwen Studio] File montado: ${file.name} ${file.size} bytes, injetando...`);
>>>>>>> c0c2f30 (Fix: Upload medias e username)
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
<<<<<<< HEAD
            if(window.__QWEN_DEBUG) console.log('[Qwen Studio] drop enfileirado (já processando), fila:', __qwenDropQueue.length);
=======
            console.log('[Qwen Studio] drop enfileirado (já processando), fila:', __qwenDropQueue.length);
>>>>>>> c0c2f30 (Fix: Upload medias e username)
            return;
        }
        __qwenDropping = true;
        while (__qwenDropQueue.length > 0) {
            const m = __qwenDropQueue.shift();
            try {
<<<<<<< HEAD
                if(window.__QWEN_DEBUG) console.log('[Qwen Studio] processando drop:', m.name, m.size, 'bytes');
=======
                console.log('[Qwen Studio] processando drop:', m.name, m.size, 'bytes');
>>>>>>> c0c2f30 (Fix: Upload medias e username)
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

    // window.open browser-like: navega dentro do WebView para http(s) do mesmo
    // site, abre externo via open_external_link para domínios terceiros.
    // Mantém compatibilidade: isTrusted=false ainda funciona porque on_navigation
    // agora permite https geral (src/webview/navigation.rs).
    const _nativeOpen = window.open.bind(window);
    window.open = function(url, target, features) {
        if (!url) return null;
        try {
            const isHttp = url.startsWith('http://') || url.startsWith('https://');
            const isSpecial = url.startsWith('mailto:') || url.startsWith('blob:') || url.startsWith('data:');
            if (isHttp || isSpecial) {
                // Tenta navegar dentro do WebView quando for _self/_blank sem noopener estrito;
                // para links externos terceiros, delega ao OS via open_external_link mas ainda
                // retorna um window proxy para não quebrar paginas que checam if (win) { win.postMessage }
                if (target === '_self' || !target || target === '_blank') {
                    // Para chat.qwen.ai e auth, deixar WebView navegar; para outros, abrir externo
                    const isChatOrAuth = url.includes('chat.qwen.ai') || url.includes('aliyun.com') || url.includes('accounts.google.com') || url.includes('oauth2.googleapis');
                    if (isChatOrAuth && isHttp) {
                        // Navegação interna: usa href para não depender de popup
                        if (target === '_self') {
                            window.location.href = url;
                            return window;
                        }
                        // _blank interno: tenta _nativeOpen se disponível, fallback para href
                        try { const w = _nativeOpen(url, target, features); if (w) return w; } catch(_) {}
                        window.electronAPI?.open_external_link?.(url);
                        return null;
                    }
                    window.electronAPI?.open_external_link?.(url);
                    return null;
                }
                window.electronAPI?.open_external_link?.(url);
                return null;
            }
        } catch(_) {}
        try { return _nativeOpen(url, target, features); } catch(_) { return null; }
    };

    // --- Find bar (Ctrl+F) — paridade navegador ---
    (function() {
        let findBar = null;
        let lastQuery = '';
        function createFindBar() {
            if (findBar) return findBar;
            const bar = document.createElement('div');
            bar.id = 'qwen-find-bar';
            bar.style.cssText = 'position:fixed;top:0;right:20px;z-index:999999;background:#1e1e1e;color:#eee;border:1px solid #333;border-top:none;border-radius:0 0 8px 8px;padding:8px 10px;display:flex;gap:6px;align-items:center;font-family:sans-serif;font-size:13px;box-shadow:0 4px 12px rgba(0,0,0,.4)';
            bar.innerHTML = '<input id="qwen-find-input" placeholder="Buscar" style="background:#2d2d2d;color:#fff;border:1px solid #444;border-radius:4px;padding:4px 8px;outline:none;width:200px"><button id="qwen-find-prev" style="background:#2d2d2d;color:#eee;border:1px solid #444;border-radius:4px;padding:4px 8px;cursor:pointer">◀</button><button id="qwen-find-next" style="background:#2d2d2d;color:#eee;border:1px solid #444;border-radius:4px;padding:4px 8px;cursor:pointer">▶</button><span id="qwen-find-count" style="min-width:40px;text-align:center;opacity:.7"></span><button id="qwen-find-close" style="background:transparent;color:#aaa;border:none;font-size:16px;cursor:pointer;padding:2px 6px">✕</button>';
            document.documentElement.appendChild(bar);
            const input = bar.querySelector('#qwen-find-input');
            const count = bar.querySelector('#qwen-find-count');
            let findIndex = 0;
            function doFind(backwards) {
                const q = input.value;
                if (!q) { count.textContent = ''; return; }
                if (q !== lastQuery) { lastQuery = q; findIndex = 0; }
                try {
                    const found = window.find(q, false, backwards, true, false, false, false);
                    count.textContent = found ? '✓' : '0';
                } catch(_) { count.textContent = ''; }
            }
            input.addEventListener('input', () => doFind(false));
            input.addEventListener('keydown', (e) => {
                if (e.key === 'Enter') { e.preventDefault(); doFind(e.shiftKey); }
                if (e.key === 'Escape') { e.preventDefault(); closeBar(); }
            });
            bar.querySelector('#qwen-find-next').addEventListener('click', () => doFind(false));
            bar.querySelector('#qwen-find-prev').addEventListener('click', () => doFind(true));
            bar.querySelector('#qwen-find-close').addEventListener('click', closeBar);
            function closeBar() {
                bar.style.display = 'none';
                try { window.getSelection()?.removeAllRanges(); } catch(_) {}
                count.textContent = '';
            }
            findBar = bar;
            findBar._input = input;
            findBar._close = closeBar;
            // ESC global fecha
            document.addEventListener('keydown', (e) => {
                if (e.key === 'Escape' && bar.style.display !== 'none') closeBar();
            });
            return bar;
        }
        window.__qwenFindOpen = function() {
            const bar = createFindBar();
            bar.style.display = 'flex';
            const input = bar._input;
            const sel = window.getSelection()?.toString() || '';
            if (sel && sel.length < 100) input.value = sel;
            input.focus();
            input.select();
            if (input.value) {
                try { window.find(input.value, false, false, true, false, false, false); } catch(_) {}
            }
        };
        window.__qwenFindClose = function() { if (findBar) findBar._close(); };
    })();

    if(window.__QWEN_DEBUG) console.log('[Qwem Studio] Platform bridge loaded');
})();
