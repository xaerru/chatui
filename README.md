# chatui

Simple chat application.

You'll need both [`chatui_server`](https://crates.io/crates/chatui_server) and [`chatui_client`](https://crates.io/crates/chatui_client) to run this application.

## Installation

### With cargo

```bash
cargo intall chatui_server chatui_client
```

Run the server with executable `chatui_server` and run the client with executable `chatui_client`.

### Without cargo

Clone the project:

```bash
git clone https://github.com/grvxs/chatui.git
```

Move to the project directory:

```bash
cd chatui
```

Run the server (Rust):

```bash
cargo run --release --bin chatui_server
```

Run the server (Node.js):

```bash
cd chatui_server_node
node index.js
```

Run the client:

```bash
cargo run --release --bin chatui_client
```

## Screenshots

![image](https://user-images.githubusercontent.com/65955464/124386731-6090c580-dcf9-11eb-9be5-8c2c07075dde.png)
