# backend/app.py

import osmnx as ox
import networkx as nx
from flask import Flask, request, jsonify
from flask_cors import CORS

app = Flask(__name__)
CORS(app)

# Load Visakhapatnam graph data
G = ox.graph_from_place("Visakhapatnam, Andhra Pradesh, India", network_type="drive")

def get_nearest_node(point):
    # point = (lat, lon)
    return ox.distance.nearest_nodes(G, point[1], point)  # note lon, lat order here

def nodes_to_coords(nodes_list):
    return [(G.nodes[n]['y'], G.nodes[n]['x']) for n in nodes_list]

@app.route('/classical-route', methods=['POST'])
def classical_route():
    data = request.json
    start = data['start']
    deliveries = data['deliveries']

    start_node = get_nearest_node(start)
    delivery_nodes = [get_nearest_node(d) for d in deliveries]

    route_nodes = [start_node] + delivery_nodes + [start_node]

    route = []
    total_length = 0
    for i in range(len(route_nodes) - 1):
        path = nx.shortest_path(G, route_nodes[i], route_nodes[i+1], weight='length')
        length = nx.shortest_path_length(G, route_nodes[i], route_nodes[i+1], weight='length')
        route.extend(path[:-1])
        total_length += length
    route.append(route_nodes[-1])

    route_coords = nodes_to_coords(route)

    return jsonify({
        'route': route_coords,
        'distance_m': total_length
    })

@app.route('/quantum-route', methods=['POST'])
def quantum_route():
    data = request.json
    start = data['start']
    deliveries = data['deliveries']

    # Placeholder quantum optimizer - currently mimics classical route
    def quantum_optimizer(start, deliveries):
        start_node = get_nearest_node(start)
        delivery_nodes = [get_nearest_node(d) for d in deliveries]
        route_nodes = [start_node] + delivery_nodes + [start_node]

        route = []
        for i in range(len(route_nodes) - 1):
            path = nx.shortest_path(G, route_nodes[i], route_nodes[i+1], weight='length')
            route.extend(path[:-1])
        route.append(route_nodes[-1])
        return route

    quantum_route_nodes = quantum_optimizer(start, deliveries)
    route_coords = nodes_to_coords(quantum_route_nodes)

    return jsonify({
        'route': route_coords,
        'distance_m': 0  # Replace with real quantum computed distance if implemented
    })

if __name__ == "__main__":
    app.run(debug=True)
