# src/classical_routing.py
import networkx as nx
import matplotlib.pyplot as plt
import time
import random

def create_graph(n=500):
    """
    Create a weighted complete graph of n cities.
    Distances are random integers (1 to 100).
    """
    G = nx.complete_graph(n)
    for u, v in G.edges():
        G[u][v]['weight'] = random.randint(1, 100)
    return G

def nearest_neighbor_routing(G, start=0):
    """
    Nearest Neighbor heuristic: pick the closest unvisited city each step.
    """
    visited = [start]
    nodes = list(G.nodes)
    nodes.remove(start)
    current = start
    while nodes:
        next_node = min(nodes, key=lambda x: G[current][x]['weight'])
        visited.append(next_node)
        nodes.remove(next_node)
        current = next_node
    return visited

def plot_route(G, route, show_labels=False):
    """
    Plot only the final route edges over the city layout (no intermediate animation).
    Warning: 500 cities will look messy!
    """
    pos = nx.spring_layout(G, seed=42)  # layout for nodes
    plt.figure(figsize=(14, 14))

    # draw only nodes
    nx.draw_networkx_nodes(G, pos, node_size=50, node_color='lightblue')

    # optionally draw labels (will look chaotic for 500)
    if show_labels:
        nx.draw_networkx_labels(G, pos, font_size=5)

    # draw only the route edges
    route_edges = [(route[i], route[i+1]) for i in range(len(route)-1)]
    nx.draw_networkx_edges(G, pos, edgelist=route_edges, edge_color='red', width=0.8)

    plt.title("Nearest Neighbor Route Across 500 Cities")
    plt.axis("off")
    plt.show()

if __name__ == "__main__":
    G = create_graph(500)
    start_time = time.time()
    route = nearest_neighbor_routing(G)
    end_time = time.time()

    print("Route length (number of cities visited):", len(route))
    print("First 15 cities in route:", route[:15], "...")
    print("Time taken:", round(end_time - start_time, 4), "seconds")

    plot_route(G, route, show_labels=False)  # labels off for clarity
