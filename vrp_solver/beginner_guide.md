# 🚚 VRP Web Server - Beginner's Guide

Welcome! This guide will walk you through using the Vehicle Routing Problem (VRP) web server step-by-step. You'll learn how to solve real-world delivery routing problems using OpenStreetMap data.

## 📋 What You'll Learn

By the end of this guide, you'll know how to:
1. Start the VRP web server
2. Upload OpenStreetMap data for your area
3. Define depot and customer locations
4. Generate and solve routing problems
5. Export routes for visualization on maps

## 🔧 Prerequisites

### Required Software
- **Rust Programming Language** (1.70+)
  - Download from: https://rustup.rs/
  - Follow installation instructions for your OS
- **Web Browser** (Chrome, Firefox, Edge, Safari)
- **Text Editor** (VS Code, Notepad++, or any editor)

### Optional but Helpful
- **Postman** or **Insomnia** (for API testing)
- **curl** command (usually pre-installed on most systems)

### Getting the Code
```bash
# Clone the repository (replace with actual repository URL)
git clone <repository-url>
cd vrp_solver

# Build the project
cargo build --release
```

---

## 🚀 Step 1: Start the VRP Web Server

### Starting the Server

Open your terminal/command prompt and navigate to the VRP solver directory:

```bash
cd path/to/vrp_solver
cargo run --bin vrp-server
```

You should see output similar to:
```
INFO vrp_solver: Starting VRP web server...
INFO vrp_solver: Server running on http://0.0.0.0:3000
```

✅ **Success!** The server is now running on `http://localhost:3000`

### Verify Server is Running

Open your web browser and go to: `http://localhost:3000/health`

You should see:
```json
{
  "status": "healthy",
  "timestamp": 1735083600,
  "stats": {
    "graphs": 0,
    "mappings": 0,
    "vrp_instances": 0,
    "solutions": 0
  }
}
```

---

## 🗺️ Step 2: Get OpenStreetMap Data

### Option A: Download from Geofabrik (Recommended for Beginners)

1. **Visit**: https://download.geofabrik.de/
2. **Navigate** to your country/region
3. **Download** the `.osm.pbf` file for your area

**Examples:**
- **USA**: https://download.geofabrik.de/north-america/us/
- **Europe**: https://download.geofabrik.de/europe/
- **India**: https://download.geofabrik.de/asia/india/

**⚠️ Start Small**: Choose a city or state-level file first (not the entire country).

**Good beginner options:**
- Monaco: https://download.geofabrik.de/europe/monaco-latest.osm.pbf (~200KB)
- Rhode Island: https://download.geofabrik.de/north-america/us/rhode-island-latest.osm.pbf (~5MB)
- Delaware: https://download.geofabrik.de/north-america/us/delaware-latest.osm.pbf (~10MB)

### Option B: Extract Custom Area

For advanced users, you can extract specific areas using tools like:
- **Overpass Turbo**: https://overpass-turbo.eu/
- **OSM Extract**: https://extract.bbbike.org/

---

## 📂 Step 3: Upload Your OSM Data

### Method 1: Using curl (Command Line)

Save your downloaded PBF file to the VRP solver directory, then:

```bash
# Replace 'your-map.osm.pbf' with your actual filename
curl -X POST -F "file=@your-map.osm.pbf" http://localhost:3000/osm/upload
```

**Expected Response:**
```json
{
  "graph_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "nodes": 15234,
  "edges": 8745,
  "message": "Successfully parsed OSM data: 15234 nodes, 8745 ways"
}
```

**📝 Important**: Save the `graph_id` - you'll need it for the next steps!

### Method 2: Using URL (if file is online)

```bash
# Upload directly from URL
curl -X POST -F "file_url=https://download.geofabrik.de/europe/monaco-latest.osm.pbf" http://localhost:3000/osm/upload
```

### Method 3: Using Postman/Insomnia

1. **Create new POST request** to `http://localhost:3000/osm/upload`
2. **Set Body type** to `form-data`
3. **Add field** `file` and select your PBF file
4. **Send request**

---

## 📍 Step 4: Define Your Locations

Now you need to define where your depot (warehouse/distribution center) and customers are located.

### Find Your Coordinates

