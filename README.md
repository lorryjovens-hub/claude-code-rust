# Claude Desktop (Tauri Edition)

A desktop client for AI chat assistants, built with Tauri 2.0, Rust, and React. Provides a native-feeling GUI for interacting with Claude and other LLM providers, with local data persistence and a plugin-style architecture for extending functionality.

---

## What This Is

This is a personal/community desktop application that wraps AI chat APIs in a native window. It is not an official Anthropic product, and it is not a polished commercial release. It is under active development: features change frequently, some things are incomplete, and you should expect rough edges.

The app uses Tauri 2.0's Rust backend to handle file system access, process management, SQLite storage, and HTTP proxying, while the React 19 frontend provides the chat UI, settings panels, and auxiliary views.

---

## Features

### Chat and Models
- Multi-provider chat with streaming responses (SSE)
- Multiple model support across Anthropic, OpenAI, DeepSeek, and other compatible APIs
- Chat history persistence in SQLite
- Context-aware memory system that surfaces relevant past conversations
- Markdown rendering with syntax highlighting, KaTeX math, and Mermaid diagrams
- Voice input support

### Tooling and Extensions
- MCP (Model Context Protocol) server integration -- connect external tools that the AI can invoke
- Multi-agent orchestration -- coordinate multiple AI agents working on subtasks in parallel
- Skills system -- user-extensible plugin modules for specialized capabilities
- Slash commands for quick actions within the chat input
- Built-in terminal panel (xterm.js)
- Code diff viewer for reviewing AI-generated code changes
- Code execution panel for running code snippets

### Panels and Views
- File explorer for browsing the local filesystem
- Knowledge base panel for managing reference documents
- Research panel for multi-step web research workflows
- Artifact preview for rendered HTML/JSX/WebGPU outputs
- Live preview panel for web content
- Document panel with DOCX and PDF preview
- Swarm collaboration view for multi-agent task management
- IM integration panel (Feishu/WeChat)
- Analytics and cost tracking dashboard
- GitHub integration panel
- App Studio for building mini-apps within the desktop

### Desktop Integration
- System tray icon
- Native file dialogs
- Clipboard integration
- Desktop notifications
- Automatic updater (checks for new releases)

### Permissions
- Configurable permission modes: ask every time, auto-accept edits, plan-only, or full bypass
- Per-tool permission controls

---

## Tech Stack

### Frontend
| Technology | Purpose |
|------------|---------|
| React 19 | UI framework |
| TypeScript 5.7 | Type system |
| Vite 6 | Build tooling |
| Tailwind CSS 3 | Utility-first styling |
| Zustand 5 | State management |
| React Router 6 | Client-side routing |
| xterm.js 5 | Terminal emulation |
| KaTeX | Math rendering |
| Mermaid | Diagram rendering |
| recharts | Charting |
| highlight.js / react-syntax-highlighter | Code highlighting |
| react-markdown / remark / rehype | Markdown pipeline |

### Backend (Rust)
| Technology | Purpose |
|------------|---------|
| Tauri 2.0 | Desktop application framework |
| Axum 0.8 | Internal HTTP bridge between frontend and backend |
| Tokio 1 | Async runtime |
| rusqlite 0.31 (bundled) | Local SQLite database |
| reqwest 0.12 | HTTP client for AI API calls |
| Tower HTTP 0.6 | HTTP middleware (CORS) |
| Tracing / OpenTelemetry | Structured logging and telemetry |
| Serde / serde_json | Serialization |
| UUID 1 / Chrono 0.4 | Identifiers and timestamps |
| diffy 0.4 | Text diffing for code review |
| notify 6 | Filesystem watcher |
| enigo 0.2 | Input simulation |
| Prometheus 0.13 | Metrics exposition |

---

## Project Structure

