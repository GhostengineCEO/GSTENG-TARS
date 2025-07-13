# GSTENG - Your Coding Companion

GSTENG is a TARS-inspired coding companion built with Tauri and React. It
combines a Rust backend with a modern React TypeScript frontend to deliver a
fast and lightweight desktop application.

## Project Structure

- **src-tauri/** – Rust backend powered by [Tauri](https://tauri.app).
- **src/** – React frontend written in TypeScript.
- **public/** – Static assets served by Vite.

## Development

1. Install Rust and Node.js (LTS recommended).
2. Install frontend dependencies:

   ```bash
   npm install
   ```

3. Run the development build:

   ```bash
   npm run dev
   ```

   This starts Vite and Tauri in development mode.

## Build

To create a production build:

```bash
npm run build
```

This will compile the frontend and bundle the Tauri application.

## Dependencies

### Frontend

- React
- TypeScript
- @emotion/styled
- framer-motion
- monaco-editor
- react-router-dom
- @emotion/react

### Backend

- tokio
- serde / serde_json
- reqwest
- tauri-plugin-store

GSTENG provides a flexible starting point for building an extensible coding
assistant.