Use one of these methods to get latitude/longitude coordinates:

1. **Google Maps**: Right-click on a location → click coordinates that appear
2. **OpenStreetMap**: Right-click → "Show address" → coordinates shown in URL
3. **GPS coordinates**: Use any mapping app on your phone

### Create Location File

Create a file called `my_locations.json`:

```json
{
  "graph_id": "YOUR_GRAPH_ID_FROM_STEP_3",
  "depot": {
    "lat": 40.7589,
    "lon": -73.9851,
    "name": "Main Warehouse"
  },
  "customers": [
    {
      "lat": 40.7505,
      "lon": -73.9934,
      "name": "Customer A"
    },
    {
      "lat": 40.7614,
      "lon": -73.9776,
      "name": "Customer B"
    },
    {
      "lat": 40.7505,
      "lon": -73.9857,
      "name": "Customer C"
    }
  ]
}
```

**⚠️ Important**: Replace `YOUR_GRAPH_ID_FROM_STEP_3` with the actual graph_id from Step 3.

### Upload Location Mapping

```bash
curl -X POST -H "Content-Type: application/json" --data @my_locations.json http://localhost:3000/vrp/map
```

**Expected Response:**
```json
{
  "mapped_depot": {
    "node_id": 12345,
    "lat": 40.758901,
    "lon": -73.985102,
    "distance_to_original": 25.5
  },
  "mapped_customers": [
    {
      "node_id": 67890,
      "lat": 40.750502,
      "lon": -73.993403,
      "distance_to_original": 15.2
    }
  ]
}
```

**📝 Note**: The server maps your coordinates to the nearest road intersections. The `distance_to_original` shows how far (in meters) the mapped location is from your original coordinates.

---

## 🚛 Step 5: Create VRP Problem Instance

Define your vehicle constraints and create the routing problem.

### Create Problem Configuration

Create a file called `vrp_problem.json`:

```json
{
  "graph_id": "YOUR_GRAPH_ID_FROM_STEP_3",
  "vehicles": 2,
  "capacity": 100.0,
  "constraints": {
    "time_windows": false,
    "max_distance": 50000.0,
    "max_duration": 14400.0,
    "service_time": 600.0
  }
}
```

**Configuration Explained:**
- `vehicles`: Number of delivery trucks/vehicles available
- `capacity`: Maximum load each vehicle can carry (in your units - kg, boxes, etc.)
- `max_distance`: Maximum distance per vehicle in meters (50km = 50,000m)
- `max_duration`: Maximum time per vehicle in seconds (4 hours = 14,400s)
- `service_time`: Time spent at each customer in seconds (10 minutes = 600s)

### Generate VRP Instance

```bash
curl -X POST -H "Content-Type: application/json" --data @vrp_problem.json http://localhost:3000/vrp/generate
```

**Expected Response:**
```json
{
  "vrp_id": "b2c3d4e5-f6g7-8901-bcde-f23456789012",
  "customers": 3,
  "vehicles": 2,
  "depot_count": 1
}
```

**📝 Important**: Save the `vrp_id` - you'll need it to solve the problem!

---

## 🧠 Step 6: Solve the Routing Problem

Now for the exciting part - let the AI solve your routing problem!

### Choose Your Algorithm

| Algorithm | Best For | Speed | Quality |
|-----------|----------|-------|---------|
| `greedy` | Small problems (<20 customers) | ⚡ Fastest | 🥉 Good |
| `clarke_wright` | Medium problems (20-50 customers) | 🚶 Medium | 🥈 Better |
| `multi_start` | Any size, when quality matters | 🐌 Slower | 🥇 Best |

### Create Solve Request

Create a file called `solve_request.json`:

```json
{
  "vrp_id": "YOUR_VRP_ID_FROM_STEP_5",
  "algorithm": "multi_start"
}
```

**Replace `YOUR_VRP_ID_FROM_STEP_5`** with the actual vrp_id from Step 5.

### Solve the Problem

```bash
curl -X POST -H "Content-Type: application/json" --data @solve_request.json http://localhost:3000/vrp/solve
```

