# How to play with floodr

Compile floodr:

```
cargo build --release
```

### Example 1 (Delayed responses)

Start a Node HTTP server from `server` directory in another terminal:

```
cd example/server
DELAY_MS=100 node server.js
```

and then run:

```
cd example
../target/release/floodr --benchmark benchmark.yml
```

### Example 2 (Cookies)

Start a Node HTTP server from `server` directory in another terminal:

```
cd example/server
npm install
node server.js
```

and then run:

```
cd example
../target/release/floodr --benchmark cookies.yml
```

### Example 3 (Custom headers)

Start a Node HTTP server from `server` directory in another terminal:

```
cd example/server
npm install
node server.js
```

and then run:

```
cd example
../target/release/floodr --benchmark headers.yml
