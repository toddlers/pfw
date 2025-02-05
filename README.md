# pfw

# local testing

1. start a simple python server

```bash
python3 -m http.server 3004
```

2. `cargo run` inside this dir, this forwards the port from 3003 to 3004

3. run a simple curl command like `curl http://localhost:3003
