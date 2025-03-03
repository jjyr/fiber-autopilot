import networkx as nx
from networkx.readwrite import json_graph
import sys
import json
import random

# define weight range
ONE_CKB = 100000000
WEIGHT_MIN = 0
WEIGHT_MAX = 1_000_000 * ONE_CKB


def main():
    # Generate a random graph
    G = nx.erdos_renyi_graph(100, 0.05)
    # Generate a random weight for each edge
    for edge in G.edges():
        G[edge[0]][edge[1]]["weight"] = random.uniform(WEIGHT_MIN, WEIGHT_MAX)
    # Convert the graph to a JSON object
    data = json_graph.node_link_data(G, edges="links")
    # get path from args
    if len(sys.argv) < 2:
        print("Usage: uv run main.py <path>")
        sys.exit(1)
    path = sys.argv[1]
    # Save the graph to a file
    with open(path, "w") as f:
        json.dump(data, f)
    print(f"Graph saved to {path}")



if __name__ == "__main__":
    main()
