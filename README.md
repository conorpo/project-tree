# project-tree

A simple ascii file tree generator. Designed to be used in project root. By default it will print to stdout, and copy to clipboard. By default it will not recurse into node_modules, .git, or .vscode folders. In Rust projects (where Cargo.toml is present) it will not recurse into the `target` directory. If a .gitignore file is found it will color the entries matched and not recurse into them. This behaviour can be overriden. I made this so I can give ChatGPT my project tree easily, and it can better understand the context of my projects.

```rust
//! TODO:
//! Is HashMap<PathBuf> really the best way to do this?
```

## Usage

```bash
project-tree [flags] [options] [gitignore]
```

## Flags

| Flag | Description |
| --- | --- |
| --node_modules | Include node_modules |
| --git | Include .git |
| --vscode | Include .vscode |
| --target | Include contents of target in Rust projects |
| --noclip | Do not copy to clipboard |
| -r, --root | Include parent directory in tree, and indent all other files |
| -d, --dirs | Prioritize directories over files (default alphabetical) |

## Options

| Option | Arg | Description |
| --- | --- | --- |
| -o, --output | path | Output file |
| -i, --ignore | path | A file/folder to ignore, can be repeated |
| -s, --stop | path | A file/folder to not recurse into, can be repeated |

## GitIgnore

| Value | Description |
| --- | --- |
| gi-off | Do not use .gitignore file |
| gi-ignore | Ignore all entries specified in .gitignore |
| gi-stop | Do not recurse into directories specified in .gitignore |
| gi-dim | Color .gitignore enties a dimmer shade of grey |
| gi-dim-and-stop | A combination of both gi-dim and gi-stop |

## Examples

```bash
project-tree -i Cargo.lock -s target -r -dirs
```

```bash
project-tree
├── src/
│   └── main.rs
├── target/
├── .gitignore
├── Cargo.toml
└── README.md
```

On another one of my projects: [pt-gpt](https://github.com/conorpo/pt-gpt)

```bash
project-tree -i .github -i frontend/.expo -i frontend/node_modules -i frontend/web-build/ -s frontend/assets -dirs
```

```bash 
config/
│   ├── logger.js
│   ├── mongo_connection.js
│   └── openai_connection.js
controllers/
│   ├── auth.js
│   ├── chat.js
│   └── user.js
frontend/
│   ├── assets/
│   ├── components/
│   │   ├── AlertModal.js
│   │   ├── BackButton.js
│   │   ├── Back_Icon.svg
│   │   ├── icons8-settings.svg
│   │   └── SettingsButton.js
│   ├── contexts/
│   │   └── Main.js
│   ├── dist/
│   │   ├── assets/
│   │   └── bundles/
│   ├── screens/
│   │   ├── unit-testing/
│   │   │   └── chat.test.js
│   │   ├── Chat.js
│   │   ├── Loading.js
│   │   ├── Login.js
│   │   └── Profile.js
│   ├── App.js
│   ├── app.json
│   ├── babel.config.js
│   ├── eas.json
│   ├── package-lock.json
│   └── package.json
helpers/
│   └── emailSender.js
logs/
middlewares/
│   └── jwt_auth.js
models/
│   └── User.js
node_modules/
routes/
│   └── api/
│       ├── protected/
│       │   ├── chat.js
│       │   └── user.js
│       └── auth.js
.babelrc.json
.env
.gitignore
app.js
package-lock.json
package.json
README.md
```
