# ⚡ Frontier: Resumo Técnico do Projeto

O **Frontier** é um Engine de Interface Gráfica (GUI) Poliglota e Nativo. Ele permite que desenvolvedores criem interfaces utilizando tecnologias Web (**HTML5, CSS3, JavaScript**) para controlar backends escritos em qualquer linguagem (**C, Python, Rust, Go, Node.js**), consolidando o resultado em um **Executável Único e Estático** para Windows.

---

## 1. Arquitetura do Sistema

A arquitetura é fundamentada em dois binários Rust distintos que operam em ciclos de vida diferentes:

### 🛠️ A. O Manager (`manager.rs`) - "O Construtor"
Atua como CLI, sistema de build e orquestrador de pacotes.

* **Configuração:** Lê o arquivo `frontier.toml` para definir metadados (versão, copyright) e recursos visuais (ícones).
* **Gestão de Módulos:** Identifica linguagens na pasta `app/backend` e executa a pré-compilação necessária baseada nas regras de cada módulo.
* **Empacotamento:** Agrupa assets (HTML, CSS, JS) e binários compilados.
* **Pipeline de Build:** Invoca o compilador Rust (`Cargo`) para gerar o Core e organiza a entrega na pasta `dist/`.

### 🧠 B. O Core (`core.rs`) - "O Runtime"
É o motor do executável final (ex: `MeuApp.exe`).

* **WebView Nativo:** Renderiza a interface através do motor do sistema operacional (Edge WebView2 no Windows), linkado de forma estática para eliminar dependências de DLLs externas.
* **Protocolo `frontier://`:** Sistema de arquivos virtual que serve o conteúdo diretamente da memória (Produção) ou disco (Dev), mitigando erros de CORS.
* **IPC (Inter-Process Communication):** Ponte de comunicação que recebe comandos do JavaScript (`window.ipc.postMessage`) e despacha a execução para o binário ou script de backend em segundo plano.
* **Orquestração de Janelas:** Define propriedades da janela (dimensões, ícone, redimensionamento) dinamicamente via `<meta>` tags no HTML.
* **Persistência de Estado:** Armazena automaticamente coordenadas e estado da janela em `%LOCALAPPDATA%`, restaurando a experiência do usuário ao reiniciar.



---

## 2. Ciclo de Vida e Fluxo de Dados

### Modo Desenvolvimento (`.\frontier dev`)
1.  Define a flag de ambiente `FRONTIER_DEV`.
2.  O **Core** escaneia `app/backend` em busca de fontes (ex: `.c`, `.go`).
3.  **Compilação On-the-fly:** Se detectado, invoca o compilador local (ex: GCC) para gerar binários em um cache temporário (`.frontier/target/dev_cache`).
4.  **Hot Reload:** Um *watcher* monitora alterações. Mudanças no Front disparam um `reload`; mudanças no Back disparam uma recompilação silenciosa.

### Modo Produção (`.\frontier build`)
1.  O **Manager** limpa e prepara o diretório de assets.
2.  Scripts de backend são compilados e movidos para o bundle interno.
3.  **Injeção de Recursos:** Gera um `build.rs` dinâmico para embutir o ícone `.ico` e metadados diretamente no manifesto do executável Windows.
4.  **Compilação Estática:** O Core é compilado em modo `Release` (MSVC Estático).
5.  **Bundling:** Utiliza a macro `rust-embed` para "engolir" todos os assets, resultando em um único binário independente.

---

## 3. Matriz de Funcionalidades

| Recurso | Status | Descrição Técnica |
| :--- | :---: | :--- |
| **Executável Único** | ✅ | Compilação via MSVC Estático (Zero DLLs externas). |
| **Metadados Win32** | ✅ | Versão, Copyright e Ícone injetados via recurso nativo. |
| **Configuração via HTML** | ✅ | Layout e comportamento definidos por `<meta>` tags. |
| **Persistência de Janela** | ✅ | Cache de estado (Posição/Tamanho) no sistema de arquivos. |
| **Hot Reload** | ✅ | Atualização em tempo real para Front e Backend. |
| **Suporte Poliglota** | ✅ | Arquitetura modular que aceita qualquer binário via `manifest.toml`. |
| **Console Silencioso** | ✅ | Supressão de janelas de terminal (popups) para processos de fundo. |

---

> **Nota Técnica:** O Frontier resolve o problema de distribuição de apps "web-based" eliminando o overhead do Electron e a complexidade de gerenciar múltiplas runtimes no cliente final.
