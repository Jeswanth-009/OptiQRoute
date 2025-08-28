import itertools
import time
import numpy as np
import matplotlib.pyplot as plt

from qiskit import Aer, execute
from qiskit.circuit.library import TwoLocal
from qiskit_optimization.applications import Tsp
from qiskit_optimization.converters import QuadraticProgramToQubo
from qiskit_optimization.algorithms import MinimumEigenOptimizer
from qiskit.algorithms import QAOA

# Classical Held-Karp (Exact TSP)
def held_karp_tsp(dist_matrix):
    n = len(dist_matrix)
    C = {}
    for k in range(1, n):
        C[(1 << k, k)] = (dist_matrix[0][k], 0)
    for subset_size in range(2, n):
        for subset in itertools.combinations(range(1, n), subset_size):
            bits = 0
            for bit in subset:
                bits |= 1 << bit
            for k in subset:
                prev = bits & ~(1 << k)
                res = []
                for m in subset:
                    if m == 0 or m == k:
                        continue
                    res.append((C[(prev, m)][0] + dist_matrix[m][k], m))
                C[(bits, k)] = min(res)
    bits = (2**n - 1) - 1
    res = []
    for k in range(1, n):
        res.append((C[(bits, k)][0] + dist_matrix[k][0], k))
    opt, parent = min(res)
    return opt

# Quantum QAOA TSP (works best for small cities)
def qaoa_tsp(num_cities):
    coords = np.random.rand(num_cities, 2)
    tsp = Tsp.create_random_instance(num_cities, seed=42)
    qp = tsp.to_quadratic_program()
    qubo = QuadraticProgramToQubo().convert(qp)
    backend = Aer.get_backend("qasm_simulator")
    qaoa = QAOA(optimizer=None, reps=1, quantum_instance=backend)
    solver = MinimumEigenOptimizer(qaoa)
    result = solver.solve(qubo)
    return result

# Compare runtimes
classical_times = []
quantum_times = []
city_sizes = list(range(5, 16, 2))

for n in city_sizes:
    # Classical timing
    dist_matrix = np.random.randint(1, 20, size=(n, n))
    np.fill_diagonal(dist_matrix, 0)
    start = time.time()
    opt_length = held_karp_tsp(dist_matrix)
    end = time.time()
    classical_times.append(end - start)
    print(f"Classical TSP with {n} cities took {end - start:.4f} seconds, Opt length = {opt_length}")

# Quantum timing (only up to ~8 cities, beyond this too big for simulator)
for n in [4, 6, 8]:
    start = time.time()
    result = qaoa_tsp(n)
    end = time.time()
    quantum_times.append((n, end - start))
    print(f"Quantum QAOA TSP with {n} cities took {end - start:.4f} seconds")

# Plot classical runtimes
plt.figure(figsize=(8,5))
plt.plot(city_sizes, classical_times, marker='o', label='Classical Held-Karp')

# Plot quantum runtimes
q_sizes = [x[0] for x in quantum_times]
q_vals = [x[1] for x in quantum_times]
plt.plot(q_sizes, q_vals, marker='s', label='Quantum QAOA (simulator)')

plt.xlabel("Number of Cities")
plt.ylabel("Execution Time (seconds)")
plt.title("Classical vs Quantum TSP Runtime Growth")
plt.legend()
plt.grid(True)
plt.show()
