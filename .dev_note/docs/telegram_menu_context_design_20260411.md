## Telegram Menu Context Design

### 1. Boundary

The regression lives at the Telegram channel edge. The daemon already
renders command menus, but plain follow-up replies such as `1` are
currently routed like ordinary chat text. The fix should remain inside
`telegram_client.rs` and must not modify `AgentCore` or global IPC
contracts.

### 2. Persistence

`TelegramChatState` will store the most recent pending menu selection as
a small enum so the choice survives daemon restart and shares the same
state persistence path already used for Telegram chat settings.

The menu marker is only needed for:

- `/select`
- `/coding_agent`
- `/model`
- `/mode`
- `/auto_approve`

It is cleared once the user makes a valid selection or a command with an
explicit argument replaces the menu flow.

### 3. Runtime Contract

When a Telegram menu is shown without explicit command arguments:

1. Record the menu kind in chat state.
2. If the next plain-text reply is a positive integer, map that number
   to the matching command argument for the recorded menu.
3. Re-run normal command handling with the reconstructed command text.
4. Clear the pending menu marker after a successful numeric match.

Examples:

- `/select` then `2` => `/select coding`
- `/coding_agent` then `1` => `/coding_agent codex`
- `/mode` then `2` => `/mode fast`

### 4. Verification

There is no existing JSON-RPC method that can drive Telegram routing via
`tizenclaw-tests`, so this cycle will verify the externally visible
contract with focused Telegram unit regressions plus host deployment and
repository test execution through `./deploy_host.sh`.
