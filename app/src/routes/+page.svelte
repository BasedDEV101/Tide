<script lang="ts">
	import { Connection, PublicKey } from '@solana/web3.js';
	import { AnchorProvider, Program } from '@coral-xyz/anchor';

	const TIDES_PROGRAM_ID = new PublicKey('Tides1111111111111111111111111111111111111111');

	// Initialize Solana connection
	const connection = new Connection('http://localhost:8899', 'confirmed');

	// This is a placeholder - in full implementation, you would load the IDL and initialize the program
	// const program = new Program(idl, TIDES_PROGRAM_ID, provider);
</script>

<div class="min-h-screen bg-gradient-to-b from-blue-900 to-blue-950 text-white">
	<div class="container mx-auto px-4 py-16">
		<header class="text-center mb-16">
			<h1 class="text-6xl font-bold mb-4">🌊 Tides</h1>
			<p class="text-xl text-blue-200">
				An onchain multiplayer fishing game built on Solana
			</p>
		</header>

		<div class="max-w-4xl mx-auto">
			<div class="bg-blue-800/50 rounded-lg p-8 mb-8">
				<h2 class="text-3xl font-semibold mb-4">Welcome to Tides</h2>
				<p class="text-blue-100 mb-4">
					Navigate vast oceans, catch rare fish, manage your ship and inventory, and compete in a dynamic player-driven economy.
				</p>
				<p class="text-blue-200">
					Connect your Solana wallet to get started!
				</p>
			</div>

			<div class="grid md:grid-cols-2 gap-6">
				<div class="bg-blue-800/50 rounded-lg p-6">
					<h3 class="text-xl font-semibold mb-2">🎮 Gameplay</h3>
					<ul class="text-blue-100 space-y-2">
						<li>• Explore hex-grid oceans</li>
						<li>• Catch fish with strategic bait</li>
						<li>• Manage inventory in Tetris-like cargo</li>
						<li>• Trade in dynamic markets</li>
					</ul>
				</div>

				<div class="bg-blue-800/50 rounded-lg p-6">
					<h3 class="text-xl font-semibold mb-2">⚓ Features</h3>
					<ul class="text-blue-100 space-y-2">
						<li>• Built on Solana blockchain</li>
						<li>• Fast and low-cost transactions</li>
						<li>• Player-driven economy</li>
						<li>• Seasonal leaderboards</li>
					</ul>
				</div>
			</div>
		</div>
	</div>
</div>

