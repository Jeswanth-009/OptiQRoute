# Quantum_vs_Classical_Routing.py
import networkx as nx
import matplotlib.pyplot as plt
import numpy as np
import time

from qiskit_optimization import QuadraticProgram
from qiskit_optimization.algorithms import MinimumEigenOptimizer
from qiskit.algorithms import QAOA
from qiskit.algorithms.optimizers import COBYLA
from qiskit_aer import AerSimulator

# --- Graph Creation ---
def create_graph(n=7):
    G = nx.complete_graph(n)
    pos = nx.circular_layout(G)  # nice circular layout for animation
    for u, v in G.edges():
        G[u][v]['weight'] = np.random.randint(1, 20)
    return G, pos

# --- Classical Nearest Neighbor ---
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

# --- Animate Route ---
def animate_route(G, pos, route, title="Route"):
    plt.figure(figsize=(6,6))
    nx.draw(G, pos, with_labels=True, node_color='lightblue', edge_color='gray')
    for i in range(len(route)-1):
        plt.plot([pos[route[i]][0], pos[route[i+1]][0]],
                 [pos[route[i]][1], pos[route[i+1]][1]],
                 'r', linewidth=2)
        plt.pause(0.5)
    plt.title(title)
    plt.show()
    # --- Compare Distances ---
def compute_total_distance(G, route):
    dist = 0
    for i in range(len(route)-1):
        dist += G[route[i]][route[i+1]]['weight']
    return dist



# --- Quantum QUBO + QAOA ---
def quantum_route(G):
    n = len(G.nodes)
    qp = QuadraticProgram()
    for i in range(n):
        qp.binary_var(name=f"x{i}")

    # Simple objective: minimize distance between consecutive nodes
    # Note: this is a demo QUBO for small graph
    linear = {f"x{i}": 0 for i in range(n)}
    quadratic = {}
    for i in range(n):
        for j in range(i+1, n):
            quadratic[(f"x{i}", f"x{j}")] = G[i][j]['weight']
    qp.minimize(linear=linear, quadratic=quadratic)

    backend = AerSimulator()
    qaoa = QAOA(optimizer=COBYLA(), reps=1, quantum_instance=backend)
    optimizer = MinimumEigenOptimizer(qaoa)
    result = optimizer.solve(qp)

    # Extract route from binary solution
    route = []
    for i in range(n):
        if result.x[i] > 0.5:
            route.append(i)
    if 0 not in route:
        route = [0] + route
    return route

# --- Main ---
if __name__ == "__main__":
    G, pos = create_graph()
    
    # Classical
    t0 = time.time()
    c_route = classical_route(G)
    t1 = time.time()
    print("Classical route:", c_route)
    print("Classical time:", round(t1 - t0, 4), "seconds")
    animate_route(G, pos, c_route, title="Classical Route")
    
    # Quantum
    t0 = time.time()
    q_route = quantum_route(G)
    t1 = time.time()
    print("Quantum route:", q_route)
    print("Quantum time:", round(t1 - t0, 4), "seconds")
    animate_route(G, pos, q_route, title="Quantum Route")
    c_distance = compute_total_distance(G, c_route)
    q_distance = compute_total_distance(G, q_route)

plt.figure(figsize=(6,4))
plt.bar(['Classical', 'Quantum'], [c_distance, q_distance], color=['blue', 'orange'])
plt.ylabel("Total Distance")
plt.title("Classical vs Quantum Route Distance")
for i, v in enumerate([c_distance, q_distance]):
    plt.text(i, v + 0.5, str(v), ha='center')
plt.show()
