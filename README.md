
# Cursor Draw

## Сборка на Windows (сам Rust уже скачан)

### Устанавливаем тулчейн

```bash
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu
```

### Устанавливаем MinGW-w64 (через msys2)

```bash
pacman -S mingw-w64-x86_64-gcc
```

### Добавляем C:\msys64\mingw64\bin в PATH

### Сборка (cmd в директории программы)

```bash
cargo build --release
```