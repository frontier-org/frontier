# ⚡ Frontier Documentation

O **Frontier** é um Engine de Interface Gráfica (GUI) agnóstico a linguagem. Ele permite criar aplicativos Desktop nativos e portáteis para Windows, onde o Backend pode ser escrito em qualquer linguagem (C, Python, Java, Go, Batch, Node) e o Frontend é feito com tecnologias Web modernas.

---

## 📂 1. Estrutura do Projeto

Um projeto Frontier saudável segue esta estrutura:

```text
/MeuProjeto
│
├── frontier.cmd           # CLI (Interface de Linha de Comando)
├── frontier.toml          # Metadados do Executável (Versão, Ícone do EXE)
│
├── app/
│   ├── frontend/          # HTML, CSS, JS e Ícones de Janela
│   │   ├── index.html
│   │   └── style.css
│   └── backend/           # Seus Scripts e Códigos Fonte
│       ├── calculo.c
│       ├── script.py
│       └── ComplexApp!java_gradle/  (Pasta como Backend)
│
├── modules/               # Definições de Linguagem (Compiladores/Interpretadores)
│   ├── mod_c/
│   └── mod_python/
│
└── .frontier/             # Engine (Rust, Cache, Build System) - Não mexa aqui
```

---

## ⚙️ 2. Configuração do Executável (`frontier.toml`)

Este arquivo controla **apenas** os metadados do arquivo `.exe` final gerado no Windows. As configurações de janela (tamanho, posição) agora são controladas pelo HTML.

**Arquivo:** `frontier.toml`
```toml
[app]
name = "MeuSuperApp"       # Nome do arquivo final (ex: MeuSuperApp.exe)
version = "1.0.0"          # Versão (aparece em Propriedades do Arquivo)
description = "Descrição"  # Descrição do arquivo
copyright = "© 2025 Corp"  # Direitos Autorais
author = "Dev Name"        # Autor

[window]
# Ícone que aparece no Windows Explorer e Barra de Tarefas.
# OBRIGATÓRIO SER .ICO VÁLIDO (não renomeie png).
icon = "app/frontend/icon.ico" 
```

---

## 🖥️ 3. Frontend & Gerenciamento de Janelas

O Frontier trata o HTML como a "configuração da janela". Você controla o comportamento da janela nativa usando **Meta Tags** no `<head>`.

### Configurações Disponíveis (Meta Tags)

| Meta Name | Valor Exemplo | Descrição |
| :--- | :--- | :--- |
| `frontier-title` | "Meu App" | Título da Janela (Ou use a tag `<title>`). |
| `frontier-width` | `800` | Largura inicial. |
| `frontier-height` | `600` | Altura inicial. |
| `frontier-min-width`| `400` | Largura mínima permitida. |
| `frontier-min-height`| `300` | Altura mínima permitida. |
| `frontier-x` | `(screen_w - win_w) / 2` | Posição Horizontal. Aceita Fórmulas Matemáticas. |
| `frontier-y` | `0` | Posição Vertical (0 = Topo). Aceita Fórmulas. |
| `frontier-resizable`| `true` / `false` | Permite redimensionar a borda. |
| `frontier-maximized`| `true` / `false` | Inicia maximizado. |
| `frontier-minimizable`| `true` / `false` | Mostra/Oculta botão de minimizar. |
| `frontier-maximizable`| `true` / `false` | Mostra/Oculta botão de maximizar. |
| `frontier-icon` | `icone.png` | Ícone da barra de título (caminho relativo ao HTML). |
| `frontier-persistent`| `true` | Salva/Restaura posição e tamanho ao fechar. |
| `frontier-id` | `main_window` | ID único para o arquivo de save da persistência. |

### Fórmulas Matemáticas
Nas tags `x` e `y`, você pode usar variáveis:
*   `screen_w`: Largura do Monitor.
*   `screen_h`: Altura do Monitor.
*   `win_w`: Largura da Janela.
*   `win_h`: Altura da Janela.

**Exemplo de HTML Completo:**
```html
<!DOCTYPE html>
<html>
<head>
    <title>Painel Admin</title>
    <!-- Centralizar -->
    <meta name="frontier-x" content="(screen_w - win_w) / 2">
    <meta name="frontier-y" content="(screen_h - win_h) / 2">
    <!-- Tamanho e Ícone -->
    <meta name="frontier-width" content="1024">
    <meta name="frontier-height" content="768">
    <meta name="frontier-icon" content="assets/admin.png">
    <!-- Persistência -->
    <meta name="frontier-persistent" content="true">
    <meta name="frontier-id" content="admin_panel">
</head>
<body>
    <h1>App Rodando</h1>
    <button onclick="run()">Executar Backend</button>
    <script>
        // API IPC
        function run() {
            // Sintaxe: "arquivo_backend|argumentos"
            window.ipc.postMessage('calculo|10 20');
        }
        
        // Receber Resposta
        window.Frontier = {
            dispatch: (tipo, msg) => {
                console.log(msg); // Recebe do Rust
            }
        };
    </script>
</body>
</html>
```

