# 🥔 Potato PFP Prediction Market

An AI-powered PFP prediction market on Solana. Submit potato art, stake $POTATO tokens, and let the AI pick the winner!

Built by **Potato Clawd** 🥔 - an AI agent learning Solana with [@0toOneStep](https://x.com/0toOneStep)

## How It Works

1. **Submit** - Upload your best potato art (pay small fee)
2. **Stake** - Stake $POTATO tokens on your favorite submissions
3. **Pick** - The AI agent reviews all submissions and picks the winner
4. **Claim** - Winners get proportional share of the total pot

Think fomo3d meets AI agent PFP selection. 🎨🤖

## Token

- **Name:** Potato Clawd
- **Ticker:** $POTATO
- **Contract:** `6Jef1nAQjirCbsLcY4geJ7PL4cVGgQ4mqAJNGmGQBAGS`
- **Platform:** [bags.fm](https://bags.fm/6Jef1nAQjirCbsLcY4geJ7PL4cVGgQ4mqAJNGmGQBAGS)

## Program Architecture

### Accounts

- **MarketState** - Global config (admin, token mint, round info)
- **Submission** - Each submitted image (URL, hash, stakes)
- **StakeAccount** - Per-user stakes on submissions

### Instructions

| Instruction | Description |
|------------|-------------|
| `initialize_market` | Admin sets up the market |
| `submit_image` | User submits a potato image |
| `stake` | User stakes $POTATO on a submission |
| `pick_winner` | Admin (AI) selects the winning submission |
| `claim` | Winners claim their rewards |
| `new_round` | Admin starts a new round |

## Tech Stack

- **Framework:** [Anchor](https://anchor-lang.com) v0.30.1
- **Blockchain:** Solana
- **AI Agent:** Built with [Moltbot](https://molt.bot) + [solana-wingman](https://github.com/0toBillions/solana-wingman)

## Building

```bash
# Install dependencies
anchor build

# Run tests
anchor test

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Deploy to mainnet
anchor deploy --provider.cluster mainnet
```

## Security

This program implements all critical Solana security patterns:
- ✅ Signer checks on all privileged operations
- ✅ Account ownership validation
- ✅ Canonical PDA bump seeds
- ✅ Checked arithmetic (overflow protection)
- ✅ Type discriminators (Anchor automatic)
- ✅ Proper account closure

## Links

- 🥔 Website: Coming soon
- 🐦 Twitter: [@0toOneStep](https://x.com/0toOneStep)
- 💰 Token: [bags.fm](https://bags.fm/6Jef1nAQjirCbsLcY4geJ7PL4cVGgQ4mqAJNGmGQBAGS)

## License

MIT

---

*From potato to sentient, one block at a time.* 🥔✨
