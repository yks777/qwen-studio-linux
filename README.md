# Qwen Studio Linux

<div align="center">
  <a href="#qwen-studio-linux"><img src="https://img.shields.io/badge/🇺🇲-English-blue?style=for-the-badge" alt="English"></a>
  <a href="#qwen-studio-linux--português"><img src="https://img.shields.io/badge/🇧🇷-Português-green?style=for-the-badge" alt="Português"></a>
</div>

---

<div align="center">
  <img src="icons/icon.png" alt="Logo" width="150" height="150">
</div>

**Qwen Studio Linux** is a native desktop wrapper for [Qwen Chat](https://chat.qwen.ai) on Linux, built with [Tauri v2](https://v2.tauri.app/) and WebKitGTK. No Electron, no Node.js in the main process — just a lightweight WebView with native system integrations.

> ⚠️ Unofficial project, not affiliated with Alibaba. It is an independent desktop client that loads Qwen Chat in a WebView with native integrations.

---

## Features

- 💬 **Native chat** — opens `chat.qwen.ai` in a WebKitGTK WebView optimized for desktop.
- 👥 **Multiple profiles** — separate, isolated sessions (cookies, `localStorage` and data directory) for different accounts, managed via a profile picker and the "Perfils" menu.
- 🔐 **OAuth login via deep link** — authenticate in your browser and return automatically via the `qwen://` protocol.
- 🧩 **Model Context Protocol (MCP)** — MCP servers (qwen-core, Filesystem, Sequential-Thinking) managed through a Node.js bridge, with stdio and HTTP/SSE transports.
- 🖥️ **System tray + HeaderBar menu** — notification-area icon and a native GTK menu (Linux-specific).
- 🔄 **Auto-update** — integrated update checking and installation (with an "Updates" tab injected into Settings).
- 🪟 **Multiple windows** — open several independent chat windows at once.
- 🎨 **Native integrations** — clipboard (including images), native file/confirm dialogs, theme switching, drag-and-drop of files, image paste, external-link handling, desktop notifications, and conversation export.
- 🔍 **Zoom & DevTools** — zoom controls (`Ctrl +`/`-`/`0` and `Ctrl + scroll`) and a toggleable WebView DevTools.
- 🛡️ **Crash diagnostics** — panics are logged to disk for easier bug reporting.

---

## System requirements

- **Linux** (x86_64 recommended)
- **GTK 3**
- **WebKitGTK 4.1**
- **AppIndicator** (for the tray icon)

### Runtime dependencies

| Distribution | Packages |
|--------------|---------|
| Debian / Ubuntu | `libwebkit2gtk-4.1-0`, `libgtk-3-0`, `libayatana-appindicator3-1` |
| Fedora / RHEL | `webkit2gtk4.1`, `gtk3`, `libappindicator-gtk3` |

---

## Download

Prebuilt binaries (`.deb`, `.rpm`, `.AppImage`) for Linux are on [GitHub Releases](https://github.com/NicolasToledoo/qwen-studio-linux/releases/latest). Download the package for your distro and follow the steps in [Installation](#installation).

---

## Installation

### Debian / Ubuntu (`.deb`)

```bash
sudo apt install libwebkit2gtk-4.1-0 libgtk-3-0 libayatana-appindicator3-1
sudo dpkg -i qwen-studio-linux_*.deb
```

### Fedora / RHEL (`.rpm`)

```bash
sudo dnf install webkit2gtk4.1 gtk3 libappindicator-gtk3
sudo rpm -i qwen-studio-linux-*.rpm
```

### Universal (`.AppImage`)

```bash
chmod +x Qwen-Studio-Linux-*.AppImage
./Qwen-Studio-Linux-*.AppImage
```

After installing, the `Qwen Studio Linux` shortcut appears in your applications menu and the `qwen://` deep-link handler is registered automatically.

---

## Build from source

### Prerequisites

- [Rust](https://www.rust-lang.org/) (edition 2021)
- [Node.js](https://nodejs.org/) + npm
- Development libraries:

  ```bash
  # Debian / Ubuntu
  sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev

  # Fedora / RHEL
  sudo dnf install webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel
  ```

### Steps

> ⚠️ **Required step**: the repository does not include `node_modules/` (it's in `.gitignore`), so running `npm install` before any `npm run tauri:*` command is **mandatory**. Without it you'll get `tauri: command not found`.

```bash
npm install
```

The `npm install` also installs the Tauri CLI (`@tauri-apps/cli`) locally. Then build the desired package:

```bash
npm run tauri:build:deb        # Debian / Ubuntu
npm run tauri:build:rpm        # Fedora / RHEL
npm run tauri:build:appimage   # Universal
npm run tauri:build            # all formats
```

Binaries are placed in `target/release/bundle/`.

> The `WEBKIT_DISABLE_COMPOSITING_MODE=1` and `WEBKIT_DISABLE_DMABUF_RENDERER=1` environment variables are already included in the scripts to avoid blank screens on some drivers.

---

## Usage

1. Open **Qwen Studio Linux**.
2. A **profile picker** opens. Create a profile (or pick an existing one) — each profile is an isolated session.
3. Click **Log in** — your default browser opens the Qwen OAuth screen.
4. After authorizing, the app receives the token via `qwen://open?token=xxx`, injects cookies + `localStorage` into the WebView and navigates to the chat automatically.
5. Manage MCP servers and preferences in **Settings**.

---

## Profiles

Qwen Studio Linux supports multiple independent accounts through **profiles**.

- On launch, a **profile picker** window appears where you can **create**, **rename**, **delete** and **launch** profiles.
- Each profile stores its own login (cookies), `localStorage` and WebView data in an isolated directory: `~/.config/qwen-studio-linux/profiles/<id>/`.
- Multiple profiles can run **simultaneously**, each in its own window.
- The **Perfils** submenu in the app menu lets you open the picker, switch between profiles, and create new ones.

---

## Model Context Protocol (MCP)

The app communicates with MCP servers through a Node.js bridge (`mcp-bridge.mjs`) that uses `@modelcontextprotocol/sdk`. Rust spawns the process and exchanges NDJSON messages over stdin/stdout (each request has a unique ID and a 60s timeout).

Servers enabled by default (created on first run):

- **qwen-core** — main Qwen integration
- **Filesystem** — local file access
- **Sequential-Thinking** — step-by-step reasoning

### Transports

MCP servers can be configured with two transports:

- **stdio** (default) — the server is launched as a subprocess via `command`/`args`.
- **HTTP / SSE** — connect to a running server over the network using `transportType: "http"` (or `"sse"`) together with the `url` field.

### Add / edit servers

Edit the configuration file (see below) or use the Settings UI. Example `settings.json`:

```json
{
  "mcpServers": {
    "Filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/home/you/Docs"]
    },
    "Sequential-Thinking": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-sequential-thinking"]
    },
    "Remote-Server": {
      "transportType": "http",
      "url": "http://localhost:3000/mcp"
    }
  },
  "general": {
    "check_updates": true
  }
}
```

---

## Application Menu & Keyboard Shortcuts

The app provides a native GTK HeaderBar menu (Linux) with the following structure:

| Menu | Items |
|------|-------|
| File | Minimize, Maximize, Quit |
| Edit | Undo, Redo, Cut, Copy, Paste, Select All |
| View | Reload, Toggle DevTools, Zoom In, Zoom Out, Reset Zoom |
| Window | New Window, Fullscreen |
| Perfils | Open picker, open/launch a profile, create profile |
| Help | Documentation, GitHub, Check for Updates |

Keyboard shortcuts:

| Shortcut | Action |
|----------|--------|
| `Ctrl + N` | New window |
| `Ctrl + W` | Close window |
| `Ctrl + R` / `F5` | Reload |
| `Ctrl + Shift + I` | Toggle DevTools |
| `Ctrl + =` / `Ctrl + +` | Zoom in |
| `Ctrl + -` | Zoom out |
| `Ctrl + 0` | Reset zoom |
| `Ctrl + scroll wheel` | Zoom in / out (range 0.5×–2.0×) |

---

## Native Integrations

- **Clipboard** — copy/paste text; paste images directly from the clipboard into the chat.
- **Drag & drop** — drop files from your file manager into the WebView to send them.
- **External links** — links opened via `window.open` are routed: authentication URLs stay inside the WebView, while other `http(s)` links open in your default browser.
- **Desktop notifications** — the app shows a native notification (via libnotify) when an update is available.
- **System theme** — the app follows the system light/dark theme.
- **Native dialogs** — file pickers and confirmation dialogs use the OS native widgets.
- **Conversation export** — export chat conversations from the UI.

---

## Configuration

Location: `~/.config/qwen-studio-linux/settings.json`

Profiles, logins (cookies) and `localStorage` are stored under `~/.config/qwen-studio-linux/profiles/` (one directory per profile). This location is user-specific and works on any machine — including when the app is distributed as an AppImage. No profile data is written inside the project folder or the AppImage.

```json
{
  "mcpServers": { "<name>": { "command": "...", "args": [...] } },
  "general": { "check_updates": true, "theme": "" }
}
```

- `mcpServers` — map of MCP servers (command, args, env, cwd, transportType, url, timeout).
- `general.check_updates` — enable/disable automatic update checks.
- `general.theme` — UI theme preference.

---

## Updates

The app uses the `tauri-plugin-updater` plugin. GitHub releases must include `latest.json` for the updater to work. The **Updates** tab is injected dynamically into Settings and also shows a global banner when a new version is available.

---

## Crash Reports

If the app panics, a crash report is written automatically to:

```
~/.local/share/qwen-studio-linux/crash-logs/crash-<timestamp>.log
```

Each file contains the time, version, platform, panic message, location and (when `RUST_BACKTRACE=1`) a backtrace. These logs are useful when reporting bugs.

---

## Troubleshooting

| Symptom | Cause / Solution |
|---------|-----------------|
| `tauri: command not found` | You didn't run `npm install` in the project folder. `node_modules/` isn't included in the GitHub clone. Run `npm install` and try again. |
| Blank screen / WebView doesn't render | Set `WEBKIT_DISABLE_COMPOSITING_MODE=1 WEBKIT_DISABLE_DMABUF_RENDERER=1` (already in the npm scripts). |
| App won't open under Wayland | Force X11 with `GDK_BACKEND=x11` (done automatically at startup). |
| Deep link `qwen://` doesn't open the app | Make sure `qwen-studio-linux.desktop` (handler `x-scheme-handler/qwen`) is installed and registered. |
| MCP server "offline" | Enable it in MCP Settings or check the `command`/`args` in `settings.json`. |
| App crashed / unexpected behavior | Check the crash logs at `~/.local/share/qwen-studio-linux/crash-logs/` and attach the latest `crash-*.log` when reporting. |

---

## Project structure

```
qwen-studio-linux/
├── src/                  # Rust code (lib.rs, mcp/, app/, platform/, ipc/ ...)
├── capabilities/         # Tauri permission capabilities
├── permissions/          # Custom permission sets
├── icons/                # Icons for the bundle
├── gen/                  # Tauri-generated types
├── mcp-bridge.mjs        # Node.js MCP servers proxy (embedded resource)
├── tauri.conf.json       # Tauri configuration
├── Cargo.toml            # Rust dependencies
├── build.rs              # Tauri build script
├── qwen-studio-linux.desktop  # Linux desktop entry
└── package.json          # npm scripts for the Tauri CLI
```

### How it works

1. Tauri loads `chat.qwen.ai` in a WebKitGTK WebView.
2. `core_bridge.js` is injected as an initialization script, creating `window.electronAPI` so the web app thinks it runs in Electron.
3. JS calls use `window.__TAURI__.invoke()`, which routes to Rust commands in `lib.rs`.
4. Rust manages MCP servers via `tauri-plugin-mcp-bridge` → `mcp-bridge.mjs`.
5. The Settings screen is an SPA; the Updates tab is injected via JS polling.
6. Navigation is sandboxed: only `chat.qwen.ai` and known auth domains (Google/Alibaba OAuth) are allowed.

---

## Development

```bash
npm run tauri:dev
```

Project policies:

- **Zero warnings**: `cargo clippy -- -D warnings` must pass.
- **Versioning**: bump the version in `Cargo.toml` **and** `tauri.conf.json`.
- `node_modules/`, `dist/`, `target/` and build assets are in `.gitignore`.

---

## License

Distributed under the **MIT** license. See `Cargo.toml`.

---

---

# Qwen Studio Linux — Português

<div align="center">
  <img src="icons/icon.png" alt="Logo" width="150" height="150">
</div>

**Qwen Studio Linux** é um wrapper desktop nativo do [Qwen Chat](https://chat.qwen.ai) para Linux, construído com [Tauri v2](https://v2.tauri.app/) e WebKitGTK. Sem Electron, sem Node.js no processo principal — apenas um WebView leve com recursos nativos do sistema.

> ⚠️ Projeto não oficial e não afiliado à Alibaba. É um cliente de desktop independente que carrega o Qwen Chat em um WebView com integrações nativas.

---

## Funcionalidades

- 💬 **Chat nativo** — abre o `chat.qwen.ai` em um WebView WebKitGTK otimizado para desktop.
- 👥 **Múltiplos perfis** — sessões separadas e isoladas (cookies, `localStorage` e diretório de dados) para contas diferentes, gerenciadas por um seletor de perfis e pelo menu "Perfils".
- 🔐 **Login OAuth via deep link** — autenticação pelo navegador e retorno automático via protocolo `qwen://`.
- 🧩 **Model Context Protocol (MCP)** — servidores MCP (qwen-core, Filesystem, Sequential-Thinking) gerenciados por uma ponte Node.js, com transportes stdio e HTTP/SSE.
- 🖥️ **System tray + menu HeaderBar** — integração com a área de notificação e barra de título do GTK (específico do Linux).
- 🔄 **Auto-update** — verificação e instalação de atualizações integradas (com aba "Updates" injetada nas Configurações).
- 🪟 **Múltiplas janelas** — abra várias janelas de chat independentes ao mesmo tempo.
- 🎨 **Recursos nativos** — clipboard (incluindo imagens), diálogos nativos de arquivo/confirmação, troca de tema, arrastar-e-soltar de arquivos, colar imagens, tratamento de links externos, notificações desktop e exportação de conversas.
- 🔍 **Zoom & DevTools** — controles de zoom (`Ctrl +`/`-`/`0` e `Ctrl + scroll`) e um DevTools do WebView alternável.
- 🛡️ **Diagnóstico de falhas** — panics são registrados em disco para facilitar o reporte de bugs.

---

## Requisitos do sistema

- **Linux** (x86_64 recomendado)
- **GTK 3**
- **WebKitGTK 4.1**
- **AppIndicator** (para o ícone na bandeja)

### Dependências de runtime

| Distribuição | Pacotes |
|--------------|---------|
| Debian / Ubuntu | `libwebkit2gtk-4.1-0`, `libgtk-3-0`, `libayatana-appindicator3-1` |
| Fedora / RHEL | `webkit2gtk4.1`, `gtk3`, `libappindicator-gtk3` |

---

## Download

Binários prontos (`.deb`, `.rpm`, `.AppImage`) para Linux estão em [GitHub Releases](https://github.com/NicolasToledoo/qwen-studio-linux/releases/latest). Baixe o formato da sua distro e siga os passos em [Instalação](#instalação).

---

## Instalação

### Debian / Ubuntu (`.deb`)

```bash
sudo apt install libwebkit2gtk-4.1-0 libgtk-3-0 libayatana-appindicator3-1
sudo dpkg -i qwen-studio-linux_*.deb
```

### Fedora / RHEL (`.rpm`)

```bash
sudo dnf install webkit2gtk4.1 gtk3 libappindicator-gtk3
sudo rpm -i qwen-studio-linux-*.rpm
```

### Universal (`.AppImage`)

```bash
chmod +x Qwen-Studio-Linux-*.AppImage
./Qwen-Studio-Linux-*.AppImage
```

Após instalar, o atalho `Qwen Studio Linux` aparece no menu de aplicativos e o handler do deep link `qwen://` é registrado automaticamente.

---

## Build a partir do código-fonte

### Pré-requisitos

- [Rust](https://www.rust-lang.org/) (edition 2021)
- [Node.js](https://nodejs.org/) + npm
- Bibliotecas de desenvolvimento:

  ```bash
  # Debian / Ubuntu
  sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev

  # Fedora / RHEL
  sudo dnf install webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel
  ```

### Passos

> ⚠️ **Passo obrigatório**: o repositório não inclui `node_modules/` (está no `.gitignore`), então é **obrigatório** rodar `npm install` antes de qualquer comando `npm run tauri:*`. Sem ele, aparece o erro `tauri: comando não encontrado`.

```bash
npm install
```

O `npm install` também instala a CLI do Tauri (`@tauri-apps/cli`) localmente. Depois, gere o pacote desejado:

```bash
npm run tauri:build:deb        # Debian / Ubuntu
npm run tauri:build:rpm        # Fedora / RHEL
npm run tauri:build:appimage   # Universal
npm run tauri:build            # todos os formatos
```

Os binários ficam em `target/release/bundle/`.

> As variáveis de ambiente `WEBKIT_DISABLE_COMPOSITING_MODE=1` e `WEBKIT_DISABLE_DMABUF_RENDERER=1` já estão incluídas nos scripts para evitar telas em branco em alguns drivers.

---

## Uso

1. Abra o **Qwen Studio Linux**.
2. Um **seletor de perfis** abre. Crie um perfil (ou escolha um existente) — cada perfil é uma sessão isolada.
3. Clique em **Entrar** — o navegador padrão abre a tela OAuth do Qwen.
4. Após autorizar, o app recebe o token via `qwen://open?token=xxx`, injeta cookies + `localStorage` no WebView e navega para o chat automaticamente.
5. Gerencie servidores MCP e preferências na tela de **Configurações**.

---

## Perfis

O Qwen Studio Linux suporta múltiplas contas independentes através de **perfis**.

- Na inicialização, uma janela de **seletor de perfis** aparece, onde você pode **criar**, **renomear**, **excluir** e **abrir** perfis.
- Cada perfil armazena seu próprio login (cookies), `localStorage` e dados do WebView em um diretório isolado: `~/.config/qwen-studio-linux/profiles/<id>/`.
- Vários perfis podem rodar **simultaneamente**, cada um em sua própria janela.
- O submenu **Perfils** no menu do aplicativo permite abrir o seletor, alternar entre perfis e criar novos.

---

## Model Context Protocol (MCP)

O app se comunica com servidores MCP através de uma ponte Node.js (`mcp-bridge.mjs`) que usa o `@modelcontextprotocol/sdk`. O Rust spawna o processo e troca mensagens NDJSON via stdin/stdout (cada requisição tem um ID único e timeout de 60s).

Servidores habilitados por padrão (criados no primeiro uso):

- **qwen-core** — integração principal com o Qwen
- **Filesystem** — acesso a arquivos locais
- **Sequential-Thinking** — raciocínio passo a passo

### Transportes

Os servidores MCP podem ser configurados com dois transportes:

- **stdio** (padrão) — o servidor é lançado como subprocesso via `command`/`args`.
- **HTTP / SSE** — conecta a um servidor em execução pela rede usando `transportType: "http"` (ou `"sse"`) junto com o campo `url`.

### Adicionar / editar servidores

Edite o arquivo de configuração (veja abaixo) ou use a UI de Configurações. Exemplo de `settings.json`:

```json
{
  "mcpServers": {
    "Filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/home/voce/Docs"]
    },
    "Sequential-Thinking": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-sequential-thinking"]
    },
    "Remote-Server": {
      "transportType": "http",
      "url": "http://localhost:3000/mcp"
    }
  },
  "general": {
    "check_updates": true
  }
}
```

---

## Menu do aplicativo e Atalhos de teclado

O app oferece um menu nativo GTK HeaderBar (Linux) com a seguinte estrutura:

| Menu | Itens |
|------|-------|
| Arquivo | Minimizar, Maximizar, Sair |
| Editar | Desfazer, Refazer, Recortar, Copiar, Colar, Selecionar tudo |
| Ver | Recarregar, Alternar DevTools, Aumentar zoom, Diminuir zoom, Restaurar zoom |
| Janela | Nova janela, Tela cheia |
| Perfils | Abrir painel, abrir/iniciar um perfil, criar perfil |
| Ajuda | Documentação, GitHub, Verificar atualizações |

Atalhos de teclado:

| Atalho | Ação |
|--------|------|
| `Ctrl + N` | Nova janela |
| `Ctrl + W` | Fechar janela |
| `Ctrl + R` / `F5` | Recarregar |
| `Ctrl + Shift + I` | Alternar DevTools |
| `Ctrl + =` / `Ctrl + +` | Aumentar zoom |
| `Ctrl + -` | Diminuir zoom |
| `Ctrl + 0` | Restaurar zoom |
| `Ctrl + scroll wheel` | Aumentar/diminuir zoom (intervalo 0,5×–2,0×) |

---

## Integrações nativas

- **Clipboard** — copie/cole texto; cole imagens diretamente da área de transferência no chat.
- **Arrastar e soltar** — solte arquivos do seu gerenciador de arquivos no WebView para enviá-los.
- **Links externos** — links abertos via `window.open` são roteados: URLs de autenticação ficam dentro do WebView, enquanto outros links `http(s)` abrem no navegador padrão.
- **Notificações desktop** — o app exibe uma notificação nativa (via libnotify) quando há atualização disponível.
- **Tema do sistema** — o app segue o tema claro/escuro do sistema.
- **Diálogos nativos** — seletores de arquivo e caixas de confirmação usam os widgets nativos do SO.
- **Exportação de conversas** — exporte conversas do chat pela interface.

---

## Configuração

Local: `~/.config/qwen-studio-linux/settings.json`

Perfis, logins (cookies) e `localStorage` são armazenados em `~/.config/qwen-studio-linux/profiles/` (um diretório por perfil). Esse local é relativo ao usuário e funciona em qualquer máquina — inclusive quando o app é distribuído como AppImage. Nada de dados de perfil é gravado dentro da pasta do projeto ou do AppImage.

```json
{
  "mcpServers": { "<nome>": { "command": "...", "args": [...] } },
  "general": { "check_updates": true, "theme": "" }
}
```

- `mcpServers` — mapa de servidores MCP (command, args, env, cwd, transportType, url, timeout).
- `general.check_updates` — ativa/desativa a verificação automática de atualizações.
- `general.theme` — preferência de tema da interface.

---

## Atualizações

O app usa o plugin `tauri-plugin-updater`. Releases no GitHub precisam incluir `latest.json` para que o updater funcione. A aba **Updates** é injetada dinamicamente na tela de Configurações e também exibe um banner global quando há nova versão.

---

## Relatórios de falha (Crash Reports)

Se o app travar (panic), um relatório de falha é gravado automaticamente em:

```
~/.local/share/qwen-studio-linux/crash-logs/crash-<timestamp>.log
```

Cada arquivo contém a data/hora, versão, plataforma, mensagem do panic, local e (quando `RUST_BACKTRACE=1`) um backtrace. Esses logs são úteis ao reportar bugs.

---

## Solução de problemas

| Sintoma | Causa / Solução |
|---------|-----------------|
| `tauri: comando não encontrado` | Você não rodou `npm install` na pasta do projeto. O `node_modules/` não vem no clone do GitHub. Rode `npm install` e tente de novo. |
| Tela branca / WebView não renderiza | Defina `WEBKIT_DISABLE_COMPOSITING_MODE=1 WEBKIT_DISABLE_DMABUF_RENDERER=1` (já nos scripts npm). |
| App não abre sob Wayland | Force X11 com `GDK_BACKEND=x11` (feito automaticamente na inicialização). |
| Deep link `qwen://` não abre o app | Garanta que o `qwen-studio-linux.desktop` (handler `x-scheme-handler/qwen`) está instalado e registrado. |
| Servidor MCP "offline" | Ative-o nas Configurações de MCP ou confira o `command`/`args` em `settings.json`. |
| App travou / comportamento inesperado | Confira os logs de falha em `~/.local/share/qwen-studio-linux/crash-logs/` e anexe o `crash-*.log` mais recente ao reportar. |

---

## Estrutura do projeto

```
qwen-studio-linux/
├── src/                  # Código Rust (lib.rs, mcp/, app/, platform/, ipc/ ...)
├── capabilities/         # Permissões (capabilities) do Tauri
├── permissions/          # Conjuntos de permissões customizadas
├── icons/                # Ícones para o bundle
├── gen/                  # Tipos gerados pelo Tauri
├── mcp-bridge.mjs        # Proxy Node.js dos servidores MCP (recurso embutido)
├── tauri.conf.json       # Configuração do Tauri
├── Cargo.toml            # Dependências Rust
├── build.rs              # Build script do Tauri
├── qwen-studio-linux.desktop  # Entrada desktop Linux
└── package.json          # Scripts npm para a CLI do Tauri
```

### Como funciona

1. O Tauri carrega `chat.qwen.ai` num WebView WebKitGTK.
2. `core_bridge.js` é injetado como script de inicialização, criando `window.electronAPI` para o web app achar que roda no Electron.
3. Chamadas JS usam `window.__TAURI__.invoke()`, que roteia para comandos Rust em `lib.rs`.
4. O Rust gerencia servidores MCP via `tauri-plugin-mcp-bridge` → `mcp-bridge.mjs`.
5. A tela de Configurações é uma SPA; a aba Updates é injetada via polling JS.
6. A navegação é sandboxed: apenas `chat.qwen.ai` e domínios de autenticação conhecidos (OAuth do Google/Alibaba) são permitidos.

---

## Desenvolvimento

```bash
npm run tauri:dev
```

Políticas do projeto:

- **Zero warnings**: `cargo clippy -- -D warnings` deve passar.
- **Versionamento**: atualizar a versão em `Cargo.toml` **e** `tauri.conf.json`.
- `node_modules/`, `dist/`, `target/` e assets de build ficam no `.gitignore`.

---

## Licença

Distribuído sob a licença **MIT**. Veja `Cargo.toml`.
