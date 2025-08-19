# WriteMagic WASM Module

This crate provides WebAssembly bindings for the WriteMagic core engine, enabling the Rust core functionality to be used in web applications through JavaScript interop.

## Overview

The WASM module exposes the core WriteMagic functionality through a JavaScript-friendly API:

- **Document Management**: Create, update, and retrieve documents
- **Project Management**: Manage writing projects and organize documents
- **AI Integration**: Access AI-powered writing assistance features
- **Cross-Platform Storage**: Compatible storage abstraction for web environments

## Quick Start

### 1. Setup Development Environment

First, set up the WASM development tools:

```bash
# Run the setup script from project root
./scripts/setup-wasm.sh
```

This script will install:
- `wasm32-unknown-unknown` Rust target
- `wasm-pack` for building WASM packages
- `wasm-bindgen-cli` for generating JavaScript bindings

### 2. Build the WASM Module

Build the WASM module using the provided script:

```bash
# Development build (with debug symbols)
./scripts/build-wasm.sh dev

# Release build (optimized)
./scripts/build-wasm.sh release

# Profiling build (optimized with debug info)
./scripts/build-wasm.sh profiling
```

### 3. Use in Web Application

After building, import and use the WASM module in your JavaScript application:

```javascript
import init, { WriteMagicEngine } from './core/wasm/pkg/writemagic_wasm.js';

async function initWriteMagic() {
    // Initialize the WASM module
    await init();
    
    // Create engine instance
    const engine = new WriteMagicEngine();
    
    // Initialize with configuration
    await engine.initialize('{"storage": "memory"}');
    
    // Create a project
    const project = await engine.create_project(
        "My Novel",
        "A story about...",
        "user123"
    );
    
    // Create a document
    const document = await engine.create_document(
        project.id,
        "Chapter 1",
        "It was a dark and stormy night..."
    );
    
    console.log('Created document:', document);
}

initWriteMagic().catch(console.error);
```

## API Reference

### WriteMagicEngine

The main WASM engine class that provides access to all WriteMagic functionality.

#### Constructor

```javascript
const engine = new WriteMagicEngine();
```

#### Methods

##### `initialize(config: string): Promise<void>`

Initialize the engine with configuration options.

```javascript
await engine.initialize('{"enable_logging": true}');
```

##### `create_document(project_id: string, title: string, content: string): Promise<WasmDocument>`

Create a new document in the specified project.

##### `update_document(document_id: string, content: string): Promise<WasmDocument>`

Update the content of an existing document.

##### `get_document(document_id: string): Promise<WasmDocument>`

Retrieve a document by its ID.

##### `create_project(name: string, description: string, owner_id: string): Promise<WasmProject>`

Create a new writing project.

##### `get_project(project_id: string): Promise<WasmProject>`

Retrieve a project by its ID.

##### `complete_text(request: WasmCompletionRequest): Promise<WasmCompletionResponse>`

Generate AI-powered text completions.

##### `is_initialized(): boolean`

Check if the engine has been initialized.

### Data Types

#### WasmDocument

Represents a document in the WriteMagic system.

```typescript
interface WasmDocument {
    readonly id: string;
    readonly title: string;
    readonly content: string;
    readonly project_id: string;
    readonly created_at: string;
    readonly updated_at: string;
}
```

#### WasmProject

Represents a writing project.

```typescript
interface WasmProject {
    readonly id: string;
    readonly name: string;
    readonly description: string;
    readonly owner_id: string;
    readonly created_at: string;
    readonly updated_at: string;
}
```

#### WasmCompletionRequest

Request object for AI text completion.

```typescript
class WasmCompletionRequest {
    constructor(prompt: string, model: string);
    set_max_tokens(tokens: number): void;
    set_temperature(temperature: number): void;
    set_context(context: string): void;
}
```

#### WasmCompletionResponse

Response object from AI text completion.

```typescript
interface WasmCompletionResponse {
    readonly content: string;
    readonly model: string;
    readonly tokens_used: number;
    readonly finish_reason: string;
}
```

## Development

### Building from Source

1. **Install Dependencies**
   ```bash
   ./scripts/setup-wasm.sh
   ```

2. **Build Development Version**
   ```bash
   cd core/wasm
   wasm-pack build --target web --out-dir pkg --dev
   ```

3. **Build Release Version**
   ```bash
   cd core/wasm
   wasm-pack build --target web --out-dir pkg
   ```

### Testing

