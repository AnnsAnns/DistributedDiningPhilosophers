# Use the official Rust image from the Docker Hub
FROM rust:latest

# Set the working directory inside the container
WORKDIR /usr/src/cutlery

# Copy the current directory contents into the container at /usr/src/myapp
COPY . .

# Remove the .env file if it exists
RUN rm .env 

# Build the Rust program
RUN cargo build --release --package cutlery

# Run the Rust program
CMD ["./target/release/cutlery"]