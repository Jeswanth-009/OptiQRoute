# VRP Solver Web Server API Documentation

A comprehensive Vehicle Routing Problem (VRP) solver with OpenStreetMap integration, providing REST APIs for solving logistics optimization problems.

## Table of Contents

- [Getting Started](#getting-started)
- [Authentication](#authentication)
- [Base URL](#base-url)
- [Common Data Types](#common-data-types)
- [Error Handling](#error-handling)
- [API Endpoints](#api-endpoints)
  - [Health & Status](#health--status)
  - [OSM Data Management](#osm-data-management)
  - [Location Mapping](#location-mapping)
  - [VRP Instance Management](#vrp-instance-management)
  - [Solution Management](#solution-management)
- [Complete Workflow Example](#complete-workflow-example)
- [Rate Limits](#rate-limits)

## Getting Started

### Prerequisites
- Rust 1.70+ installed
- OSM PBF files or access to OSM data URLs

### Starting the Server

```bash
# Clone and build
git clone <repository>
cd vrp_solver
cargo build --release

# Run the web server
cargo run --bin vrp-server

# Or with custom configuration
PORT=8080 HOST=127.0.0.1 cargo run --bin vrp-server
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `3000` | Server port |
| `HOST` | `0.0.0.0` | Server host |
| `RUST_LOG` | `vrp_solver=info,tower_http=debug` | Logging level |

## Authentication

Currently, no authentication is required. All endpoints are publicly accessible.

## Base URL

```
http://localhost:3000
```

## Common Data Types

### LocationCoordinate
```json
{
  "lat": 17.735,
  "lon": 83.315,
  "name": "Optional location name"
}
```

### MappedLocation
```json
{
  "node_id": 3688822252,
  "lat": 17.7351,
  "lon": 83.3149,
  "distance_to_original": 12.5
}
```

### VrpConstraints
```json
{
  "time_windows": false,
  "max_distance": 50000.0,
  "max_duration": 14400.0,
  "service_time": 300.0
}
```

### SolverAlgorithm
Available values:
- `"greedy"` - Greedy Nearest Neighbor (nearest start)
- `"greedy_farthest"` - Greedy Nearest Neighbor (farthest start)
- `"clarke_wright"` - Clarke-Wright Savings Algorithm
- `"multi_start"` - Multi-Start Metaheuristic (best of all)

## Error Handling

All endpoints return JSON error responses with appropriate HTTP status codes.

### Error Response Format
```json
{
  "error": "error_code",
  "message": "Human-readable error description",
  "details": "Optional detailed error information"
}
```

### Common HTTP Status Codes

| Code | Description |
|------|-------------|
| `200` | Success |
| `400` | Bad Request - Invalid input or constraint violation |
| `404` | Not Found - Resource doesn't exist |
| `500` | Internal Server Error |

### Example Error Response
```json
{
  "error": "not_found",
  "message": "VRP instance not found",
  "details": "Instance with ID 550e8400-e29b-41d4-a716-446655440000 does not exist"
}
```

---

## API Endpoints

## Health & Status

### Health Check
Get server health status and statistics.

**Endpoint:** `GET /health`

**Response:**
```json
{
  "status": "healthy",
  "timestamp": 1735083600,
  "stats": {
    "graphs": 5,
    "mappings": 3,
    "vrp_instances": 2,
    "solutions": 4
  }
}
```

### Application Statistics
Get detailed application statistics.

**Endpoint:** `GET /stats`

**Response:**
```json
{
  "graphs": 5,
  "mappings": 3,
  "vrp_instances": 2,
  "solutions": 4
}
```

---

## OSM Data Management

### Upload OSM Data
Upload and parse OpenStreetMap data from file or URL.

**Endpoint:** `POST /osm/upload`

**Content-Type:** `multipart/form-data`

#### Option A: File Upload

**Request Body:**
```bash
# Using curl
curl -X POST \
  -F 'file=@your-map.osm.pbf' \
  http://localhost:3000/osm/upload
```

#### Option B: URL Download

**Request Body:**
```bash
# Using curl
curl -X POST \
  -F 'file_url=https://download.geofabrik.de/asia/india/andhra-pradesh-latest.osm.pbf' \
  http://localhost:3000/osm/upload
```

#### Using JavaScript Fetch API

```javascript
// File upload
const formData = new FormData();
formData.append('file', fileInput.files[0]);

const response = await fetch('/osm/upload', {
  method: 'POST',
  body: formData
});

// URL download
const formData = new FormData();
formData.append('file_url', 'https://example.com/map.osm.pbf');

const response = await fetch('/osm/upload', {
  method: 'POST',
  body: formData
});
```

**Response:**
```json
{
  "graph_id": "550e8400-e29b-41d4-a716-446655440000",
  "nodes": 125000,
  "edges": 85000,
  "message": "Successfully parsed OSM data: 125000 nodes, 85000 ways"
}
```

**Response Fields:**
- `graph_id`: UUID to reference this OSM dataset in future requests
- `nodes`: Number of road intersections/nodes parsed
- `edges`: Number of road segments/ways parsed
- `message`: Success message with parsing details

---

## Location Mapping

### Map Depot and Customers to Road Network
Convert latitude/longitude coordinates to the nearest road intersections in the OSM graph.

**Endpoint:** `POST /vrp/map`

**Content-Type:** `application/json`

**Request Body:**
```json
{
  "graph_id": "550e8400-e29b-41d4-a716-446655440000",
  "depot": {
    "lat": 17.735,
    "lon": 83.315,
    "name": "Main Depot"
  },
  "customers": [
    {
      "lat": 17.737,
      "lon": 83.320,
      "name": "Customer 1"
    },
    {
      "lat": 17.740,
      "lon": 83.310,
      "name": "Customer 2"
    },
    {
      "lat": 17.733,
      "lon": 83.318,
      "name": "Customer 3"
    }
  ]
}
```

**Request Fields:**
- `graph_id`: UUID of the uploaded OSM dataset
- `depot`: Depot location coordinates
- `customers`: Array of customer location coordinates

**Response:**
```json
{
  "mapped_depot": {
    "node_id": 3688822252,
    "lat": 17.7351,
    "lon": 83.3149,
    "distance_to_original": 12.5
  },
  "mapped_customers": [
    {
      "node_id": 987654321,
      "lat": 17.7371,
      "lon": 83.3201,
      "distance_to_original": 8.3
    },
    {
      "node_id": 123456789,
      "lat": 17.7401,
      "lon": 83.3099,
      "distance_to_original": 15.7
    },
    {
      "node_id": 456789123,
      "lat": 17.7331,
      "lon": 83.3179,
      "distance_to_original": 6.2
    }
  ]
}
```

**Response Fields:**
- `mapped_depot`: Depot mapped to nearest road intersection
- `mapped_customers`: Array of customers mapped to nearest road intersections
- `node_id`: OSM node ID of the road intersection
- `lat`, `lon`: Exact coordinates of the road intersection
- `distance_to_original`: Distance in meters from original coordinates to mapped intersection

---

## VRP Instance Management

### Generate VRP Instance
Create a Vehicle Routing Problem instance from mapped locations.

**Endpoint:** `POST /vrp/generate`

**Content-Type:** `application/json`

**Request Body:**
```json
{
  "graph_id": "550e8400-e29b-41d4-a716-446655440000",
  "vehicles": 3,
  "capacity": 50.0,
  "constraints": {
    "time_windows": false,
    "max_distance": 100000.0,
    "max_duration": 14400.0,
    "service_time": 300.0
  }
}
```

**Request Fields:**
- `graph_id`: UUID of the OSM dataset (must have mapped locations)
- `vehicles`: Number of vehicles available
- `capacity`: Vehicle capacity (units depend on your demand units)
- `constraints`: Operational constraints
  - `time_windows`: Whether to enforce time windows (currently false)
  - `max_distance`: Maximum distance per vehicle in meters (optional)
  - `max_duration`: Maximum duration per vehicle in seconds (optional)  
  - `service_time`: Service time at each customer in seconds (optional, default: 300)

**Response:**
```json
{
  "vrp_id": "650e8400-e29b-41d4-a716-446655440000",
  "customers": 10,
  "vehicles": 3,
  "depot_count": 1
}
```

**Response Fields:**
- `vrp_id`: UUID to reference this VRP instance for solving
- `customers`: Number of customer locations
- `vehicles`: Number of vehicles configured
- `depot_count`: Number of depot locations (currently always 1)

---

## Solution Management

### Solve VRP Instance
Solve a VRP instance using the specified algorithm.

**Endpoint:** `POST /vrp/solve`

**Content-Type:** `application/json`

**Request Body:**
```json
{
  "vrp_id": "650e8400-e29b-41d4-a716-446655440000",
  "algorithm": "multi_start"
}
```

**Request Fields:**
- `vrp_id`: UUID of the VRP instance to solve
- `algorithm`: Algorithm to use (see [SolverAlgorithm](#solveralgorithm))

**Response:**
```json
{
  "solution_id": "750e8400-e29b-41d4-a716-446655440000",
  "routes": [
    {
      "vehicle_id": 0,
      "path": [1, 2, 3],
      "distance": 2500.0,
      "duration": 3600.0,
      "demand": 45.0,
      "locations": [
        {
          "id": 1,
          "name": "Customer 1",
          "lat": 17.737,
          "lon": 83.320,
          "demand": 15.0,
          "service_time": 300.0
        },
        {
          "id": 2,
          "name": "Customer 2", 
          "lat": 17.740,
          "lon": 83.310,
          "demand": 20.0,
          "service_time": 300.0
        },
        {
          "id": 3,
          "name": "Customer 3",
          "lat": 17.733,
          "lon": 83.318,
          "demand": 10.0,
          "service_time": 300.0
        }
      ]
    },
    {
      "vehicle_id": 1,
      "path": [4, 5],
      "distance": 1800.0,
      "duration": 2400.0,
      "demand": 25.0,
      "locations": [
        {
          "id": 4,
          "name": "Customer 4",
          "lat": 17.742,
          "lon": 83.325,
          "demand": 12.0,
          "service_time": 300.0
        },
        {
          "id": 5,
          "name": "Customer 5",
          "lat": 17.738,
          "lon": 83.312,
          "demand": 13.0,
          "service_time": 300.0
        }
      ]
    }
  ],
  "total_cost": 4300.0,
  "total_distance": 4300.0,
  "total_duration": 6000.0,
  "vehicles_used": 2,
  "solve_time_ms": 45.2
}
```

**Response Fields:**
- `solution_id`: UUID to reference this solution
- `routes`: Array of vehicle routes
  - `vehicle_id`: ID of the vehicle assigned to this route
  - `path`: Array of location IDs in visiting order
  - `distance`: Total distance for this route in meters
  - `duration`: Total duration for this route in seconds
  - `demand`: Total demand served by this route
  - `locations`: Detailed information about each location in the route
- `total_cost`: Total solution cost (currently same as total_distance)
- `total_distance`: Sum of all route distances in meters
- `total_duration`: Sum of all route durations in seconds
- `vehicles_used`: Number of vehicles actually used
- `solve_time_ms`: Time taken to solve in milliseconds

### Get Solution Details
Retrieve full details of a solved VRP instance.

**Endpoint:** `GET /vrp/solution/{solution_id}`

**Path Parameters:**
- `solution_id`: UUID of the solution

**Response:**
```json
{
  "id": "750e8400-e29b-41d4-a716-446655440000",
  "vrp_id": "650e8400-e29b-41d4-a716-446655440000",
  "solution": {
    "routes": [...],
    "total_distance": 4300.0,
    "total_duration": 6000.0,
    "num_vehicles_used": 2
  },
  "algorithm": "multi_start",
  "solve_time_ms": 45.2,
  "created_at": 1735083600
}
```

### Export Solution
Export solution in various formats for visualization or further processing.

**Endpoint:** `GET /vrp/solution/{solution_id}/export`

**Path Parameters:**
- `solution_id`: UUID of the solution

**Query Parameters:**
- `format`: Export format (`json` or `geojson`)

#### JSON Export

**Request:**
```bash
GET /vrp/solution/750e8400-e29b-41d4-a716-446655440000/export?format=json
```

**Response:** Standard solution JSON (same as GET solution details)

#### GeoJSON Export

**Request:**
```bash
GET /vrp/solution/750e8400-e29b-41d4-a716-446655440000/export?format=geojson
```

**Response:**
```json
{
  "type": "FeatureCollection",
  "features": [
    {
      "type": "Feature",
      "geometry": {
        "type": "LineString",
        "coordinates": [
          [83.315, 17.735],
          [83.320, 17.737],
          [83.310, 17.740],
          [83.318, 17.733],
          [83.315, 17.735]
        ]
      },
      "properties": {
        "route_id": 1,
        "vehicle_id": 0,
        "distance": 2500.0,
        "duration": 3600.0,
        "demand": 45.0
      }
    },
    {
      "type": "Feature",
      "geometry": {
        "type": "Point",
        "coordinates": [83.315, 17.735]
      },
      "properties": {
        "id": 0,
        "name": "Main Depot",
        "demand": 0.0,
        "type": "depot"
      }
    },
    {
      "type": "Feature",
      "geometry": {
        "type": "Point",
        "coordinates": [83.320, 17.737]
      },
      "properties": {
        "id": 1,
        "name": "Customer 1",
        "demand": 15.0,
        "type": "customer"
      }
    }
  ]
}
```

**GeoJSON Features:**
- **LineString features**: Vehicle routes with properties (route_id, vehicle_id, distance, duration, demand)
- **Point features**: Locations with properties (id, name, demand, type)

This GeoJSON can be directly loaded into mapping libraries like Leaflet, Mapbox GL JS, or OpenLayers for visualization.

---

## Complete Workflow Example

Here's a complete example showing how to use the API to solve a VRP problem:

### Step 1: Upload OSM Data

```bash
curl -X POST \
  -F 'file=@andhra-pradesh-latest.osm.pbf' \
  http://localhost:3000/osm/upload
```

**Response:**
```json
{
  "graph_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "nodes": 89234,
  "edges": 52341,
  "message": "Successfully parsed OSM data: 89234 nodes, 52341 ways"
}
```

### Step 2: Map Locations

```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "graph_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "depot": {
      "lat": 17.735,
      "lon": 83.315,
      "name": "Distribution Center"
    },
    "customers": [
      {"lat": 17.737, "lon": 83.320, "name": "Store A"},
      {"lat": 17.740, "lon": 83.310, "name": "Store B"},
      {"lat": 17.733, "lon": 83.318, "name": "Store C"},
      {"lat": 17.742, "lon": 83.325, "name": "Store D"},
      {"lat": 17.738, "lon": 83.312, "name": "Store E"}
    ]
  }' \
  http://localhost:3000/vrp/map
```

### Step 3: Generate VRP Instance

```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "graph_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "vehicles": 2,
    "capacity": 100.0,
    "constraints": {
      "time_windows": false,
      "max_distance": 50000.0,
      "max_duration": 28800.0,
      "service_time": 600.0
    }
  }' \
  http://localhost:3000/vrp/generate
```

**Response:**
```json
{
  "vrp_id": "b2c3d4e5-f6g7-8901-bcde-f23456789012",
  "customers": 5,
  "vehicles": 2,
  "depot_count": 1
}
```

### Step 4: Solve VRP

```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "vrp_id": "b2c3d4e5-f6g7-8901-bcde-f23456789012",
    "algorithm": "multi_start"
  }' \
  http://localhost:3000/vrp/solve
```

**Response:**
```json
{
  "solution_id": "c3d4e5f6-g7h8-9012-cdef-34567890123a",
  "routes": [
    {
      "vehicle_id": 0,
      "path": [1, 2, 3],
      "distance": 4250.0,
      "duration": 5400.0,
      "demand": 75.0,
      "locations": [...]
    },
    {
      "vehicle_id": 1,
      "path": [4, 5],
      "distance": 2800.0,
      "duration": 3600.0,
      "demand": 55.0,
      "locations": [...]
    }
  ],
  "total_cost": 7050.0,
  "total_distance": 7050.0,
  "total_duration": 9000.0,
  "vehicles_used": 2,
  "solve_time_ms": 234.7
}
```

### Step 5: Export for Visualization

```bash
curl 'http://localhost:3000/vrp/solution/c3d4e5f6-g7h8-9012-cdef-34567890123a/export?format=geojson' \
  > solution.geojson
```

### JavaScript Example

```javascript
class VRPSolverAPI {
  constructor(baseUrl = 'http://localhost:3000') {
    this.baseUrl = baseUrl;
  }

  async uploadOSM(file) {
    const formData = new FormData();
    formData.append('file', file);
    
    const response = await fetch(`${this.baseUrl}/osm/upload`, {
      method: 'POST',
      body: formData
    });
    return await response.json();
  }

  async mapLocations(graphId, depot, customers) {
    const response = await fetch(`${this.baseUrl}/vrp/map`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ graph_id: graphId, depot, customers })
    });
    return await response.json();
  }

  async generateVRP(graphId, vehicles, capacity, constraints = {}) {
    const response = await fetch(`${this.baseUrl}/vrp/generate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ 
        graph_id: graphId, 
        vehicles, 
        capacity, 
        constraints: {
          time_windows: false,
          service_time: 300.0,
          ...constraints
        }
      })
    });
    return await response.json();
  }

  async solveVRP(vrpId, algorithm = 'multi_start') {
    const response = await fetch(`${this.baseUrl}/vrp/solve`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ vrp_id: vrpId, algorithm })
    });
    return await response.json();
  }

  async exportSolution(solutionId, format = 'geojson') {
    const response = await fetch(
      `${this.baseUrl}/vrp/solution/${solutionId}/export?format=${format}`
    );
    return await response.json();
  }
}

// Usage example
async function solveVRPProblem() {
  const api = new VRPSolverAPI();
  
  // Upload OSM file
  const fileInput = document.querySelector('#osm-file');
  const osmResult = await api.uploadOSM(fileInput.files[0]);
  console.log('OSM uploaded:', osmResult.graph_id);
  
  // Map locations
  const depot = { lat: 17.735, lon: 83.315, name: "Main Depot" };
  const customers = [
    { lat: 17.737, lon: 83.320, name: "Customer 1" },
    { lat: 17.740, lon: 83.310, name: "Customer 2" }
  ];
  
  await api.mapLocations(osmResult.graph_id, depot, customers);
  
  // Generate VRP
  const vrpResult = await api.generateVRP(
    osmResult.graph_id, 
    2, // vehicles
    50.0, // capacity
    { max_distance: 100000 }
  );
  
  // Solve VRP
  const solution = await api.solveVRP(vrpResult.vrp_id, 'multi_start');
  console.log('Solution found:', solution.solution_id);
  
  // Export for visualization
  const geoJson = await api.exportSolution(solution.solution_id, 'geojson');
  
  // Add to map (using Leaflet example)
  L.geoJSON(geoJson, {
    style: feature => feature.geometry.type === 'LineString' ? {
      color: '#ff7800',
      weight: 5,
      opacity: 0.65
    } : {},
    pointToLayer: (feature, latlng) => {
      const isDepot = feature.properties.type === 'depot';
      return L.marker(latlng, {
        icon: L.icon({
          iconUrl: isDepot ? 'depot-icon.png' : 'customer-icon.png'
        })
      }).bindPopup(feature.properties.name);
    }
  }).addTo(map);
}
```

## Rate Limits

Currently, no rate limits are implemented. For production use, consider implementing rate limiting based on your requirements.

## Performance Considerations

- **OSM File Size**: Large OSM files (>100MB) may take significant time to parse
- **Number of Customers**: Performance scales roughly O(n²) with customer count
- **Algorithm Choice**: 
  - `greedy`: Fastest, good for >100 customers
  - `clarke_wright`: Medium speed, good quality
  - `multi_start`: Slowest but best quality, recommended for <50 customers

## Troubleshooting

### Common Issues

1. **"Graph not found" error**
   - Ensure you're using the correct `graph_id` from the OSM upload response
   - Check that the OSM data was successfully uploaded

2. **"No nodes found" error**  
   - Verify that your coordinates are within the bounds of the uploaded OSM data
   - Ensure coordinates use decimal degrees (not degrees/minutes/seconds)

3. **"No solution found" error**
   - Check that vehicle capacity is sufficient for customer demands
   - Verify that distance/duration constraints are reasonable
   - Try a different algorithm or relaxed constraints

4. **Slow solving performance**
   - Use `greedy` algorithm for large instances (>50 customers)
   - Reduce the number of vehicles if possible
   - Check system resources (CPU/memory usage)

### Support

For issues or questions:
1. Check the server logs for detailed error messages
2. Verify request format matches the API documentation
3. Test with smaller datasets first
4. Consider using the CLI version for debugging: `cargo run --bin vrp-cli`

---

*This documentation covers version 1.0 of the VRP Solver Web API. For updates and additional features, check the project repository.*
