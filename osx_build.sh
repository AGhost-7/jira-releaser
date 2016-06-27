# Building on osx is a bit wierd... It has a hard time finding the header
# files needed for openssl.
cargo clean
OPENSSL_INCLUDE_DIR=/usr/local/opt/openssl/include \
	DEP_OPENSSL_INCLUDE=/usr/local/opt/openssl/include \
	cargo build
