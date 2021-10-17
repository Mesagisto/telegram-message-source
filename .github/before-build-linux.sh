sudo apt install -y musl-tools clang
sudo ln -s /usr/include/x86_64-linux-gnu/asm /usr/include/x86_64-linux-musl/asm
sudo ln -s /usr/include/asm-generic /usr/include/x86_64-linux-musl/asm-generic
sudo ln -s /usr/include/linux /usr/include/x86_64-linux-musl/linux
sudo ln -s /usr/bin/g++ /usr/bin/musl-g++
rustup target add x86_64-unknown-linux-musl
