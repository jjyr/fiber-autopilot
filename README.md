# Fiber Autopilot

Fiber Autopilot continuously analyzes peers in the network graph and recommends potential peers for the operated node to open channels with. The recommendations are currently based on the following heuristic strategies: `Centrality`, `Richness`, and `Random`.


## Run

``` sh
# Copy configure file
copy $PROJECT/fiber-autopilot.toml .
# Run
RUST_LOG=info,fiber_autopilot=debug cargo run
```