### Abrindo Novas Janelas
Você pode abrir janelas secundárias (popups) via JS:
```javascript
// Abre o arquivo popup.html em uma nova janela nativa
window.ipc.postMessage('open|popup.html');
```

---

## 🧱 4. Implementação de Backend

Coloque seus arquivos em `app/backend/`. O Frontier detecta a extensão e busca o módulo correspondente.

### Tipos de Backend Suportados

1.  **Arquivo Único (`script.py`, `codigo.c`)**
    *   O Frontier pega o nome do arquivo como gatilho.
    *   Ex: `app/backend/analise.py` -> Gatilho: `analise`.

2.  **Pasta de Projeto (`Nome!extensao`)**
    *   Use para projetos complexos (Java Gradle, C Make, Node Modules).
    *   A pasta deve ter o nome no formato: `NomeDoComando!extensao_do_modulo`.
    *   Ex: Pasta `app/backend/Benchmark!java`.
    *   O Frontier entra na pasta, roda o build definido no módulo `java` e gera o executável.
    *   Gatilho: `Benchmark`.

### Argumentos
Tudo que você passa no JS (`window.ipc.postMessage('gatilho|arg1 arg2')`) é repassado para o binário/script como argumentos de linha de comando (`argv`).

---

## 📦 5. Criação de Módulos (`modules/`)

Um módulo ensina o Frontier a compilar ou rodar uma linguagem.
Crie uma pasta em `modules/nome_do_modulo/` e adicione um `manifest.toml`.

### Referência do `manifest.toml`

```toml
name = "Nome Legível"
version = "1.0.0"       # Para sistema de update
extension = "py"        # Extensão que este módulo controla

# (Opcional) Interpretador para rodar o arquivo final
# Use isso para linguagens de script (Python, JS, Bat) ou Bytecode (Java)
interpreter = "python" 

# (Opcional) Se true, não mostra a janela preta do console ao rodar
suppress_window = true

# CONFIGURAÇÃO DE BUILD (Produção e Dev "Build Strategy")
[build]
# Variáveis Mágicas:
# %IN%  -> Caminho absoluto do arquivo fonte (ou pasta do projeto)
# %OUT% -> Caminho absoluto onde o Frontier espera o arquivo final
command = "gcc %IN% -o %OUT%"

# CONFIGURAÇÃO DE DEV (Hot Reload)
[dev]
# "interpreter": Não faz nada quando salva, apenas roda. (Python, JS)
# "build": Roda o comando [build] toda vez que o arquivo é salvo. (C, Go, Rust)
strategy = "interpreter"
```

### Exemplos Práticos

**Python (Script):**
```toml
extension = "py"
interpreter = "python"
suppress_window = true
[dev]
strategy = "interpreter"
```

**C (Nativo):**
```toml
extension = "c"
suppress_window = true
[build]
command = "gcc %IN% -o %OUT%"
[dev]
strategy = "build"
```

**Java Gradle (Pasta):**
```toml
extension = "java"
interpreter = "java -jar"
[build]
# O Frontier define o diretório de trabalho automaticamente para dentro da pasta
command = "call gradle build -x test && copy /Y build\\libs\\app.jar %OUT%"
[dev]
strategy = "build"
```

---

## 💻 6. CLI (Linha de Comando)

Use o script `.\frontier` na raiz.

*   **`.\frontier dev`**
    *   Inicia o modo de desenvolvimento.
    *   Ativa **Hot Reload** (alterações no Front ou Back refletem na hora).
    *   Lê arquivos diretamente da pasta `app/`.
    *   Compila binários (C/Go) em cache temporário.
*   **`.\frontier build`**
    *   Inicia o modo de produção.
    *   Compila todos os scripts e projetos.
    *   Gera um executável único em `dist/`.
    *   Este executável é **estático** (não precisa de DLLs ao lado).
*   **`.\frontier install <url>`**
    *   Baixa módulos da internet.
    *   Suporta `gh:user/repo` (GitHub).
    *   Suporta `https://.../arquivo.zip`.
    *   Suporta `--folder nome` para baixar subpastas de monorepos.
*   **`.\frontier clean`**
    *   Limpa pastas temporárias (`target`, `assets`, `dist`). Use se algo estranho acontecer.

---

## 🛡️ Notas Técnicas

1.  **Persistência:** Os dados da janela (e cookies/localstorage) são salvos em `%LOCALAPPDATA%\FrontierData\NomeDoApp`.
