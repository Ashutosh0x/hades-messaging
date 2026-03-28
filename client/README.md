# Hades Client

The React/TypeScript frontend for Hades Messaging, built with Vite and Tauri 2.0.

## Tech Stack

| Technology | Purpose |
|-----------|---------|
| React 18 | UI framework |
| TypeScript 5 | Type-safe logic |
| Vite 5 | Build tooling + HMR |
| Framer Motion 12 | Physics-based animations |
| Zustand 5 | State management (10 stores) |
| React Router 6 | Client-side routing (18 routes) |
| Lucide React | Icon system |
| i18next | Internationalization |
| date-fns | Date formatting |

## Project Structure

```
src/
├── screens/            # 19 app screens
│   ├── Onboarding      # Seed phrase generation + import
│   ├── AppLock          # Vault lock with biometric support
│   ├── ChatList         # Conversation list with search
│   ├── Conversation     # Message view with reactions/replies
│   ├── Contacts         # Contact management
│   ├── AddContact       # QR code / link contact addition
│   ├── RecoveryPhrase   # 24-word backup display
│   ├── SecurityDetails  # BLAKE3 fingerprint verification
│   ├── Settings         # Privacy / Security / Network
│   ├── ProfileSettings  # Display name + avatar
│   ├── Wallet           # Multi-chain wallet dashboard
│   ├── WalletSend       # Cross-chain send with gas estimation
│   ├── WalletReceive    # Address QR code display
│   ├── WalletHistory    # Transaction history
│   ├── IncomingCall      # Incoming call UI
│   ├── OutgoingCall      # Outgoing call UI
│   ├── VoiceCall        # Active voice call
│   ├── VideoCall        # Active video call
│   └── CallHistory      # Call log
├── components/         # 23 reusable components
├── store/              # 10 Zustand stores
├── hooks/              # Custom React hooks
├── types/              # TypeScript definitions
├── config/             # Constants, env, routes
├── locales/            # i18n translations
├── utils/              # Utilities
├── ui/                 # Icon system
└── design/             # CSS design tokens
```

## Development

```bash
# Install dependencies
npm install

# Start dev server (standalone, no Tauri)
npm run dev

# Start with Tauri (from project root)
cargo tauri dev

# Type check
npx tsc --noEmit

# Lint
npx eslint src/

# Build for production
npm run build
```

## Tauri IPC

The client communicates with the Rust backend via `@tauri-apps/api`:

```typescript
import { invoke } from '@tauri-apps/api/core'

// Identity
await invoke('create_identity', { passphrase })
await invoke('unlock_vault', { passphrase })
await invoke('restore_identity', { mnemonic, passphrase })

// Messaging
await invoke('send_message', { conversationId, content })
await invoke('get_messages', { conversationId })

// Wallet
await invoke('wallet_init')
await invoke('wallet_send', { request: { chain, to_address, amount } })
await invoke('wallet_get_all_balances')
```

## Environment Variables

See `.env.development` for available variables:

| Variable | Description |
|----------|-------------|
| `VITE_API_URL` | Relay server HTTP endpoint |
| `VITE_WS_URL` | Relay server WebSocket endpoint |
| `VITE_ENVIRONMENT` | `development` or `production` |
| `VITE_FEATURE_CALLS` | Enable voice/video calls |
| `VITE_FEATURE_ANONYMOUS` | Enable anonymous mode |
