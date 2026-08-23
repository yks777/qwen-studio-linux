const invoke = window.__TAURI__.core.invoke;

const listEl = document.getElementById("profile-list");
const emptyEl = document.getElementById("empty-state");
const createBtn = document.getElementById("create-btn");
const createForm = document.getElementById("create-form");
const createInput = document.getElementById("create-input");
const createCancel = document.getElementById("create-cancel");

document.addEventListener("DOMContentLoaded", loadProfiles);

async function loadProfiles() {
    try {
        const profiles = await invoke("list_profiles");
        renderProfiles(profiles || []);
    } catch (e) {
        console.error("Erro ao carregar perfis:", e);
        renderProfiles([]);
    }
}

function renderProfiles(profiles) {
    listEl.innerHTML = "";

    if (!profiles || profiles.length === 0) {
        emptyEl.style.display = "flex";
        return;
    }
    emptyEl.style.display = "none";

    for (const p of profiles) {
        const card = document.createElement("div");
        card.className = "profile-card";
        card.dataset.id = p.id;

        const avatar = document.createElement("div");
        avatar.className = "avatar";
        avatar.textContent = (p.name || "?").trim().charAt(0).toUpperCase() || "?";

        const info = document.createElement("div");
        info.className = "profile-info";
        const nameEl = document.createElement("span");
        nameEl.className = "profile-name";
        nameEl.textContent = p.name;
        nameEl.title = p.name;
        info.appendChild(nameEl);

        const actions = document.createElement("div");
        actions.className = "profile-actions";

        const enterBtn = document.createElement("button");
        enterBtn.className = "btn btn-primary btn-sm";
        enterBtn.textContent = "Entrar";
        enterBtn.addEventListener("click", () => launchProfile(p.id, enterBtn));

        const editBtn = document.createElement("button");
        editBtn.className = "btn btn-ghost btn-sm";
        editBtn.textContent = "Editar";
        editBtn.addEventListener("click", () => startRename(card, p));

        const delBtn = document.createElement("button");
        delBtn.className = "btn btn-danger btn-sm";
        delBtn.textContent = "Excluir";
        delBtn.addEventListener("click", () => deleteProfile(p.id));

        actions.append(enterBtn, editBtn, delBtn);
        card.append(avatar, info, actions);
        listEl.appendChild(card);
    }
}

function toggleCreateForm() {
    if (createForm.classList.contains("active")) {
        createForm.classList.remove("active");
        createBtn.style.display = "inline-flex";
        createInput.value = "";
    } else {
        createForm.classList.add("active");
        createBtn.style.display = "none";
        setTimeout(() => createInput.focus(), 50);
    }
}

async function handleCreate(e) {
    e.preventDefault();
    const name = createInput.value.trim();
    if (!name) return;

    try {
        await invoke("create_profile", { name });
        toggleCreateForm();
        await loadProfiles();
    } catch (err) {
        console.error("Erro ao criar perfil:", err);
        alert("Falha ao criar perfil.");
    }
}

function startRename(card, profile) {
    const info = card.querySelector(".profile-info");
    const actions = card.querySelector(".profile-actions");

    info.innerHTML = "";
    const input = document.createElement("input");
    input.className = "input-inline";
    input.id = "rename-input-" + profile.id;
    input.value = profile.name;
    info.appendChild(input);

    actions.innerHTML = "";
    const saveBtn = document.createElement("button");
    saveBtn.className = "btn btn-primary btn-sm";
    saveBtn.textContent = "Salvar";
    saveBtn.addEventListener("click", () => saveRename(profile.id));

    const cancelBtn = document.createElement("button");
    cancelBtn.className = "btn btn-ghost btn-sm";
    cancelBtn.textContent = "Cancelar";
    cancelBtn.addEventListener("click", loadProfiles);

    actions.append(saveBtn, cancelBtn);

    input.focus();
    input.select();
    input.addEventListener("keydown", (e) => {
        if (e.key === "Enter") {
            e.preventDefault();
            saveRename(profile.id);
        } else if (e.key === "Escape") {
            e.preventDefault();
            loadProfiles();
        }
    });
}

async function saveRename(id) {
    const input = document.getElementById("rename-input-" + id);
    if (!input) return;

    const newName = input.value.trim();
    if (!newName) return;

    try {
        await invoke("rename_profile", { id, name: newName });
        await loadProfiles();
    } catch (err) {
        console.error("Erro ao renomear perfil:", err);
        alert("Falha ao renomear perfil.");
    }
}

async function deleteProfile(id) {
    if (!window.confirm("Tem certeza que deseja excluir este perfil? Esta ação não pode ser desfeita.")) {
        return;
    }
    try {
        await invoke("delete_profile", { id });
        await loadProfiles();
    } catch (err) {
        console.error("Erro ao excluir perfil:", err);
        alert("Falha ao excluir perfil.");
    }
}

async function launchProfile(id, btn) {
    try {
        await invoke("launch_profile", { id });
        if (btn) btn.textContent = "Focar";
    } catch (err) {
        console.error("Erro ao lançar perfil:", err);
        alert("Falha ao abrir o perfil.");
    }
}

// Open the creation form when requested from the app menu / tray.
try {
    window.__TAURI__.event.listen("focus-create", () => {
        if (!createForm.classList.contains("active")) {
            toggleCreateForm();
        }
        setTimeout(() => createInput.focus(), 50);
    });
} catch (e) {
    console.warn("Não foi possível registrar o listener de criação:", e);
}

createBtn.addEventListener("click", toggleCreateForm);
createCancel.addEventListener("click", toggleCreateForm);
createForm.addEventListener("submit", handleCreate);