```
claude-desktop-tauri/
├── src/                          # React frontend source
│   ├── components/               # UI components
│   │   ├── chat/                 # Chat interface components
│   │   ├── swarm/                # Swarm collaboration components
│   │   ├── ui/                   # Shared UI primitives
│   │   └── ...                   # ~80+ component files
│   ├── features/                 # Feature modules
│   │   ├── chat/                 # Chat feature logic
│   │   ├── streaming/            # SSE stream handling
│   │   ├── skills/               # Skills integration
│   │   ├── slash-commands/       # Slash command handling
│   │   └── ...
│   ├── stores/                   # Zustand state stores
│   ├── hooks/                    # React custom hooks
│   ├── services/                 # API service layer
│   ├── types/                    # TypeScript type definitions
│   ├── utils/                    # Utility functions
│   ├── locales/                  # i18n localization
│   ├── assets/                   # Static assets
│   ├── App.tsx                   # Root application component
│   └── main.tsx                  # Entry point
├── src-tauri/                    # Rust backend source
│   ├── src/
│   │   ├── main.rs               # Tauri entry point
│   │   ├── lib.rs                # Library root
│   │   ├── bridge/               # Axum HTTP API server
│   │   ├── engine/               # AI engine integration
│   │   ├── native_engine/        # Native engine logic
│   │   ├── memory/               # Conversation memory system
│   │   ├── db/                   # SQLite database layer
│   │   ├── mcp/                  # MCP client/server
│   │   ├── multiagent/           # Multi-agent orchestration
│   │   ├── orchestration/        # Task orchestration
│   │   ├── permissions/          # Permission management
│   │   ├── skills/               # Skills loader
│   │   ├── tools/                # Tool implementations
│   │   ├── worktree/             # Git worktree management
│   │   ├── terminal/             # Terminal backend
│   │   ├── diff/                 # Diff utilities
│   │   ├── streaming/            # SSE streaming
│   │   ├── im_integration/       # IM platform integration
│   │   ├── knowledge/            # Knowledge base
│   │   ├── research/             # Research engine
│   │   ├── config/               # Configuration management
│   │   ├── analytics/            # Usage analytics
│   │   ├── cost_tracker/         # API cost tracking
│   │   ├── web_search/           # Web search integration
│   │   ├── sandbox/              # Code execution sandbox
│   │   ├── project/              # Project management
│   │   ├── agent_bus/            # Agent communication bus
│   │   ├── git/                  # Git integration
│   │   ├── github/               # GitHub integration
│   │   ├── fs/                   # Filesystem operations
│   │   ├── process/              # Process management
│   │   ├── notification/         # Desktop notifications
│   │   ├── updater/              # Auto-update logic
│   │   ├── watcher/              # File system watcher
│   │   ├── scheduler/            # Task scheduler
│   │   ├── computer_use/         # Computer use (input simulation)
│   │   ├── app_studio/           # App Studio backend
│   │   ├── remotion/             # Remotion video rendering
│   │   ├── document/             # Document handling
│   │   ├── commands/             # Custom commands
│   │   ├── slash_commands/       # Slash commands backend
│   │   ├── api/                  # External API clients
│   │   ├── prompt/               # Prompt templates
│   │   ├── logger/               # Logging infrastructure
│   │   ├── metrics/              # Metrics collection
│   │   ├── cache/                # Caching layer
│   │   ├── clipboard/            # Clipboard operations
│   │   ├── upload/               # File upload handling
│   │   ├── prefetch/             # Data prefetching
│   │   ├── ide/                  # IDE integration
│   │   └── superpowers/          # Superpowers module
│   ├── Cargo.toml                # Rust dependencies
│   └── tauri.conf.json           # Tauri configuration
├── package.json                  # Node.js dependencies
├── vite.config.ts                # Vite configuration
├── tailwind.config.js            # Tailwind configuration
├── tsconfig.json                 # TypeScript configuration
├── postcss.config.js             # PostCSS configuration
├── vitest.config.ts              # Test configuration
├── index.html                    # HTML entry point
├── scripts/                      # Build and utility scripts
├── docs/                         # Project documentation
└── data/                         # Bundled data files
```

---

## Build Instructions

### Prerequisites
- **Rust** 1.70+ (install via [rustup](https://rustup.rs))
- **Node.js** 18+ (recommend [nvm](https://github.com/nvm-sh/nvm) or [fnm](https://github.com/Schniz/fnm))
- **Platform-specific Tauri dependencies**:
  - Windows: Microsoft Visual Studio C++ Build Tools, WebView2 (pre-installed on Windows 10+)
  - macOS: Xcode Command Line Tools
  - Linux: `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, and other system libraries
  - See [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for full details

### Development

```bash
# Clone the repository
git clone <repo-url>
cd claude-desktop-tauri

# Install frontend dependencies
npm install

# Start the development server with hot reload
npx tauri dev
```

This launches both the Vite dev server (frontend) and the Tauri Rust backend. The frontend hot-reloads on file changes; Rust changes require a rebuild (Tauri handles this automatically on save).

### Production Build

```bash
# Build for the current platform
npx tauri build

# Build artifacts are placed in:
#   src-tauri/target/release/bundle/
```

### Running Tests

```bash
# Frontend tests (Vitest)
npm test

# Frontend tests with coverage
npm run test:coverage

# Rust tests
cd src-tauri && cargo test
```

---

## Development Status

This project is **under active development**. It is not production-ready software and does not have official releases or published installers. Key caveats:

- APIs and internal interfaces may change without notice between commits
- Some features are partially implemented or experimental
- Configuration currently requires editing files directly; there is no first-run setup wizard
- The UI has areas that need polish and accessibility work
- Test coverage is uneven across the codebase
- Documentation is sparse and mainly lives in source comments and the `docs/` directory

If you want to try it, build from source following the instructions above. Expect to encounter bugs and incomplete features.

---

## Contributing

Contributions are welcome. Here is the process:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/your-feature`)
3. Make your changes
4. Run the existing tests to make sure nothing is broken
5. Commit using [Conventional Commits](https://www.conventionalcommits.org/) style (`git commit -m 'feat: add your feature'`)
6. Push to your fork and open a pull request

### Code conventions
- **Rust**: Format with `rustfmt`, pass `cargo clippy` without warnings
- **TypeScript**: Follow the project's ESLint configuration
- **Commits**: Use conventional commit prefixes (`feat:`, `fix:`, `refactor:`, `docs:`, `chore:`, `test:`)

Before starting on a large change, consider opening an issue to discuss the approach first.

---

## License

MIT -- see the [LICENSE](LICENSE) file for details. (If no LICENSE file exists at the repository root, the project defaults to MIT terms as stated here.)

---

## Acknowledgments

Built on excellent open-source projects:
- [Tauri](https://tauri.app/) -- desktop application framework
- [React](https://react.dev/) -- UI library
- [Axum](https://github.com/tokio-rs/axum) -- Rust web framework
- [SQLite](https://www.sqlite.org/) -- embedded database
- [Zustand](https://github.com/pmndrs/zustand) -- state management
- [xterm.js](https://xtermjs.org/) -- terminal emulator
