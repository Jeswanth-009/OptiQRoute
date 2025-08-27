# src/classical_routing.py
import networkx as nx
import matplotlib.pyplot as plt
import time

def create_graph(n=7):
    G = nx.complete_graph(n)
    for u, v in G.edges():
        G[u][v]['weight'] = abs(u-v) + 1  # simple distances
    return G

def nearest_neighbor_routing(G, start=0):
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

def animate_route(G, route):
    pos = nx.spring_layout(G)
    nx.draw(G, pos, with_labels=True, node_color='lightblue')
    plt.pause(1)
    for i in range(len(route)-1):
        nx.draw_networkx_edges(G, pos, edgelist=[(route[i], route[i+1])],
                               edge_color='red', width=2)
        plt.pause(1)
    plt.show()

if __name__ == "__main__":
    G = create_graph()
    start_time = time.time()
    route = nearest_neighbor_routing(G)
    end_time = time.time()
    print("Classical route:", route)
    print("Time taken:", round(end_time - start_time, 4), "seconds")
    plt.figure(figsize=(6,6))
    animate_route(G, route)
