This project is structured in 3 main components.

# 1. Philosopher
The philosopher is the main component of the project. It is responsible for the following tasks:
- **Communication**: The philosopher communicates with the other philosophers.
- **Eating**: The philosopher eats when it is possible.
- **Thinking**: The philosopher thinks when it is not eating.

# 2. Nikki_Log

Houjiru_DNS is a simple DNS server and Logger that resolves the philosopher's names to their IP addresses. It is responsible for the following tasks:
- **Resolving**: Resolves the philosopher's names to their IP addresses.
- **Caching**: Caches the philosopher's IP addresses.
- **Informing**: Informs the philosopher about the other philosopher's IP addresses.
- **Logging**: Logs the philosopher's actions.
- **Storing**: Stores the logs in a file.

# 3. Shared

The shared library is a library that contains the shared data structures and functions that are used by the philosopher and the Nikki_Log components.