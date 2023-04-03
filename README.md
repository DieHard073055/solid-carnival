# solid-carnival
### Crypto Exchange Simulator

This repository contains the source code for a cryptocurrency exchange simulator, specifically designed to replicate the behavior of Binance. The simulator uses real price data from Binance and allows you to run your trading strategies in a risk-free environment. By hosting an HTTP server, the simulator enables the creation of multiple exchange instances with unique wallets, providing a comprehensive testing platform for your trading algorithms.

### Features

- Simulates the Binance exchange with real price data
- Hosts an HTTP server for creating and managing exchange instances
- Supports wallet management and querying
- Customizable order types, including limit and market orders
- Allows for testing multiple trading strategies simultaneously
- Ideal for both live strategy testing and backtesting purposes

### Getting Started

1. Clone the repository: `git clone https://github.com/DieHard073055/solid-carnival.git`
2. Install dependencies: `cargo build`
3. Configure the simulator by editing the `config.toml` file with your preferred settings
4. Run the simulator: `cargo run`. `Not yet working`

### Usage

Once the simulator is up and running, you can interact with its API to create exchange instances, manage wallets, place orders, and retrieve the status of your trading strategies. Your external code can connect to the simulator's API to perform these actions and evaluate the results of your trading algorithms.

### Contributing

We welcome contributions from the community. If you would like to contribute to the development of this project, please follow these steps:

1. Fork the repository
2. Create a new branch with a descriptive name (e.g., `feature-add-new-feature`)
3. Make your changes and commit them with a clear and concise commit message
4. Open a pull request with a detailed description of your changes in markdown format

Please ensure that your code follows the existing style and structure of the project. If you have any questions or need assistance, feel free to open an issue or reach out to the maintainers.

### License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more information.