Run WASM tests in headless browsers:

```bash
./scripts/test-wasm.sh
```

Or run tests manually:

```bash
cd core/wasm
wasm-pack test --headless --firefox
wasm-pack test --headless --chrome
```

### Local Development Server

To test the WASM module locally, start an HTTP server:

```bash
# Using Python
python3 -m http.server 8000

# Or using basic-http-server (if installed)
basic-http-server .
```

Then navigate to `http://localhost:8000` to test your web application.

## Architecture

### Crate Structure

```
core/wasm/
├── src/
│   └── lib.rs          # Main WASM bindings
├── Cargo.toml          # Dependencies and build config
├── wasm-pack.toml      # wasm-pack configuration
├── build.rs            # Build script
└── pkg/                # Generated WASM package (after build)
```

### Features

- **`console_error_panic_hook`** (default): Better error messages in browser console
- **`wasm`**: Enables WASM-specific optimizations in dependencies

### Memory Management

The WASM module is configured with:
- Maximum memory limit of 64MB
- Optimized for size with `-Os` flag
- Panic handling that logs to browser console
- Proper cleanup of JavaScript handles

### Performance Considerations

1. **Size Optimization**: The release build uses size optimization (`-Os`) to minimize WASM bundle size
2. **Memory Efficiency**: Limited memory usage with proper cleanup
3. **Async Operations**: All I/O operations are async to avoid blocking the JavaScript event loop
4. **Lazy Loading**: The WASM module supports lazy initialization

## Troubleshooting

### Common Issues

1. **"RuntimeError: unreachable"**
   - Usually indicates a panic in Rust code
   - Enable `console_error_panic_hook` feature for better error messages

2. **"Module not found" errors**
   - Ensure the WASM module is built and the path is correct
   - Check that your web server serves `.wasm` files with correct MIME type

3. **Memory-related errors**
   - The WASM module has a 64MB memory limit
   - Large operations should be chunked or streamed

4. **CORS issues in development**
   - Use a local HTTP server instead of `file://` URLs
   - Configure your development server to serve WASM files

### Debug Mode

Build in debug mode for better error messages:

```bash
./scripts/build-wasm.sh dev
```

This enables:
- Debug symbols in WASM
- Detailed panic messages
- Console logging
- Assertions and overflow checks

## Integration Examples

### React Integration

```jsx
import React, { useEffect, useState } from 'react';
import init, { WriteMagicEngine } from './wasm/pkg/writemagic_wasm.js';

function App() {
    const [engine, setEngine] = useState(null);
    const [document, setDocument] = useState(null);

    useEffect(() => {
        async function initEngine() {
            await init();
            const eng = new WriteMagicEngine();
            await eng.initialize('{}');
            setEngine(eng);
        }
        initEngine();
    }, []);

    const createDocument = async () => {
        if (engine) {
            const doc = await engine.create_document(
                'proj-1',
                'New Document',
                'Hello, world!'
            );
            setDocument(doc);
        }
    };

    return (
        <div>
            <button onClick={createDocument} disabled={!engine}>
                Create Document
            </button>
            {document && <p>Created: {document.title}</p>}
        </div>
    );
}
```

### Vue Integration

```vue
<template>
    <div>
        <button @click="createDocument" :disabled="!engine">
            Create Document
        </button>
        <div v-if="document">
            <h3>{{ document.title }}</h3>
            <p>{{ document.content }}</p>
        </div>
    </div>
</template>

<script>
import init, { WriteMagicEngine } from './wasm/pkg/writemagic_wasm.js';

export default {
    data() {
        return {
            engine: null,
            document: null
        };
    },
    async mounted() {
        await init();
        this.engine = new WriteMagicEngine();
        await this.engine.initialize('{}');
    },
    methods: {
        async createDocument() {
            if (this.engine) {
                this.document = await this.engine.create_document(
                    'proj-1',
                    'Vue Document',
                    'Hello from Vue!'
                );
            }
        }
    }
};
</script>
```

## Contributing

When contributing to the WASM module:

1. **Test thoroughly**: Run both unit tests and browser tests
2. **Optimize for size**: Be mindful of WASM bundle size
3. **Handle errors gracefully**: Always convert Rust errors to JavaScript-friendly formats
4. **Document TypeScript types**: Update the TypeScript definitions for new APIs
5. **Maintain compatibility**: Ensure changes don't break existing JavaScript code

## License

This project is licensed under MIT OR Apache-2.0.