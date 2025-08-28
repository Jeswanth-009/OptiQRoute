# Quantum_computing.py
import networkx as nx
import matplotlib.pyplot as plt
import time
import random
from qiskit import QuantumCircuit
from qiskit_aer import AerSimulator

def create_graph(n=500):
    """
    Create a weighted complete graph of n cities.
    Distances are random integers (1–100).
    """
    G = nx.complete_graph(n)
    for u, v in G.edges():
        G[u][v]['weight'] = random.randint(1, 100)
    return G

def classical_route(G):
    """
    Nearest Neighbor heuristic for routing.
    """
    visited = [0]
    nodes = list(G.nodes)
    nodes.remove(0)
    current = 0
    while nodes:
        next_node = min(nodes, key=lambda x: G[current][x]['weight'])
        visited.append(next_node)
        nodes.remove(next_node)
        current = next_node
    return visited

def plot_route(G, route, show_labels=False):
    """
    Plot only nodes + the final route edges.
    """
    pos = nx.spring_layout(G, seed=42)  # layout for visualization
    plt.figure(figsize=(14, 14))

    # draw nodes
    nx.draw_networkx_nodes(G, pos, node_size=50, node_color='lightblue')

    # optional labels (skipped for clarity with 500 cities)
    if show_labels:
        nx.draw_networkx_labels(G, pos, font_size=5)

    # draw route edges
    route_edges = [(route[i], route[i+1]) for i in range(len(route)-1)]
    nx.draw_networkx_edges(G, pos, edgelist=route_edges, edge_color='red', width=0.8)

    plt.title("Nearest Neighbor Route Across 500 Cities")
    plt.axis("off")
    plt.show()

def quantum_demo():
    """
    Minimal quantum circuit to show timing and execution.
    (3 qubits only — can't scale to 500).
    """
    qc = QuantumCircuit(3, 3)
    qc.h(0)
    qc.cx(0, 1)
    qc.cx(1, 2)
    qc.measure_all()

    backend = AerSimulator()
    t0 = time.time()
    job = backend.run(qc)
    result = job.result()
    t1 = time.time()

    print("\nQuantum measurement counts:", result.get_counts())
    print("Quantum execution time:", round(t1 - t0, 4), "seconds")

if __name__ == "__main__":
    # Classical 500-city routing
    G = create_graph(500)
    t0 = time.time()
    route = classical_route(G)
    t1 = time.time()

    print("Classical route length:", len(route))
    print("First 15 cities in route:", route[:15], "...")
    print("Classical execution time:", round(t1 - t0, 4), "seconds")

    # Plot classical route
    plot_route(G, route, show_labels=False)

    # Quantum demo
    quantum_demo()
