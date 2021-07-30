sudo apt install musl-tools
sudo ln -s /usr/include/x86_64-linux-gnu/asm /usr/include/x86_64-linux-musl/asm &&   sudo  ln -s /usr/include/asm-generic /usr/include/x86_64-linux-musl/asm-generic &&   sudo  ln -s /usr/include/linux /usr/include/x86_64-linux-musl/linux

mkdir $GITHUB_WORKSPACE/musl
mkdir $GITHUB_WORKSPACE/cache
cd $GITHUB_WORKSPACE/cache
wget https://github.com/openssl/openssl/archive/OpenSSL_1_1_1f.tar.gz
tar zxvf OpenSSL_1_1_1f.tar.gz 
cd openssl-OpenSSL_1_1_1f/
CC="musl-gcc -fPIE -pie" ./Configure no-shared no-async --prefix=$GITHUB_WORKSPACE/musl --openssldir=$GITHUB_WORKSPACE/musl/ssl linux-x86_64
make depend
make -j$(nproc)
make install
