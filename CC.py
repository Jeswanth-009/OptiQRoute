import itertools
import numpy as np
import time

# Distance matrix for 4 cities
distance_matrix = np.array([
    [0, 10, 15, 20],
    [10, 0, 35, 25],
    [15, 35, 0, 30],
    [20, 25, 30, 0]
])

def total_distance(path, dist_matrix):
    distance = 0
    for i in range(len(path) - 1):
        distance += dist_matrix[path[i]][path[i+1]]
    distance += dist_matrix[path[-1]][path[0]]  # return to start
    return distance

start = time.time()
cities = range(len(distance_matrix))
best_path, best_cost = None, float('inf')

for perm in itertools.permutations(cities):
    cost = total_distance(perm, distance_matrix)
    if cost < best_cost:
        best_path, best_cost = perm, cost

end = time.time()

print("Classical Brute Force Solution")
print("Best Path:", best_path)
print("Best Cost:", best_cost)
print("Time Taken: %.5f seconds" % (end - start))