**Expected Response:**
```json
{
  "solution_id": "c3d4e5f6-g7h8-9012-cdef-34567890123a",
  "routes": [
    {
      "vehicle_id": 0,
      "path": [1, 2],
      "distance": 2500.0,
      "duration": 3600.0,
      "demand": 50.0,
      "locations": [
        {
          "id": 1,
          "name": "Customer A",
          "lat": 40.750502,
          "lon": -73.993403,
          "demand": 25.0,
          "service_time": 600.0
        },
        {
          "id": 2,
          "name": "Customer B",
          "lat": 40.761402,
          "lon": -73.977603,
          "demand": 25.0,
          "service_time": 600.0
        }
      ]
    }
  ],
  "total_cost": 4500.0,
  "total_distance": 4500.0,
  "total_duration": 7200.0,
  "vehicles_used": 1,
  "solve_time_ms": 15.5
}
```

**🎉 Success!** You now have an optimized delivery route!

**📝 Important**: Save the `solution_id` for exporting and visualization.

---

## 📊 Step 7: Export and Visualize Routes

### Export as GeoJSON for Mapping

```bash
curl "http://localhost:3000/vrp/solution/YOUR_SOLUTION_ID/export?format=geojson" > my_routes.geojson
```

**Replace `YOUR_SOLUTION_ID`** with the solution_id from Step 6.

### Visualize Your Routes

#### Option 1: Using geojson.io (Easiest)

1. **Go to**: http://geojson.io/
2. **Drag and drop** your `my_routes.geojson` file
3. **View your routes** on the interactive map!

#### Option 2: Using QGIS (Professional)

1. **Download QGIS**: https://qgis.org/
2. **Open QGIS**
3. **Layer** → **Add Layer** → **Add Vector Layer**
4. **Select** your `my_routes.geojson` file
5. **View** professional mapping visualization

#### Option 3: Custom Web Map

Create an HTML file:

```html
<!DOCTYPE html>
<html>
<head>
    <title>My VRP Routes</title>
    <link rel="stylesheet" href="https://unpkg.com/leaflet/dist/leaflet.css" />
</head>
<body>
    <div id="map" style="height: 600px;"></div>
    
    <script src="https://unpkg.com/leaflet/dist/leaflet.js"></script>
    <script>
        // Initialize map
        const map = L.map('map').setView([40.7589, -73.9851], 13);
        
        // Add OpenStreetMap tiles
        L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png').addTo(map);
        
        // Load your GeoJSON file
        fetch('my_routes.geojson')
            .then(response => response.json())
            .then(data => {
                L.geoJSON(data, {
                    style: feature => feature.geometry.type === 'LineString' ? 
                        { color: '#ff7800', weight: 5 } : {},
                    pointToLayer: (feature, latlng) => {
                        const isDepot = feature.properties.type === 'depot';
                        return L.marker(latlng)
                            .bindPopup(feature.properties.name);
                    }
                }).addTo(map);
            });
    </script>
</body>
</html>
```

---

## 📈 Step 8: Analyze Your Results

### Understanding the Output

From your solution response, here's what each field means:

```json
{
  "total_distance": 4500.0,     // Total km all vehicles travel
  "total_duration": 7200.0,     // Total time in seconds (2 hours)
  "vehicles_used": 1,           // How many vehicles were needed
  "solve_time_ms": 15.5         // How fast the AI solved it
}
```

### Route Details

For each route:
- **`vehicle_id`**: Which vehicle (0, 1, 2, etc.)
- **`path`**: Order of visiting customers [1, 2, 3]
- **`distance`**: Total distance for this vehicle
- **`duration`**: Total time including driving + service time
- **`demand`**: Total load carried by this vehicle

### Performance Tips

- **Distance < 50km per vehicle**: Good for local deliveries
- **Duration < 8 hours**: Fits in a work day
- **Vehicles_used**: Fewer vehicles = more efficient routes

---

## 🆘 Troubleshooting Common Issues

### "Server not responding"
- Check if server is still running: `http://localhost:3000/health`
- Restart server: `Ctrl+C` then `cargo run --bin vrp-server`

### "Graph not found" error
- Make sure you're using the correct `graph_id` from the upload response
- Check that OSM upload was successful

### "No nodes found" error
- Verify your coordinates are within the OSM data area
- Use decimal degrees format: `40.7589, -73.9851`
- Check coordinates aren't swapped (latitude first, longitude second)

