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

    let __zoomTimer = null;
    function applyZoom(scale) {
        currentZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, scale));
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
    }
    // Expose for Rust menu/shortcuts via eval
    window.__qwenCurrentZoom = currentZoom;
    window.__qwenSetZoom = function(factor) {
        applyZoom(factor);
    };
    window.__qwenSyncZoom = function(factor) {
        currentZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, factor));
        window.__qwenCurrentZoom = currentZoom;
    };

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

    // --- fallback anti-loading infinito para preview qwenlm.io ---
    setInterval(function() {
        try {
            var frame = document.querySelector('iframe[src*="qwenlm.io"]');
            var loading = document.querySelector('.artifact-iframe-loading');
            if (frame && loading && loading.style.display !== 'none') {
                var hasContent = (frame.contentDocument && frame.contentDocument.readyState === 'complete') || (frame.src && frame.src.indexOf('qwenlm.io') !== -1);
                if (hasContent) {
                    loading.style.display = 'none';
                    frame.style.display = 'block';
                    if(window.__QWEN_DEBUG) console.log('[Qwen Studio] artifact spinner hidden, iframe forced visible');
                }
            }
        } catch (_) {}
    }, 1000);

    // --- download interceptor: abre diálogo "Salvar como" do SO ---
    // Intercepta <a download>, blob: e data: para exibir o seletor nativo
    // (xdg-desktop-portal -> Dolphin/Nautilus/qualquer gerenciador padrão).
    let __downloadInProgress = false;

    function sanitizeFilenameForDownload(name) {
        if (!name || typeof name !== 'string') return 'download';
        // remove path, keep basename
        let base = name.split('/').pop().split('\\').pop().trim();
        if (!base) base = 'download';
        // strip query/hash
        base = base.split('?')[0].split('#')[0];
        if (base.length > 200) {
            const dot = base.lastIndexOf('.');
            if (dot > 0) {
                const ext = base.slice(dot);
                base = base.slice(0, 200 - ext.length) + ext;
            } else {
                base = base.slice(0, 200);
            }
        }
        return base || 'download';
    }

    function filenameFromUrl(url, fallback) {
        try {
            if (url.startsWith('data:')) {
                // data:mime;... — no filename; use fallback or guess ext
                const m = url.slice(5, 80).split(';')[0].split(',')[0].trim();
                const extMap = { 'image/png':'png','image/jpeg':'jpg','image/gif':'gif','image/webp':'webp','application/pdf':'pdf','text/plain':'txt','text/csv':'csv' };
                const ext = extMap[m] || '';
                const base = fallback && fallback.includes('.') ? fallback : (fallback || 'download');
                if (ext && !base.includes('.')) return base + '.' + ext;
                return base;
            }
            if (url.startsWith('blob:')) {
                return sanitizeFilenameForDownload(fallback || 'download');
            }
            const u = new URL(url, location.href);
            const pathPart = u.pathname.split('/').pop();
            if (pathPart && pathPart.includes('.')) return sanitizeFilenameForDownload(decodeURIComponent(pathPart));
            if (pathPart && pathPart.length > 1) return sanitizeFilenameForDownload(decodeURIComponent(pathPart));
        } catch (_) {}
        return sanitizeFilenameForDownload(fallback || 'download');
    }

    function blobToBase64(blob) {
        return new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.onloadend = () => {
                try {
                    const res = reader.result; // data:mime;base64,...
                    const comma = res.indexOf(',');
                    resolve(comma >= 0 ? res.slice(comma + 1) : res);
                } catch (e) { reject(e); }
            };
            reader.onerror = () => reject(reader.error || new Error('FileReader failed'));
            reader.readAsDataURL(blob);
        });
    }

    async function handleDownloadRequest(href, suggestedName, mimeHint) {
        if (__downloadInProgress) return;
        // filter out non-download navigations
        const isBlob = href.startsWith('blob:');
        const isData = href.startsWith('data:');
        const hasDownloadAttr = !!suggestedName;
        // also treat blob/data as download even without attr
        if (!isBlob && !isData && !hasDownloadAttr) return;
        // ignore javascript: / mailto: etc
        if (href.startsWith('javascript:') || href.startsWith('mailto:')) return;

        __downloadInProgress = true;
        try {
            if (window.__QWEN_DEBUG) console.log('[Qwen Studio] interceptando download:', href.slice(0, 120), 'nome:', suggestedName);
            const filename = filenameFromUrl(href, suggestedName);
            // fetch the resource (works for blob: and data: and same-origin https:)
            let blob;
            try {
                const resp = await fetch(href);
                if (!resp.ok) throw new Error('fetch status ' + resp.status);
                blob = await resp.blob();
            } catch (fetchErr) {
                // fallback: data: manual decode
                if (isData) {
                    const comma = href.indexOf(',');
                    const meta = href.slice(5, comma);
                    const isB64 = meta.includes('base64');
                    const mimePart = meta.split(';')[0] || mimeHint || 'application/octet-stream';
                    let b64 = href.slice(comma + 1);
                    if (!isB64) b64 = btoa(decodeURIComponent(b64));
                    const invoke = window.__TAURI__?.core?.invoke;
                    if (invoke) {
                        await invoke('save_downloaded_file', { filename, mime: mimePart, dataBase64: b64 });
                        if (window.__QWEN_DEBUG) console.log('[Qwen Studio] download data: salvo via fallback', filename);
                    }
                    return;
                }
                console.warn('[Qwen Studio] falha ao buscar blob para download:', fetchErr);
                // fallback para https: tenta abrir externamente para não perder o arquivo
                if (href.startsWith('http://') || href.startsWith('https://')) {
                    try { window.electronAPI?.open_external_link?.(href); } catch (_) {}
                }
                return;
            }
            const mime = blob.type || mimeHint || 'application/octet-stream';
            // 100 MiB cap consistent with Rust
            if (blob.size > 100 * 1024 * 1024) {
                console.warn('[Qwen Studio] arquivo muito grande para download (100 MiB):', blob.size);
                alert('Arquivo muito grande (limite 100 MB): ' + filename);
                return;
            }
            const base64 = await blobToBase64(blob);
            const invoke = window.__TAURI__?.core?.invoke;
            if (!invoke) {
                console.warn('[Qwen Studio] Tauri invoke indisponível para salvar download');
                return;
            }
            try {
                const savedPath = await invoke('save_downloaded_file', { filename, mime, dataBase64: base64 });
                if (window.__QWEN_DEBUG) console.log('[Qwen Studio] download salvo em:', savedPath);
            } catch (saveErr) {
                const msg = String(saveErr || '');
                // "No file selected" = usuário cancelou -> silencioso
                if (msg.includes('No file selected') || msg.includes('cancel')) {
                    if (window.__QWEN_DEBUG) console.log('[Qwen Studio] salvamento cancelado pelo usuário');
                } else {
                    console.warn('[Qwen Studio] falha ao salvar download:', saveErr);
                }
            }
        } finally {
            // small cooldown to avoid double-fire from click+patched click
            setTimeout(() => { __downloadInProgress = false; }, 300);
        }
    }

    // 1) Captura cliques em <a> (fase capture) — pega cliques do usuário
    document.addEventListener('click', function(e) {
        // only left click without modifiers that force new tab
        if (e.defaultPrevented) return;
        if (e.button !== 0) return;
        if (e.ctrlKey || e.metaKey || e.shiftKey) return;
        const anchor = e.target && e.target.closest ? e.target.closest('a[href]') : null;
        if (!anchor) return;
        const href = anchor.getAttribute('href') || anchor.href || '';
        if (!href) return;
        const downloadAttr = anchor.getAttribute('download'); // null if absent, "" if present without value
        const hasDownload = anchor.hasAttribute('download');
        const isBlob = href.startsWith('blob:');
        const isData = href.startsWith('data:');
        if (!hasDownload && !isBlob && !isData) return;
        const suggested = hasDownload ? (downloadAttr || anchor.download || '') : '';
        // intercept
        e.preventDefault();
        e.stopPropagation();
        e.stopImmediatePropagation();
        const nameHint = suggested || anchor.download || '';
        handleDownloadRequest(href, nameHint || null, '');
    }, true);

    // 2) Monkey-patch <a>.click() para pegar downloads programáticos
    // (ex.: chat.qwen.ai cria <a download> temporário e chama .click())
    (function patchAnchorClick() {
        const origClick = HTMLAnchorElement.prototype.click;
        HTMLAnchorElement.prototype.click = function() {
            try {
                const href = this.getAttribute('href') || this.href || '';
                const hasDownload = this.hasAttribute('download');
                const isBlob = href.startsWith('blob:');
                const isData = href.startsWith('data:');
                if ((hasDownload || isBlob || isData) && href) {
                    const suggested = hasDownload ? (this.getAttribute('download') || this.download || '') : '';
                    // async intercept, don't call original
                    handleDownloadRequest(href, suggested || null, '');
                    return;
                }
            } catch (err) {
                console.warn('[Qwen Studio] patchAnchorClick check failed', err);
            }
            return origClick.apply(this, arguments);
        };
    })();

    // API pública para site/scripts chamarem manualmente se necessário
    window.__qwenSaveBlob = async function(blob, filename) {
        if (!(blob instanceof Blob)) {
            console.warn('[Qwen Studio] __qwenSaveBlob espera um Blob');
            return;
        }
        const name = sanitizeFilenameForDownload(filename || blob.name || 'download');
        const b64 = await blobToBase64(blob);
        const invoke = window.__TAURI__?.core?.invoke;
        if (!invoke) throw new Error('Tauri invoke unavailable');
        return invoke('save_downloaded_file', { filename: name, mime: blob.type || 'application/octet-stream', dataBase64: b64 });
    };
    window.__qwenSaveUrl = function(url, filename) {
        return handleDownloadRequest(url, filename || null, '');
    };

    window.open = function(url, target, features) {
        if (url && typeof url === 'string' && url.includes('qwenlm.io')) {
            if (window.__TAURI__?.core?.invoke) {
                window.__TAURI__.core.invoke('open_preview_window', { url }).catch(() => {});
            } else if (window.electronAPI?.open_external_link) {
                window.electronAPI.open_external_link(url);
            }
            return { closed: false, close: function(){}, postMessage: function(){}, focus: function(){}, closed:false };
        }
        if (url && (url.startsWith('http://') || url.startsWith('https://'))) {
            window.electronAPI?.open_external_link?.(url);
        }
        return null;
    };

    if(window.__QWEN_DEBUG) console.log('[Qwem Studio] Platform bridge loaded');
})();
