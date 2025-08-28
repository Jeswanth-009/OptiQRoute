# Quantum_computing.py
import networkx as nx
import time
from qiskit import QuantumCircuit
from qiskit_aer import AerSimulator

def create_graph(n=7):
    G = nx.complete_graph(n)
    for u, v in G.edges():
        G[u][v]['weight'] = abs(u - v) + 1
    return G

def classical_route(G):
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

def quantum_demo():
    # Minimal quantum demo for timing
    qc = QuantumCircuit(3, 3)
    qc.h(0)
    qc.cx(0, 1)
    qc.cx(1, 2)
    qc.measure_all()

    backend = AerSimulator()
    t0 = time.time()
    job = backend.run(qc)        # <-- modern way
    result = job.result()
    t1 = time.time()
    print("Quantum measurement counts:", result.get_counts())
    print("Quantum execution time:", round(t1 - t0, 4), "seconds")

if __name__ == "__main__":
    G = create_graph()
    t0 = time.time()
    route = classical_route(G)
    t1 = time.time()
    print("Classical route:", route)
    print("Classical execution time:", round(t1 - t0, 4), "seconds")
    
    quantum_demo()
