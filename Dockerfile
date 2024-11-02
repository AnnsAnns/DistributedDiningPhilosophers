# Use the official Rust image from the Docker Hub
FROM rust:latest

# Set the working directory inside the container
WORKDIR /usr/src/vsp

# Copy the current directory contents into the container at /usr/src/myapp
COPY . .

# Build the Rust program
RUN cargo build --release

# Run the Rust program
CMD ["./target/release/vsp"]