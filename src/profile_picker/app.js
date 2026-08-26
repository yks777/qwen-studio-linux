const invoke = window.__TAURI__.core.invoke;

const ICONS = {
    money: {
        label: 'Dinheiro',
        svg: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 6v12"/><path d="M15.5 9.5c0-1.38-1.57-2.5-3.5-2.5s-3.5 1.12-3.5 2.5 1.57 2.5 3.5 2.5 3.5 1.12 3.5 2.5-1.57 2.5-3.5 2.5"/></svg>`
    },
    code: {
        label: 'Código',
        svg: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>`
    },
    terminal: {
        label: 'Terminal',
        svg: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>`
    },
    life: {
        label: 'Vida',
        svg: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 20A7 7 0 0 1 9.8 6.9C15.5 4.9 17 3.5 17 3.5s1 7.5-5 12.5"/><path d="M11 20v-7.5"/><path d="M5.5 14.5C4 13 2 10 2 7.5 2 4 5 2 7.5 3.5c1.5 1 3 3 4.5 5"/></svg>`
    },
    briefcase: {
        label: 'Trabalho',
        svg: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="7" width="20" height="14" rx="2" ry="2"/><path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"/></svg>`
    },
    book: {
        label: 'Educação',
        svg: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/></svg>`
    },
    heart: {
        label: 'Amor',
        svg: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>`
    },
    star: {
        label: 'Favorito',
        svg: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>`
    },
};

const SEARCH_SVG = `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>`;
const EDIT_SVG = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>`;
const DELETE_SVG = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/></svg>`;
const DRAG_HANDLE_SVG = `<svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><circle cx="8" cy="4" r="2"/><circle cx="16" cy="4" r="2"/><circle cx="8" cy="12" r="2"/><circle cx="16" cy="12" r="2"/><circle cx="8" cy="20" r="2"/><circle cx="16" cy="20" r="2"/></svg>`;

const listEl = document.getElementById('profile-list');
const emptyEl = document.getElementById('empty-state');
const createBtn = document.getElementById('create-btn');
const createForm = document.getElementById('create-form');
const createName = document.getElementById('create-name');
const createCategory = document.getElementById('create-category');
const createCategorySuggestions = document.getElementById('create-category-suggestions');
const createIconPicker = document.getElementById('create-icon-picker');
const createCancel = document.getElementById('create-cancel');
const searchInput = document.getElementById('search-input');
const categoryTabs = document.getElementById('category-tabs');
const editForm = document.getElementById('edit-form');
const editId = document.getElementById('edit-id');
const editName = document.getElementById('edit-name');
const editCategory = document.getElementById('edit-category');
const editCategorySuggestions = document.getElementById('edit-category-suggestions');
const editIconPicker = document.getElementById('edit-icon-picker');
const editCancel = document.getElementById('edit-cancel');

let allProfiles = [];
let allCategories = [];
let activeCategory = null;
let selectedCreateIcon = null;
let selectedEditIcon = null;

let isDragging = false;
let dragGhost = null;
let dragSourceId = null;
let dropIndicator = null;
let currentDropTarget = null;

document.addEventListener('DOMContentLoaded', init);

async function init() {
    renderIconPicker(createIconPicker, 'create');
    renderIconPicker(editIconPicker, 'edit');
    await loadProfiles();
    setupSearch();
    setupCategoryInput(createCategory, createCategorySuggestions, 'create');
    setupCategoryInput(editCategory, editCategorySuggestions, 'edit');
}

async function loadProfiles() {
    try {
        allProfiles = await invoke('list_profiles');
        allCategories = await invoke('list_categories');
        renderCategoryTabs();
        renderFilteredProfiles();
    } catch (e) {
        console.error('Erro ao carregar perfis:', e);
        allProfiles = [];
        allCategories = [];
        renderCategoryTabs();
        renderFilteredProfiles();
    }
}

function getFilteredProfiles() {
    const query = (searchInput.value || '').trim().toLowerCase();
    let filtered = allProfiles;

    if (activeCategory !== null) {
        if (activeCategory === '') {
            filtered = filtered.filter(p => !p.category);
        } else {
            filtered = filtered.filter(p => p.category === activeCategory);
        }
    }

    if (query) {
        filtered = filtered.filter(p => (p.name || '').toLowerCase().includes(query));
    }

    return filtered;
}

function renderFilteredProfiles() {
    renderProfiles(getFilteredProfiles());
}

function renderCategoryTabs() {
    categoryTabs.innerHTML = '';

    const totalCount = allProfiles.length;
    const allPill = createCategoryPill('Todos', null, null, totalCount, true);
    categoryTabs.appendChild(allPill);

    const uncategorizedCount = allProfiles.filter(p => !p.category).length;
    if (uncategorizedCount > 0) {
        const uncategorizedPill = createCategoryPill('Sem categoria', '', null, uncategorizedCount, false);
        categoryTabs.appendChild(uncategorizedPill);
    }

    for (const cat of allCategories) {
        const catProfiles = allProfiles.filter(p => p.category === cat);
        const icon = catProfiles.length > 0 ? catProfiles[0].icon : null;
        const pill = createCategoryPill(cat, cat, icon, catProfiles.length, false);
        categoryTabs.appendChild(pill);
    }
}

function createCategoryPill(label, value, icon, count, isActive) {
    const pill = document.createElement('button');
    pill.type = 'button';
    pill.className = 'category-pill' + ((isActive && activeCategory === null) || activeCategory === value ? ' active' : '');

    if (icon && ICONS[icon]) {
        const iconSpan = document.createElement('span');
        iconSpan.className = 'pill-icon';
        iconSpan.innerHTML = ICONS[icon].svg;
        pill.appendChild(iconSpan);
    }

    const labelSpan = document.createElement('span');
    labelSpan.textContent = label;
    pill.appendChild(labelSpan);

    const countSpan = document.createElement('span');
    countSpan.className = 'pill-count';
    countSpan.textContent = count;
    pill.appendChild(countSpan);

    pill.addEventListener('click', () => {
        activeCategory = value;
        renderCategoryTabs();
        renderFilteredProfiles();
    });

    return pill;
}

function isFilteredView() {
    const query = (searchInput.value || '').trim();
    return activeCategory !== null || query !== '';
}

function renderProfiles(profiles) {
    listEl.innerHTML = '';

    if (!profiles || profiles.length === 0) {
        emptyEl.style.display = 'flex';
        return;
    }
    emptyEl.style.display = 'none';

    const filtered = isFilteredView();

    for (const p of profiles) {
        const card = document.createElement('div');
        card.className = 'profile-card';
        card.dataset.id = p.id;

        const handle = document.createElement('div');
        handle.className = 'drag-handle' + (filtered ? ' disabled' : '');
        handle.innerHTML = DRAG_HANDLE_SVG;
        if (filtered) {
            handle.title = 'Limpe a busca/filtro para reordenar';
        }

        if (!filtered) {
            handle.addEventListener('mousedown', (e) => {
                e.preventDefault();
                e.stopPropagation();
                startDrag(p.id, card, e);
            });
        }

        const icon = document.createElement('div');
        icon.className = 'profile-icon' + (p.icon ? '' : ' no-icon');
        if (p.icon && ICONS[p.icon]) {
            icon.dataset.icon = p.icon;
            icon.innerHTML = ICONS[p.icon].svg;
        } else {
            icon.textContent = (p.name || '?').trim().charAt(0).toUpperCase() || '?';
        }

        const info = document.createElement('div');
        info.className = 'profile-info';

        const nameEl = document.createElement('span');
        nameEl.className = 'profile-name';
        nameEl.textContent = p.name;
        nameEl.title = p.name;
        info.appendChild(nameEl);

        if (p.category) {
            const catEl = document.createElement('span');
            catEl.className = 'profile-category';
            const catIcon = document.createElement('span');
            catIcon.className = 'cat-icon';
            const catProfiles = allProfiles.filter(pr => pr.category === p.category);
            const catIconId = catProfiles.length > 0 ? catProfiles[0].icon : null;
            if (catIconId && ICONS[catIconId]) {
                catIcon.innerHTML = ICONS[catIconId].svg;
            }
            const catText = document.createTextNode(p.category);
            catEl.append(catIcon, catText);
            info.appendChild(catEl);
        }

        const actions = document.createElement('div');
        actions.className = 'profile-actions';

        const enterBtn = document.createElement('button');
        enterBtn.className = 'btn btn-primary btn-sm';
        enterBtn.textContent = 'Entrar';
        enterBtn.addEventListener('click', () => launchProfile(p.id, enterBtn));

        const editBtn = document.createElement('button');
        editBtn.className = 'btn btn-ghost btn-sm btn-icon';
        editBtn.innerHTML = EDIT_SVG;
        editBtn.title = 'Editar';
        editBtn.addEventListener('click', () => openEditForm(p));

        const delBtn = document.createElement('button');
        delBtn.className = 'btn btn-danger btn-sm btn-icon';
        delBtn.innerHTML = DELETE_SVG;
        delBtn.title = 'Excluir';
        delBtn.addEventListener('click', () => deleteProfile(p.id));

        actions.append(enterBtn, editBtn, delBtn);
        card.append(handle, icon, info, actions);
        listEl.appendChild(card);
    }
}

function startDrag(profileId, sourceCard, e) {
    isDragging = true;
    dragSourceId = profileId;

    dragGhost = sourceCard.cloneNode(true);
    dragGhost.className = 'profile-card drag-ghost';
    dragGhost.style.width = sourceCard.offsetWidth + 'px';
    document.body.appendChild(dragGhost);
    moveGhost(e);

    dropIndicator = document.createElement('div');
    dropIndicator.className = 'drop-indicator';
    dropIndicator.style.display = 'none';
    listEl.appendChild(dropIndicator);

    sourceCard.classList.add('dragging');
    document.body.style.userSelect = 'none';

    document.addEventListener('mousemove', onDragMove);
    document.addEventListener('mouseup', onDragEnd);
}

function moveGhost(e) {
    if (!dragGhost) return;
    dragGhost.style.left = (e.clientX - 50) + 'px';
    dragGhost.style.top = (e.clientY - 20) + 'px';
}

function onDragMove(e) {
    if (!isDragging) return;
    moveGhost(e);

    const cards = [...listEl.querySelectorAll('.profile-card:not(.drag-ghost):not(.dragging)')];
    if (cards.length === 0) return;
    const listRect = listEl.getBoundingClientRect();

    const firstRect = cards[0].getBoundingClientRect();
    const lastRect = cards[cards.length - 1].getBoundingClientRect();

    // Antes do primeiro card
    if (e.clientY < firstRect.top) {
        currentDropTarget = { id: cards[0].dataset.id, position: 'before' };
        dropIndicator.style.display = 'block';
        dropIndicator.style.top = (firstRect.top - listRect.top - 1) + 'px';
        dropIndicator.style.left = '0';
        dropIndicator.style.right = '0';
        return;
    }

    // Depois do último card
    if (e.clientY > lastRect.bottom) {
        currentDropTarget = { id: cards[cards.length - 1].dataset.id, position: 'after' };
        dropIndicator.style.display = 'block';
        dropIndicator.style.top = (lastRect.bottom - listRect.top - 1) + 'px';
        dropIndicator.style.left = '0';
        dropIndicator.style.right = '0';
        return;
    }

    for (let i = 0; i < cards.length; i++) {
        const card = cards[i];
        const rect = card.getBoundingClientRect();

        // Dentro do card: decide pela metade
        if (e.clientY >= rect.top && e.clientY <= rect.bottom) {
            const midY = rect.top + rect.height / 2;
            const position = e.clientY < midY ? 'before' : 'after';
            currentDropTarget = { id: card.dataset.id, position };
            dropIndicator.style.display = 'block';
            if (position === 'before') {
                dropIndicator.style.top = (rect.top - listRect.top - 1) + 'px';
            } else {
                dropIndicator.style.top = (rect.bottom - listRect.top - 1) + 'px';
            }
            dropIndicator.style.left = '0';
            dropIndicator.style.right = '0';
            return;
        }

        // No gap entre este card e o próximo
        if (i < cards.length - 1) {
            const nextRect = cards[i + 1].getBoundingClientRect();
            if (e.clientY > rect.bottom && e.clientY < nextRect.top) {
                // Gap: inserir após o atual (equivale a antes do próximo)
                currentDropTarget = { id: card.dataset.id, position: 'after' };
                dropIndicator.style.display = 'block';
                dropIndicator.style.top = (rect.bottom - listRect.top - 1) + 'px';
                dropIndicator.style.left = '0';
                dropIndicator.style.right = '0';
                return;
            }
        }
    }

    dropIndicator.style.display = 'none';
    currentDropTarget = null;
}

function onDragEnd() {
    document.removeEventListener('mousemove', onDragMove);
    document.removeEventListener('mouseup', onDragEnd);

    isDragging = false;
    document.body.style.userSelect = '';

    listEl.querySelectorAll('.profile-card.dragging').forEach(el => {
        el.classList.remove('dragging');
    });

    if (dragGhost) { dragGhost.remove(); dragGhost = null; }
    if (dropIndicator) { dropIndicator.remove(); dropIndicator = null; }

    if (currentDropTarget && dragSourceId && currentDropTarget.id !== dragSourceId) {
        applyReorder(dragSourceId, currentDropTarget.id, currentDropTarget.position);
    }

    dragSourceId = null;
    currentDropTarget = null;
}

async function applyReorder(draggedId, targetId, position) {
    const fromIdx = allProfiles.findIndex(p => p.id === draggedId);
    let toIdx = allProfiles.findIndex(p => p.id === targetId);
    if (fromIdx === -1 || toIdx === -1 || fromIdx === toIdx) return;

    const [moved] = allProfiles.splice(fromIdx, 1);

    toIdx = allProfiles.findIndex(p => p.id === targetId);
    if (position === 'after') toIdx += 1;

    allProfiles.splice(toIdx, 0, moved);
    renderFilteredProfiles();

    const orderedIds = allProfiles.map(p => p.id);
    try {
        await invoke('reorder_profiles', { orderedIds });
    } catch (err) {
        console.error('Erro ao reordenar perfis:', err);
        alert('Falha ao reordenar: ' + err);
        await loadProfiles();
    }
}

function setupSearch() {
    searchInput.addEventListener('input', () => {
        renderFilteredProfiles();
    });
}

function setupCategoryInput(input, suggestionsEl, mode) {
    input.addEventListener('input', () => {
        const val = input.value.trim().toLowerCase();
        const matching = allCategories.filter(c => c.toLowerCase().includes(val));

        suggestionsEl.innerHTML = '';

        if (val && !allCategories.some(c => c.toLowerCase() === val)) {
            const createItem = document.createElement('div');
            createItem.className = 'category-suggestion create-new';
            createItem.textContent = `Criar "${input.value.trim()}"`;
            createItem.addEventListener('mousedown', (e) => {
                e.preventDefault();
                input.value = input.value.trim();
                suggestionsEl.classList.remove('active');
            });
            suggestionsEl.appendChild(createItem);
        }

        for (const cat of matching) {
            const item = document.createElement('div');
            item.className = 'category-suggestion';
            item.textContent = cat;
            item.addEventListener('mousedown', (e) => {
                e.preventDefault();
                input.value = cat;
                suggestionsEl.classList.remove('active');
            });
            suggestionsEl.appendChild(item);
        }

        if (suggestionsEl.children.length > 0) {
            suggestionsEl.classList.add('active');
        } else {
            suggestionsEl.classList.remove('active');
        }
    });

    input.addEventListener('blur', () => {
        setTimeout(() => suggestionsEl.classList.remove('active'), 150);
    });

    input.addEventListener('focus', () => {
        input.dispatchEvent(new Event('input'));
    });
}

function renderIconPicker(container, mode) {
    container.innerHTML = '';
    for (const [id, info] of Object.entries(ICONS)) {
        const opt = document.createElement('div');
        opt.className = 'icon-option';
        opt.dataset.icon = id;

        const glyph = document.createElement('span');
        glyph.className = 'icon-glyph';
        glyph.innerHTML = info.svg;

        const label = document.createElement('span');
        label.className = 'icon-label';
        label.textContent = info.label;

        opt.append(glyph, label);

        opt.addEventListener('click', () => {
            container.querySelectorAll('.icon-option').forEach(el => el.classList.remove('selected'));
            if (mode === 'create') {
                if (selectedCreateIcon === id) {
                    selectedCreateIcon = null;
                } else {
                    selectedCreateIcon = id;
                    opt.classList.add('selected');
                }
            } else {
                if (selectedEditIcon === id) {
                    selectedEditIcon = null;
                } else {
                    selectedEditIcon = id;
                    opt.classList.add('selected');
                }
            }
        });

        container.appendChild(opt);
    }
}

function toggleCreateForm() {
    if (createForm.classList.contains('active')) {
        createForm.classList.remove('active');
        createBtn.style.display = 'inline-flex';
        createName.value = '';
        createCategory.value = '';
        selectedCreateIcon = null;
        createIconPicker.querySelectorAll('.icon-option').forEach(el => el.classList.remove('selected'));
    } else {
        editForm.classList.remove('active');
        createForm.classList.add('active');
        createBtn.style.display = 'none';
        setTimeout(() => createName.focus(), 50);
    }
}

async function handleCreate(e) {
    e.preventDefault();
    const name = createName.value.trim();
    if (!name) return;

    const category = createCategory.value.trim() || null;
    const icon = selectedCreateIcon || null;

    try {
        await invoke('create_profile', { name, category, icon });
        toggleCreateForm();
        await loadProfiles();
    } catch (err) {
        console.error('Erro ao criar perfil:', err);
        alert('Falha ao criar perfil.');
    }
}

function openEditForm(profile) {
    createForm.classList.remove('active');
    createBtn.style.display = 'inline-flex';

    editId.value = profile.id;
    editName.value = profile.name;
    editCategory.value = profile.category || '';
    selectedEditIcon = profile.icon || null;

    editIconPicker.querySelectorAll('.icon-option').forEach(el => {
        el.classList.toggle('selected', el.dataset.icon === selectedEditIcon);
    });

    editForm.classList.add('active');
    setTimeout(() => editName.focus(), 50);
}

async function handleEdit(e) {
    e.preventDefault();
    const id = editId.value;
    const name = editName.value.trim();
    if (!name || !id) return;

    const category = editCategory.value.trim() || null;
    const icon = selectedEditIcon || null;

    try {
        await invoke('update_profile', { id, name, category, icon });
        editForm.classList.remove('active');
        await loadProfiles();
    } catch (err) {
        console.error('Erro ao atualizar perfil:', err);
        alert('Falha ao atualizar perfil.');
    }
}

function closeEditForm() {
    editForm.classList.remove('active');
}

async function deleteProfile(id) {
    if (!window.confirm('Tem certeza que deseja excluir este perfil? Esta ação não pode ser desfeita.')) {
        return;
    }
    try {
        await invoke('delete_profile', { id });
        await loadProfiles();
    } catch (err) {
        console.error('Erro ao excluir perfil:', err);
        alert('Falha ao excluir perfil.');
    }
}

async function launchProfile(id, btn) {
    try {
        await invoke('launch_profile', { id });
        if (btn) btn.textContent = 'Focar';
    } catch (err) {
        console.error('Erro ao lançar perfil:', err);
        alert('Falha ao abrir o perfil.');
    }
}

createBtn.addEventListener('click', toggleCreateForm);
createCancel.addEventListener('click', toggleCreateForm);
createForm.addEventListener('submit', handleCreate);

editCancel.addEventListener('click', closeEditForm);
editForm.addEventListener('submit', handleEdit);

try {
    window.__TAURI__.event.listen('focus-create', () => {
        if (!createForm.classList.contains('active')) {
            toggleCreateForm();
        }
        setTimeout(() => createName.focus(), 50);
    });
} catch (e) {
    console.warn('Não foi possível registrar o listener de criação:', e);
}
