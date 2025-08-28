import numpy as np
import time
from docplex.mp.model import Model
from qiskit_optimization.translators import from_docplex_mp
from qiskit_algorithms import QAOA
from qiskit_algorithms.optimizers import COBYLA
from qiskit.primitives import Sampler
from qiskit_optimization.algorithms import MinimumEigenOptimizer

# Distance matrix for 4 cities
distance_matrix = np.array([
    [0, 10, 15, 20],
    [10, 0, 35, 25],
    [15, 35, 0, 30],
    [20, 25, 30, 0]
])

n = len(distance_matrix)
mdl = Model(name="TSP")

# Decision variables: x[i][j] = 1 if city i is at position j
x = {(i, j): mdl.binary_var(name=f"x_{i}_{j}") for i in range(n) for j in range(n)}

# Each city appears once
for i in range(n):
    mdl.add_constraint(mdl.sum(x[i, j] for j in range(n)) == 1)

# Each position has one city
for j in range(n):
    mdl.add_constraint(mdl.sum(x[i, j] for i in range(n)) == 1)

# Objective: minimize distance
objective = 0
for i in range(n):
    for j in range(n):
        for k in range(n):
            objective += distance_matrix[i][j] * x[i, k] * x[j, (k+1)%n]

mdl.minimize(objective)

# Convert to QUBO
qp = from_docplex_mp(mdl)

# Setup QAOA
sampler = Sampler()
qaoa = QAOA(sampler=sampler, optimizer=COBYLA(maxiter=50), reps=1)
solver = MinimumEigenOptimizer(qaoa)

start = time.time()
result = solver.solve(qp)
end = time.time()

print("Quantum QAOA Solution")
print("Best Path:", result.x)
print("Objective Value (Cost):", result.fval)
print("Time Taken: %.5f seconds" % (end - start))
