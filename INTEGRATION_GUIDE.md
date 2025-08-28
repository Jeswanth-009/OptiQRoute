# OptiQRoute - Rust Classical Algorithms Integration

This update integrates the Rust-based classical algorithms into your OptiQRoute application.

## What's Changed

### 1. **Backend Integration** (`app.py`)
- Added Rust VRP bridge that calls the classical algorithms
- Enhanced `/classical-route` endpoint to use different algorithms:
  - `greedy` - Greedy Nearest Neighbor
  - `greedy_farthest` - Greedy Farthest Insertion  
  - `clarke_wright` - Clarke-Wright Savings Algorithm
  - `multi_start` - Multi-Start Optimization (runs all algorithms and picks best)
- Added `/algorithms` endpoint to list available algorithms
- Added `/health` endpoint for system status

### 2. **Rust VRP Bridge** (`rust_vrp_bridge.py`)
- Python bridge that compiles and calls the Rust VRP solver
- Handles coordinate conversion and data formatting
- Provides fallback algorithms if Rust compilation fails
- Automatic compilation of Rust code when needed

### 3. **Frontend Enhancements** (`optiqroute-frontend/src/App.js`)
- Added algorithm selection UI for classical optimization
- Real API integration instead of mock data
- Displays algorithm information and performance metrics
- Shows solver type (rust_vrp, osmnx, fallback) in results

### 4. **Rust VRP Solver** (`classical using rust/`)
- Complete VRP solver library with multiple algorithms
- CLI interface for Python bridge integration
- Haversine distance calculations for real-world accuracy
- Parallel processing with Rayon for performance

## Prerequisites

To use the Rust algorithms, you need to install Rust:

### Windows:
1. Download and install from: https://rustup.rs/
2. Run the installer and follow the prompts
3. Restart your terminal/VS Code

### Alternative (if Rust not available):
The system will automatically fall back to:
1. OSMnx-based routing (if available)
2. Simple point-to-point routing (always available)

## How to Run

### 1. Start Backend
```bash
cd c:\OptiQRoute
python app.py
```

### 2. Start Frontend  
```bash
cd c:\OptiQRoute\optiqroute-frontend
npm start
```

### 3. Use the Application
1. Select "Classical" optimization mode
2. Choose your preferred algorithm:
   - **Multi-Start**: Best overall results (recommended)
   - **Clarke-Wright**: Good for complex routes
   - **Nearest Neighbor**: Fast and simple
   - **Farthest Insertion**: Alternative greedy approach
3. Add depot and customer locations
4. Click "Run Analysis"

## Algorithm Details

### Greedy Nearest Neighbor
- **Complexity**: O(n²)
- **Speed**: Very Fast  
- **Quality**: Good for small problems
- **Best for**: Quick routing, real-time applications

### Clarke-Wright Savings
- **Complexity**: O(n²) 
- **Speed**: Medium
- **Quality**: Very Good
- **Best for**: Multiple vehicles, complex constraints

### Multi-Start Optimization
- **Complexity**: O(k×n²)
- **Speed**: Slower but best quality
- **Quality**: Excellent
- **Best for**: When optimal results are needed

## Performance Comparison

The frontend now shows:
- **Algorithm Used**: Which specific algorithm solved the problem
- **Solver Type**: rust_vrp (best), osmnx (fallback), or simple (last resort)
- **Real Distance**: Calculated using Haversine formula
- **Optimization Time**: Actual solving time
- **Vehicles Used**: Number of vehicles in solution

## Troubleshooting

### If Rust compilation fails:
- Check that Rust is installed: `cargo --version`
- Try installing: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- The system will use Python fallback algorithms

### If backend fails:
- Install required packages: `pip install flask flask-cors osmnx networkx`
- The frontend will still work with mock data

### If frontend fails to connect:
- Ensure backend is running on http://localhost:5000
- Check browser console for CORS errors

## Next Steps

1. **Install Rust** for best performance with classical algorithms
2. **Test different algorithms** on your route problems  
3. **Compare performance** between Classical and Quantum modes
4. **Export results** to analyze algorithm effectiveness

The system is now using proper Vehicle Routing Problem algorithms instead of simple shortest path routing!
