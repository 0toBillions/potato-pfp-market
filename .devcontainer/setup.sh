#!/bin/bash
set -e

echo "🥔 Setting up Potato PFP Market dev environment..."

# Install Solana CLI
echo "Installing Solana CLI..."
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)" || {
  echo "Trying alternative Solana install..."
  curl -sSfL https://release.anza.xyz/stable/install | sh
}
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Install Anchor via AVM
echo "Installing Anchor..."
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
~/.cargo/bin/avm install latest
~/.cargo/bin/avm use latest

# Add to bashrc for future sessions
echo 'export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"' >> ~/.bashrc
echo 'export PATH="$HOME/.avm/bin:$PATH"' >> ~/.bashrc

echo "🥔 Setup complete! Run 'anchor build' to compile."