### "No solution found" error
- Increase vehicle capacity
- Increase `max_distance` or `max_duration`
- Reduce number of customers or vehicles
- Try different algorithm (`greedy` is more flexible)

### File upload fails
- Ensure PBF file isn't corrupted
- Try smaller geographic area
- Check file permissions

---

## 🎯 Real-World Example Walkthrough

Let's solve a pizza delivery problem in Manhattan!

### Step 1: Get NYC Data
```bash
# Download Manhattan area (small file, good for testing)
curl -o manhattan.osm.pbf "https://download.geofabrik.de/north-america/us/new-york-latest.osm.pbf"
```

### Step 2: Upload Data
```bash
curl -X POST -F "file=@manhattan.osm.pbf" http://localhost:3000/osm/upload
# Save the graph_id from response!
```

### Step 3: Define Pizza Shop and Customers
```json
{
  "graph_id": "your-actual-graph-id",
  "depot": {
    "lat": 40.7589,
    "lon": -73.9851,
    "name": "Tony's Pizza Shop"
  },
  "customers": [
    {"lat": 40.7505, "lon": -73.9934, "name": "Empire State Building"},
    {"lat": 40.7614, "lon": -73.9776, "name": "Central Park"},
    {"lat": 40.7505, "lon": -73.9857, "name": "Flatiron Building"},
    {"lat": 40.7484, "lon": -73.9857, "name": "Madison Square Park"},
    {"lat": 40.7527, "lon": -73.9772, "name": "Union Square"}
  ]
}
```

### Step 4: Configure Delivery Problem
```json
{
  "graph_id": "your-actual-graph-id",
  "vehicles": 2,
  "capacity": 10.0,
  "constraints": {
    "time_windows": false,
    "max_distance": 25000.0,
    "max_duration": 10800.0,
    "service_time": 300.0
  }
}
```

### Step 5: Solve and Export
```bash
# Upload locations
curl -X POST -H "Content-Type: application/json" --data @pizza_locations.json http://localhost:3000/vrp/map

# Generate VRP
curl -X POST -H "Content-Type: application/json" --data @pizza_problem.json http://localhost:3000/vrp/generate

# Solve with best algorithm
curl -X POST -H "Content-Type: application/json" -d '{"vrp_id":"your-vrp-id","algorithm":"multi_start"}' http://localhost:3000/vrp/solve

# Export routes
curl "http://localhost:3000/vrp/solution/your-solution-id/export?format=geojson" > pizza_routes.geojson
```

**Result**: Optimized pizza delivery routes that save time and gas! 🍕

---

## 🎓 What's Next?

### Advanced Features to Explore

1. **Time Windows**: Add delivery time constraints
2. **Different Vehicle Types**: Varying capacities and speeds
3. **Multiple Depots**: Multiple warehouses/stores
4. **Real-time Traffic**: Integration with traffic APIs

### Integration Ideas

- **Fleet Management Systems**
- **E-commerce Platforms**
- **Food Delivery Apps**
- **Service Technician Routing**
- **Waste Collection Optimization**

### Performance Optimization

- **Larger Datasets**: Test with entire cities/regions
- **Algorithm Comparison**: Benchmark different solving methods
- **Caching**: Store frequently used routes
- **Batch Processing**: Handle multiple requests simultaneously

---

## 📚 Additional Resources

### Documentation
- **Full API Reference**: `web_server.md`
- **Technical Details**: `webserver_prototype.md`

### Learning More About VRP
- **Wikipedia**: Vehicle Routing Problem
- **OR-Tools**: Google's optimization tools
- **Academic Papers**: Search for "vehicle routing problem" on Google Scholar

### OpenStreetMap Resources
- **Learn OSM**: https://learnosm.org/
- **OSM Wiki**: https://wiki.openstreetmap.org/
- **Overpass API**: Query OSM data

### Getting Help
- Check the server logs for detailed error messages
- Start with small datasets and simple problems
- Join the VRP and logistics optimization communities

---

**🎉 Congratulations!** You've successfully learned how to use the VRP web server to solve real-world routing optimization problems. Happy routing! 🚚📍

---

*Last updated: January 2025*
