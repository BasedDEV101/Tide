# Tides Frontend

SvelteKit frontend for the Tides onchain fishing game.

## Development

Once you've installed dependencies with `bun install` (or `npm install`), start a development server:

```bash
bun run dev

# or start the server and open the app in a new browser tab
bun run dev -- --open
```

## Building

To create a production version of your app:

```bash
bun run build
```

You can preview the production build with `bun run preview`.

## Project Structure

- `src/routes/` - Page routes
- `src/lib/` - Shared components and utilities
- `static/` - Static assets

## Solana Integration

The frontend uses:
- `@solana/web3.js` for Solana blockchain interactions
- `@coral-xyz/anchor` for Anchor program interactions
- `@solana/wallet-adapter-svelte` for wallet connectivity

## Styling

- TailwindCSS v4 for styling
- bits-ui for components
- Threlte for 3D rendering (planned)

