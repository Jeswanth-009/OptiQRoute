## 🚀 End-to-End Workflow

This project allows you to go from **raw OSM data → VRP instance → solver → GeoJSON visualization** in just a few steps.

### **Step 1: Convert OSM PBF to JSON/GeoJSON**
Parse the `.osm.pbf` file into structured JSON and GeoJSON.
```bash
cargo run --bin osm_converter -- --input vishakapatnam.osm.pbf --roads-only
```
**Outputs:**
- `vishakapatnam.osm.pbf.json` → structured OSM road data
- `vishakapatnam.osm.pbf.geojson` → raw road visualization

---

### **Step 2: Generate VRP Instance from OSM Data**
Create a Vehicle Routing Problem instance (depot + customers) from the converted OSM data.
```bash
cargo run --bin planet_83_example -- --depot-lat 17.735 --depot-lon 83.315 --customers 10
```
**Outputs:**
- `osm_vrp_instance.json` → VRP problem with depot and customers

---

### **Step 3: Solve the VRP Instance**
Run the solver with heuristics (multi-start for diversity).
```bash
cargo run --bin vrp_solver -- --instance osm_vrp_instance.json --algorithm multi-start
```
**Outputs:**
- `solution.json` → optimized routes with distance, duration, and demands

---

### **Step 4: Convert Solution to GeoJSON**
Export the solution into GeoJSON for visualization in tools like Mapbox, Leaflet, or geojson.io.
```bash
cargo run --bin vrp_to_geojson \
  --solution solution.json \
  --osm-data vishakapatnam.osm.pbf.json \
  --geojson vishakapatnam_routes.geojson \
  --include-points \
  --depot-lat 17.735 --depot-lon 83.315
```
**Outputs:**
- `vishakapatnam_routes.geojson` → routes + depot + customer points

---

### ✅ Summary
1. **OSM → JSON/GeoJSON** (roads extracted)
2. **JSON → VRP Instance** (customers + depot)
3. **VRP Solver** (multi-start baseline)
4. **VRP Solution → GeoJSON** (visualization-ready)

This makes the backend pipeline clean, Rust-native, and fast. The **frontend team** or visualization tools can directly consume the generated `*.geojson` files.

