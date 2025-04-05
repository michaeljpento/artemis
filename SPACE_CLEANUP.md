# Space Cleanup Recommendations

Run these commands to free up space:

## Clean up Rust/Cargo
```bash
# Clean Artemis build artifacts
cargo clean --manifest-path=/Users/michaelpento.lv/artemis/Cargo.toml

# Clean cargo caches (saves ~500MB)
rm -rf ~/.cargo/registry/cache/*
rm -rf ~/.cargo/registry/src/*

# Clean old rustup toolchains (potentially saves ~2GB)
# Run this carefully - keeps current toolchain
rustup toolchain list
rustup toolchain uninstall <old-toolchain-name>
```

## Clean up system caches
```bash
# Clear system caches (~1.7GB)
rm -rf ~/Library/Caches/*

# Clear npm caches if you have Node.js
npm cache clean --force
```

## Other space-saving tips
1. Use `brew cleanup` if you have Homebrew installed
2. Empty Trash: Finder → Empty Trash
3. Remove old downloads: `rm -rf ~/Downloads/*` (backup important files first)
4. Use a tool like OmniDiskSweeper or DaisyDisk to identify large files
5. Check Docker with `docker system prune -a` if installed

After cleanup, run `df -h` to check available space.
```