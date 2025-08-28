#!/usr/bin/env python3
"""
Test script for the quantum VRP endpoint
"""

import requests
import json
import time

def test_quantum_endpoint():
    url = "http://127.0.0.1:5000/quantum-route"
    
    # Test data - small problem for quantum feasibility
    test_data = {
        "start": [17.6868, 83.2185],  # Depot in Visakhapatnam
        "deliveries": [
            [17.7000, 83.2300],      # Customer 1
            [17.6950, 83.2250]       # Customer 2
        ],
        "num_vehicles": 2,
        "vehicle_capacity": 100
    }
    
    print("🧪 Testing Quantum VRP Endpoint")
    print("=" * 50)
    print(f"URL: {url}")
    print(f"Request data: {json.dumps(test_data, indent=2)}")
    print("=" * 50)
    
    try:
        print("⚛️  Sending request to quantum endpoint...")
        start_time = time.time()
        
        response = requests.post(
            url, 
            json=test_data,
            headers={'Content-Type': 'application/json'},
            timeout=60  # Give quantum algorithm time to run
        )
        
        end_time = time.time()
        response_time = (end_time - start_time) * 1000  # ms
        
        print(f"📡 Response received in {response_time:.1f}ms")
        print(f"🔢 Status code: {response.status_code}")
        
        if response.status_code == 200:
            result = response.json()
            print("✅ Quantum VRP Success!")
            print("=" * 50)
            print(f"Algorithm: {result.get('algorithm', 'Unknown')}")
            print(f"Solver: {result.get('solver', 'Unknown')}")
            print(f"Total distance: {result.get('distance_m', 0):.2f} km")
            print(f"Vehicles used: {result.get('num_vehicles_used', 0)}")
            print(f"Solve time: {result.get('solve_time_ms', 0):.1f}ms")
            print(f"Quantum advantage: {result.get('quantum_advantage', 'N/A')}")
            
            if 'routes' in result and result['routes']:
                print(f"\n🛣️  Routes ({len(result['routes'])} total):")
                for i, route in enumerate(result['routes']):
                    print(f"  Route {i+1}: {route.get('customers_served', 0)} customers, {route.get('distance', 0):.2f} km")
                    
            if 'quantum_details' in result:
                details = result['quantum_details']
                print(f"\n⚛️  Quantum Details:")
                print(f"  Bitstring: {details.get('bitstring', 'N/A')}")
                print(f"  QAOA layers: {details.get('qaoa_layers', 'N/A')}")
                print(f"  Optimization iterations: {details.get('optimization_iterations', 'N/A')}")
                print(f"  Quantum measurements: {len(details.get('quantum_measurements', {}))}")
            
        else:
            print(f"❌ Request failed!")
            print(f"Response: {response.text}")
            
    except requests.exceptions.Timeout:
        print("⏱️  Request timed out - quantum algorithm may be taking too long")
    except requests.exceptions.ConnectionError:
        print("🔌 Connection error - is the Flask app running?")
    except Exception as e:
        print(f"💥 Unexpected error: {e}")

def test_classical_comparison():
    """Test classical endpoint for comparison"""
    url = "http://127.0.0.1:5000/classical-route"
    
    test_data = {
        "start": [17.6868, 83.2185],
        "deliveries": [
            [17.7000, 83.2300],
            [17.6950, 83.2250]
        ],
        "num_vehicles": 2,
        "vehicle_capacity": 100
    }
    
    print("\n🏁 Testing Classical VRP for Comparison")
    print("=" * 50)
    
    try:
        start_time = time.time()
        response = requests.post(url, json=test_data, timeout=30)
        end_time = time.time()
        response_time = (end_time - start_time) * 1000
        
        print(f"📡 Classical response in {response_time:.1f}ms")
        
        if response.status_code == 200:
            result = response.json()
            print(f"✅ Classical distance: {result.get('distance_m', 0):.2f} km")
            print(f"⚡ Classical solve time: {result.get('solve_time_ms', response_time):.1f}ms")
        else:
            print(f"❌ Classical request failed: {response.text}")
            
    except Exception as e:
        print(f"💥 Classical test error: {e}")

if __name__ == "__main__":
    print("🚀 OptiQRoute Quantum VRP Test")
    print("Testing 100% quantum QAOA implementation")
    print("This should be slower than classical for small problems (realistic)")
    
    # Test quantum endpoint
    test_quantum_endpoint()
    
    # Test classical for comparison
    test_classical_comparison()
    
    print("\n" + "=" * 50)
    print("✨ Test completed!")
    print("Note: Quantum being slower than classical is expected and realistic")
    print("Quantum advantage appears only with larger problems and future hardware")
