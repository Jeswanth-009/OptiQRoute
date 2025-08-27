import numpy as np
import matplotlib.pyplot as plt
import matplotlib.animation as animation
import itertools
import time

# Qiskit imports for the quantum solver
from qiskit_optimization import QuadraticProgram
from qiskit_optimization.algorithms import MinimumEigenOptimizer
from qiskit_algorithms.minimum_eigensolvers import QAOA
from qiskit_algorithms.optimizers import COBYLA
from qiskit_primitives import Sampler
from qiskit_aer import AerSimulator

def generate_customers(num_cities, seed=42):
    """Generates random (x, y) coordinates for a given number of cities."""
    np.random.seed(seed)
    return [(np.random.randint(0, 10), np.random.randint(0, 10)) for _ in range(num_cities)]

def euclidean_distance(city1, city2):
    """Calculates the Euclidean distance between two points."""
    return np.linalg.norm(np.array(city1) - np.array(city2))

def factorial(n):
    """Calculates the factorial of a number."""
    if n == 0:
        return 1
    return n * factorial(n - 1)

def plot_solution(customers, tour, title, num_cities, possible_routes, time_taken, solver_type):
    """Animates the plotting of the VRP solution, showing all possible routes and then highlighting the optimal one."""
    fig, ax = plt.subplots(figsize=(10, 8))

    metric_text = (
        f"Solver: {solver_type}\n"
        f"Cities: {num_cities}\n"
        f"Possible Routes: {possible_routes}\n"
        f"Time Taken: {time_taken:.3f} ms\n"
        f"Optimized Path Length: {calculate_tour_length(customers, tour):.2f}"
    )
    fig.suptitle(title, fontsize=16)
    ax.text(
        0.5, 1.05, metric_text,
        transform=ax.transAxes,
        fontsize=10,
        verticalalignment='top',
        horizontalalignment='center'
    )

    x_coords = [c[0] for c in customers]
    y_coords = [c[1] for c in customers]
    ax.scatter(x_coords, y_coords, color='red', s=100, zorder=5)
    for i, (x, y) in enumerate(customers):
        ax.text(x + 0.2, y + 0.2, f'C{i}', horizontalalignment='center', verticalalignment='bottom')

    ax.set_xlabel("X Coordinate")
    ax.set_ylabel("Y Coordinate")
    ax.set_xlim(-1, 11)
    ax.set_ylim(-1, 11)
    ax.grid(True)
    ax.set_aspect('equal', adjustable='box')

    # Draw all possible routes
    if num_cities > 1:
        all_permutations = list(itertools.permutations(range(num_cities)))
        for p in all_permutations:
            full_p = list(p) + [p[0]]
            for i in range(len(p)):
                start_city = customers[full_p[i]]
                end_city = customers[full_p[i + 1]]
                ax.plot(
                    [start_city[0], end_city[0]],
                    [start_city[1], end_city[1]],
                    'k-', alpha=0.1, lw=1
                )

    # Highlight optimized path
    edges = []
    if num_cities > 1:
        full_tour = list(tour) + [tour[0]]
        for i in range(len(tour)):
            edges.append((full_tour[i], full_tour[i + 1]))
    lines = [ax.plot([], [], 'b-', lw=3)[0] for _ in edges]

    def update(frame):
        if frame < len(edges):
            start_idx, end_idx = edges[frame]
            start_city = customers[start_idx]
            end_city = customers[end_idx]
            lines[frame].set_data(
                [start_city[0], end_city[0]],
                [start_city[1], end_city[1]]
            )
        return lines

    ani = animation.FuncAnimation(fig, update, frames=len(edges), interval=500, blit=True)
    plt.tight_layout()
    plt.show()

def calculate_tour_length(customers, tour):
    """Calculates the total length of a given tour, including the return to the start."""
    if len(tour) < 2:
        return 0
    total = 0
    for i in range(len(tour)):
        total += euclidean_distance(
            customers[tour[i]],
            customers[tour[(i + 1) % len(tour)]]
        )
    return total

def solve_quantum(customers):
    """Solves VRP using the Minimum Eigen Optimizer with QAOA on a simulator."""
    num_cities = len(customers)
    start_time = time.time()
    if num_cities <= 1:
        end_time = time.time()
        return [0], 1, (end_time - start_time) * 1000

    qp = QuadraticProgram()
    for i in range(num_cities):
        for j in range(num_cities):
            if i != j:
                qp.binary_var(name=f"x_{i}_{j}")

    distance_matrix = np.zeros((num_cities, num_cities))
    for i in range(num_cities):
        for j in range(num_cities):
            if i != j:
                distance_matrix[i, j] = euclidean_distance(customers[i], customers[j])

    linear = {f"x_{i}_{j}": distance_matrix[i, j] for i in range(num_cities) for j in range(num_cities) if i != j}
    qp.minimize(linear=linear)

    for i in range(num_cities):
        qp.linear_constraint(
            linear={f"x_{i}_{j}": 1 for j in range(num_cities) if i != j},
            sense="==",
            rhs=1,
            name=f"leave_{i}"
        )
    for j in range(num_cities):
        qp.linear_constraint(
            linear={f"x_{i}_{j}": 1 for i in range(num_cities) if i != j},
            sense="==",
            rhs=1,
            name=f"enter_{j}"
        )

    sampler = Sampler()
    optimizer = COBYLA(maxiter=50)
    qaoa = QAOA(optimizer=optimizer, reps=1, sampler=sampler)
    algo = MinimumEigenOptimizer(min_eigen_solver=qaoa)
    result = algo.solve(qp)

    end_time = time.time()
    time_taken = (end_time - start_time) * 1000

    solution_nodes = [
        int(v.split('_')[1])
        for v, val in result.variables_dict.items() if val > 0.5
    ]
    tour = sorted(set(solution_nodes)) if solution_nodes else []
    if len(tour) != num_cities:
        unvisited = set(range(num_cities))
        tour = []
        current = unvisited.pop()
        tour.append(current)
        while unvisited:
            nearest = min(
                unvisited,
                key=lambda n: euclidean_distance(customers[current], customers[n])
            )
            unvisited.remove(nearest)
            tour.append(nearest)
            current = nearest

    possible_routes = factorial(num_cities)
    return tour, possible_routes, time_taken

if __name__ == "__main__":
    try:
        num_cities = int(input("Enter the number of cities (1-6): "))
        if num_cities < 1 or num_cities > 6:
            print("Number of cities must be between 1 and 6.")
        else:
            customers = generate_customers(num_cities)
            print("\n--- Running Quantum QAOA Solver (on Simulator) ---")
            tour, possible_routes, time_taken = solve_quantum(customers)
            plot_solution(
                customers,
                tour,
                "Quantum Solution (QAOA)",
                num_cities,
                possible_routes,
                time_taken,
                'Quantum'
            )
    except ValueError:
        print("Invalid input. Please enter a valid integer.")
