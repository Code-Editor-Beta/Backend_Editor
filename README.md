# 🧠 Real-Time Code Collaboration Platform

A blazing-fast, real-time collaborative code editor built with Rust. Designed to support seamless multi-user editing with conflict-free synchronization and autosaving — even on low-end devices.

> Powered by CRDTs (YJS), WebSockets, MongoDB, Redis, and Axum.

---

## 🚀 Features

- ⚡ **Rust-Powered Backend**: Ultra-low latency with Axum and Tokio
- 🔁 **Real-Time Code Collaboration**: Multi-user editing using WebSockets and CRDTs (via YJS)
- 💾 **Persistent Autosave**: Save sessions and code automatically to MongoDB
- 📡 **Redis Pub/Sub**: Efficient message propagation and session state sync
- 🧠 **Smart Caching**: LRU-based memory cache for reduced DB hits and high performance
- 📱 **Mobile-Friendly**: Optimized to work smoothly even on low-end Android devices
- 🔐 **Secure & Scalable**: Built with modularity and production-readiness in mind

---

## 🧱 Tech Stack

| Layer         | Stack                                                                 |
|---------------|-----------------------------------------------------------------------|
| **Backend**   | [Rust](https://www.rust-lang.org/), [Axum](https://docs.rs/axum), [Tokio](https://tokio.rs/) |
| **Realtime**  | WebSockets, YJS (CRDT engine via WASM)                                |
| **Database**  | MongoDB (project/code storage), Redis (pub/sub for sync)              |
| **Caching**   | In-memory LRU Cache (for fast project access)                         |
| **Frontend**  | [Svelte](https://svelte.dev/) (planned) or any JS client              |

---

## 🛠️ Architecture

```text
  ┌────────────────────────────┐
  │     Frontend (Svelte)      │
  └────────────┬───────────────┘
               │ WebSocket
               ▼
     ┌──────────────────────┐
     │     Axum Server      │
     └────────┬─────────────┘
              │
     ┌────────▼────────┐
     │  CRDT Engine    │  ← via YJS + WASM
     └──────┬──────────┘
            │
   ┌────────▼────────┐       ┌───────────────┐
   │   Redis Pub/Sub │ ◄────►│   Other Users │
   └────────┬────────┘       └───────────────┘
            │
   ┌────────▼────────┐
   │    MongoDB      │  ← Project Save, Load, History
   └─────────────────┘
