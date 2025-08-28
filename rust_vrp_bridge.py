#!/usr/bin/env python3
"""
Rust VRP Bridge - Interface between Python Flask app and Rust VRP solver
"""

import json
import subprocess
import tempfile
import os
from typing import List, Dict, Any, Tuple, Optional
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class RustVrpBridge:
    """Bridge to communicate with Rust VRP solver"""
    
    def __init__(self, rust_executable_path: Optional[str] = None):
        """
        Initialize the bridge
        
        Args:
            rust_executable_path: Path to the compiled Rust VRP executable
        """
        try:
            self.rust_exe = rust_executable_path or self._find_rust_executable()
            self._verify_rust_executable()
            self.rust_available = True
            logger.info(f"Rust VRP solver available at: {self.rust_exe}")
        except Exception as e:
            logger.warning(f"Rust VRP solver not available: {e}")
            self.rust_exe = None
            self.rust_available = False
    
    def _find_rust_executable(self) -> str:
        """Find the Rust VRP executable"""
        # Try different possible locations with correct executable names
        possible_paths = [
            "target/release/vrp_cli.exe",
            "target/debug/vrp_cli.exe", 
            "./vrp_cli.exe",
            "classical using rust/target/release/vrp_cli.exe",
            "classical using rust/target/debug/vrp_cli.exe",
            "target/release/vrp_cli",
            "target/debug/vrp_cli", 
            "./vrp_cli",
            "classical using rust/target/release/vrp_cli",
            "classical using rust/target/debug/vrp_cli"
        ]
        
        for path in possible_paths:
            if os.path.exists(path):
                return path
                
        # If not found, we'll compile it
        return self._compile_rust_solver()
    
    def _compile_rust_solver(self) -> str:
        """Compile the Rust VRP solver"""
        rust_dir = "classical using rust"
        
        if not os.path.exists(rust_dir):
            raise FileNotFoundError("Rust VRP source directory not found")
            
        logger.info("Compiling Rust VRP solver...")
        
        try:
            # Compile in release mode for better performance
            result = subprocess.run(
                ["cargo", "build", "--release", "--bin", "vrp_cli"],
                cwd=rust_dir,
                capture_output=True,
                text=True,
                timeout=300  # 5 minutes timeout
            )
            
            if result.returncode != 0:
                logger.error(f"Cargo build failed: {result.stderr}")
                # Try debug mode if release fails
                result = subprocess.run(
                    ["cargo", "build", "--bin", "vrp_cli"],
                    cwd=rust_dir,
                    capture_output=True,
                    text=True,
                    timeout=300
                )
                
                if result.returncode != 0:
                    raise RuntimeError(f"Failed to compile Rust solver: {result.stderr}")
                    
                exe_name = "vrp_cli.exe" if os.name == 'nt' else "vrp_cli"
                return os.path.join(rust_dir, "target/debug", exe_name)
            
            exe_name = "vrp_cli.exe" if os.name == 'nt' else "vrp_cli"
            return os.path.join(rust_dir, "target/release", exe_name)
            
        except subprocess.TimeoutExpired:
            raise RuntimeError("Rust compilation timed out")
        except FileNotFoundError:
            raise RuntimeError("Cargo not found. Please install Rust and Cargo.")
    
    def _verify_rust_executable(self):
        """Verify that the Rust executable exists and works"""
        if not os.path.exists(self.rust_exe):
            raise FileNotFoundError(f"Rust VRP executable not found at: {self.rust_exe}")
    
    def create_vrp_instance(
        self,
        depot: Tuple[float, float],
        customers: List[Tuple[float, float]],
        vehicle_capacity: float = 100.0,
        num_vehicles: int = 1
    ) -> Dict[str, Any]:
        """
        Create a VRP instance from depot and customer coordinates
        
        Args:
            depot: (lat, lon) tuple for depot location
            customers: List of (lat, lon) tuples for customer locations
            vehicle_capacity: Vehicle capacity constraint
            num_vehicles: Number of available vehicles
            
        Returns:
            VRP instance dictionary
        """
        # Create locations list starting with depot
        locations = [
            {
                "id": 0,
                "name": "Depot",
                "coordinate": {"lat": depot[0], "lon": depot[1]},
                "demand": 0.0,
                "time_window": None,
                "service_time": 0.0
            }
        ]
        
        # Add customers with unit demand
        for i, (lat, lon) in enumerate(customers):
            locations.append({
                "id": i + 1,
                "name": f"Customer {i + 1}",
                "coordinate": {"lat": lat, "lon": lon},
                "demand": 10.0,  # Unit demand per customer
                "time_window": None,
                "service_time": 5.0  # 5 minutes service time
            })
        
        # Create vehicles
        vehicles = []
        for i in range(num_vehicles):
            vehicles.append({
                "id": i,
                "capacity": vehicle_capacity,
                "max_distance": None,
                "max_duration": None,
                "depot_id": 0
            })
        
        return {
            "locations": locations,
            "vehicles": vehicles,
            "distance_matrix": [],  # Will be calculated by Rust
            "time_matrix": None
        }
    
    def solve_vrp(
        self,
        depot: Tuple[float, float],
        customers: List[Tuple[float, float]],
        algorithm: str = "multi_start",
        vehicle_capacity: float = 100.0,
        num_vehicles: int = None
    ) -> Dict[str, Any]:
        """
        Solve VRP using Rust solver
        
        Args:
            depot: (lat, lon) tuple for depot location
            customers: List of (lat, lon) tuples for customer locations  
            algorithm: Algorithm to use ("greedy", "greedy_farthest", "clarke_wright", "multi_start")
            vehicle_capacity: Vehicle capacity constraint
            num_vehicles: Number of vehicles (auto-calculated if None)
            
        Returns:
            Solution dictionary with routes and metrics
        """
        if not customers:
            raise ValueError("At least one customer location is required")
        
        # If Rust is not available, use fallback immediately
        if not self.rust_available:
            logger.info("Rust solver not available, using Python fallback")
            return self._fallback_solve(depot, customers)
        
        # Auto-calculate number of vehicles if not specified
        if num_vehicles is None:
            total_demand = len(customers) * 10.0  # Assuming 10 units demand per customer
            num_vehicles = max(1, int(total_demand / vehicle_capacity) + 1)
        
        # Create VRP instance
        vrp_instance = self.create_vrp_instance(
            depot, customers, vehicle_capacity, num_vehicles
        )
        
        # Create input JSON for Rust solver
        input_data = {
            "instance": vrp_instance,
            "algorithm": algorithm,
            "settings": {
                "distance_method": "haversine",
                "parallel": True
            }
        }
        
        # Write input to temporary file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(input_data, f, indent=2)
            input_file = f.name
        
        try:
            # Create output file
            with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
                output_file = f.name
            
            # Run Rust solver
            cmd = [self.rust_exe, "--input", input_file, "--output", output_file]
            
            logger.info(f"Running Rust VRP solver: {' '.join(cmd)}")
            
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=60  # 1 minute timeout
            )
            
            if result.returncode != 0:
                logger.error(f"Rust solver failed: {result.stderr}")
                # Fall back to simple algorithm implementation
                return self._fallback_solve(depot, customers)
            
            # Read and parse output
            with open(output_file, 'r') as f:
                solution = json.load(f)
            
            return self._format_solution(solution, depot, customers)
            
        except subprocess.TimeoutExpired:
            logger.error("Rust solver timed out")
            return self._fallback_solve(depot, customers)
        except Exception as e:
            logger.error(f"Error running Rust solver: {e}")
            return self._fallback_solve(depot, customers)
        finally:
            # Cleanup temporary files
            for file_path in [input_file, output_file]:
                if os.path.exists(file_path):
                    os.unlink(file_path)
    
    def _fallback_solve(
        self,
        depot: Tuple[float, float],
        customers: List[Tuple[float, float]]
    ) -> Dict[str, Any]:
        """
        Simple fallback algorithm when Rust solver fails
        """
        logger.info("Using fallback Python solver")
        
        # Simple nearest neighbor algorithm
        route = [depot]
        unvisited = customers.copy()
        current = depot
        total_distance = 0.0
        
        while unvisited:
            # Find nearest unvisited customer
            nearest_idx = 0
            min_dist = self._calculate_distance(current, unvisited[0])
            
            for i, customer in enumerate(unvisited[1:], 1):
                dist = self._calculate_distance(current, customer)
                if dist < min_dist:
                    min_dist = dist
                    nearest_idx = i
            
            # Move to nearest customer
            nearest = unvisited.pop(nearest_idx)
            route.append(nearest)
            total_distance += min_dist
            current = nearest
        
        # Return to depot
        route.append(depot)
        total_distance += self._calculate_distance(current, depot)
        
        return {
            "routes": [{
                "vehicle_id": 0,
                "locations": list(range(len(route))),
                "coordinates": route,
                "total_distance": total_distance,
                "total_duration": total_distance / 1000 * 3.6,  # Approximate time in minutes
                "total_demand": len(customers) * 10.0
            }],
            "total_distance": total_distance,
            "total_duration": total_distance / 1000 * 3.6,
            "num_vehicles_used": 1,
            "algorithm": "fallback_nearest_neighbor"
        }
    
    def _calculate_distance(self, coord1: Tuple[float, float], coord2: Tuple[float, float]) -> float:
        """Calculate haversine distance between two coordinates in meters"""
        import math
        
        lat1, lon1 = coord1
        lat2, lon2 = coord2
        
        R = 6371000  # Earth radius in meters
        
        lat1_rad = math.radians(lat1)
        lat2_rad = math.radians(lat2)
        delta_lat = math.radians(lat2 - lat1)
        delta_lon = math.radians(lon2 - lon1)
        
        a = (math.sin(delta_lat / 2) ** 2 + 
             math.cos(lat1_rad) * math.cos(lat2_rad) * math.sin(delta_lon / 2) ** 2)
        c = 2 * math.atan2(math.sqrt(a), math.sqrt(1 - a))
        
        return R * c
    
    def _format_solution(
        self,
        solution: Dict[str, Any],
        depot: Tuple[float, float], 
        customers: List[Tuple[float, float]]
    ) -> Dict[str, Any]:
        """Format Rust solver output for Python backend"""
        
        # Create coordinate lookup
        all_coords = [depot] + customers
        
        formatted_routes = []
        for route in solution.get("routes", []):
            # Convert location IDs back to coordinates
            route_locations = route.get("locations", [])
            
            # Build complete route: depot -> customers -> depot
            coordinates = [depot]  # Start at depot
            
            # Add customer coordinates
            for loc_id in route_locations:
                if loc_id < len(all_coords) and loc_id != 0:  # Skip depot ID if present
                    coordinates.append(all_coords[loc_id])
            
            # Return to depot
            if len(coordinates) > 1:  # Only if we have customers
                coordinates.append(depot)
            
            # Build complete location IDs list: [0] + customer_ids + [0]
            complete_locations = [0]  # Start at depot (ID 0)
            for loc_id in route_locations:
                if loc_id != 0:  # Add customer IDs
                    complete_locations.append(loc_id)
            if len(complete_locations) > 1:  # Return to depot
                complete_locations.append(0)

            formatted_routes.append({
                "vehicle_id": route.get("vehicle_id", 0),
                "locations": complete_locations,
                "coordinates": coordinates,
                "total_distance": route.get("total_distance", 0.0),
                "total_duration": route.get("total_duration", 0.0),
                "total_demand": route.get("total_demand", 0.0)
            })
        
        return {
            "routes": formatted_routes,
            "total_distance": solution.get("total_distance", 0.0),
            "total_duration": solution.get("total_duration", 0.0),
            "num_vehicles_used": solution.get("num_vehicles_used", 1),
            "algorithm": solution.get("algorithm", "unknown")
        }


def test_bridge():
    """Test the Rust VRP bridge"""
    bridge = RustVrpBridge()
    
    # Test data - Visakhapatnam area
    depot = (17.6868, 83.2185)
    customers = [
        (17.7231, 83.3109),  # Customer 1
        (17.7326, 83.3093),  # Customer 2  
        (17.6583, 83.2154),  # Customer 3
    ]
    
    try:
        solution = bridge.solve_vrp(depot, customers, algorithm="multi_start")
        print("VRP Solution:")
        print(json.dumps(solution, indent=2))
        return True
    except Exception as e:
        print(f"Test failed: {e}")
        return False


if __name__ == "__main__":
    test_bridge()
