#!/bin/bash

echo "Running cargo build for x86_64-unknown-linux-gnu..."
/home/ulthirm/.cargo/bin/cargo build --release --target x86_64-unknown-linux-gnu
if [ $? -eq 0 ]; then
    echo "Success: Cargo build for x86_64-unknown-linux-gnu completed."
else
    echo "Error: Cargo build for x86_64-unknown-linux-gnu failed."
    exit 1
fi

echo "Running cargo build for x86_64-unknown-linux-musl..."
/home/ulthirm/.cargo/bin/cargo build --target x86_64-unknown-linux-musl --release
if [ $? -eq 0 ]; then
    echo "Success: Cargo build for x86_64-unknown-linux-musl completed."
else
    echo "Error: Cargo build for x86_64-unknown-linux-musl failed."
    exit 1
fi

# Uncomment and modify the following lines for additional targets
# echo "Running cargo build for another-target..."
# /home/ulthirm/.cargo/bin/cargo build --release --target another-target
# if [ $? -eq 0 ]; then
#     echo "Success: Cargo build for another-target completed."
# else
#     echo "Error: Cargo build for another-target failed."
#     exit 1
# fi
