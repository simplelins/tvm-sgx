# tvm-sgx
copy from tvm sgx,and can make more the one model compiling in a app
## Setup
1. [Install the Fortanix Enclave Development Platform](https://edp.fortanix.com/docs/installation/guide/)
2. `rustup component add llvm-tools-preview` to get `llvm-ar` and `llvm-objcopy`
3. `pip install numpy decorator psutil`
4. `cargo run` to start the enclave TCP server
5. Send a 28x28 "image" to the enclave model server using `head -c $((28*28*4)) /dev/urandom | nc 127.0.0.1 4242 | python read_results.py`
