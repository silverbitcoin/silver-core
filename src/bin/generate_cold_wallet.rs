//! Cold Wallet Generator for SilverBitcoin
//! Generates offline wallet with private and public keys
//! Run this on an offline machine for maximum security

use serde_json::json;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating cold wallet...\n");

    // Generate wallet
    let (address, public_key, private_key) = silver_core::wallet::AddressGenerator::generate()?;

    // Create wallet data
    let wallet_data = json!({
        "version": "1.0",
        "type": "cold_wallet",
        "address": address,
        "public_key": public_key,
        "private_key": private_key,
        "created_at": chrono::Local::now().to_rfc3339(),
        "network": "mainnet",
        "security_notes": [
            "This is a cold wallet - keep private key offline",
            "Never share private key with anyone",
            "Store private key in encrypted format",
            "Use this address for mining rewards",
            "To spend coins, sign transactions offline with private key"
        ]
    });

    // Save to file
    let filename = format!("cold_wallet_{}.json", chrono::Local::now().format("%Y%m%d_%H%M%S"));
    fs::write(&filename, serde_json::to_string_pretty(&wallet_data)?)?;

    println!("═══════════════════════════════════════════════════════════");
    println!("  Cold Wallet Generated Successfully");
    println!("═══════════════════════════════════════════════════════════\n");

    println!("📋 Wallet Information:");
    println!("  Address:     {}", address);
    println!("  Public Key:  {}", public_key);
    println!("  Private Key: {}", private_key);
    println!("\n📁 Saved to: {}\n", filename);

    println!("⚠️  SECURITY INSTRUCTIONS:");
    println!("  1. ✓ Save this file in encrypted storage");
    println!("  2. ✓ Back up private key in multiple secure locations");
    println!("  3. ✓ Never share private key online");
    println!("  4. ✓ Use public address for mining on server");
    println!("  5. ✓ Keep this machine offline when possible\n");

    println!("🔐 Next Steps:");
    println!("  1. Send this address to mining server:");
    println!("     {}", address);
    println!("  2. Configure mining to use this address");
    println!("  3. Mining rewards will go to this address");
    println!("  4. To spend coins, sign transactions offline\n");

    println!("═══════════════════════════════════════════════════════════");

    Ok(())
}
