# The Flying Dutchman Chess Engine

## About

"The Flying Dutchman" is a Chess Engine written in Rust, designed with a focus on safety and readability above all else. This project was created as an homage to my time spent at Hofstra University. I developed this project as both an educational tool and a challenge to create a sophisticated chess bot based on fundamental computer science principles.

While there are many robust and powerful chess engines out there (like Stockfish), the primary goal of "The Flying Dutchman" is not to outperform these engines, but to serve as a resource for those interested in understanding the underlying workings of a chess engine. This project is essentially a fusion of my passion for programming and the strategic complexity of chess, despite me being admittedly not very good at the game.

The engine's evaluation function is simple: it calculates the sum of the pieces and the number of attacks. This simplicity is a deliberate design choice, keeping in line with the project's educational focus.

## Project Structure

The project is organized as follows:

### Evaluate

This directory contains the evaluation function for the chess engine. It's responsible for calculating the value of a given chess board configuration.

### Search

The search directory contains the implementation of the Alpha-Beta Pruning and Quiescence Search algorithms, which are used to search through the game tree and find the best possible move.

### Tests

Contains the module for testing the chess engine against itself.

### Transposition

Here you can find the implementation of a transposition table, a common optimization technique used in chess engines to store board positions and avoid redundant computations.

### UCI

UCI (Universal Chess Interface) is a standard protocol for chess engines to communicate with user interfaces. This directory contains the code for interpreting and responding to UCI commands.

## Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request.

## License

"The Flying Dutchman" is available under the MIT License. See `LICENSE` for more details.
