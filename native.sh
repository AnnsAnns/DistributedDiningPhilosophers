#!/bin/bash

# Function to handle kill signal
cleanup() {
    echo "Terminating all background processes..."
    pkill -P $$
    wait
    exit 0
}

# Trap kill signal (SIGINT, SIGTERM)
trap cleanup SIGINT SIGTERM

# Start waiter service
export WAITER_IP="127.0.0.1"
export WAITER_PORT="3000"
export WAITER_HTTP_PORT="3001"
export VISITORS="3"
./target/debug/waiter &

# Wait for 3 seconds
echo "Waiting for waiter service to start..."
sleep 3

# Start cutlery service
for i in {1..3}; do
    export WAITER_IP="127.0.0.1"
    export WAITER_PORT="3000"
    ./target/debug/cutlery &
done

# Start philosopher service
for i in {1..3}; do
    export WAITER_IP="127.0.0.1"
    export WAITER_PORT="3000"
    ./target/debug/philosopher &
done

# Wait for all background processes to finish
wait
